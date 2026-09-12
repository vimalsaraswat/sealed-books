//! Database connection lifecycle and schema management for Sealed Books Server.

pub mod error;
pub mod repository;
pub mod schema;

use libsql::Connection;

pub use error::DbError;
pub use repository::*;
pub use schema::migrate;

/// Thread-safe handle to the database connection (supports both local SQLite files and remote Turso/libSQL).
#[derive(Clone)]
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Resolves the database target from environment variables.
    ///
    /// Checks the following in order of precedence:
    /// 1. `DATABASE_URL` (standard in cloud providers & ORMs)
    /// 2. `TURSO_DATABASE_URL`
    /// 3. `DATABASE_PATH` (legacy local path)
    /// 4. Fallback default: `"sealed_books.db"`
    pub fn resolve_target_from_env() -> String {
        std::env::var("DATABASE_URL")
            .or_else(|_| std::env::var("TURSO_DATABASE_URL"))
            .or_else(|_| std::env::var("DATABASE_PATH"))
            .unwrap_or_else(|_| "sealed_books.db".to_string())
    }

    /// Opens or connects to a database from a path, URI, or remote Turso endpoint.
    ///
    /// Supports:
    /// - Remote Turso / libSQL: `libsql://your-db.turso.io`, `https://your-db.turso.io`, `turso://your-db.turso.io`
    /// - Standard local file paths: `sealed_books.db`, `/tmp/sealed_books.db`
    /// - SQLite URIs: `sqlite://data.db`, `sqlite:///var/data/ledger.db`
    pub async fn open(target: &str) -> Result<Self, DbError> {
        let auth_token = std::env::var("TURSO_AUTH_TOKEN")
            .or_else(|_| std::env::var("DATABASE_AUTH_TOKEN"))
            .unwrap_or_default();

        let db = if target.starts_with("libsql://")
            || target.starts_with("https://")
            || target.starts_with("http://")
            || target.starts_with("turso://")
        {
            let remote_url = if let Some(stripped) = target.strip_prefix("turso://") {
                format!("https://{stripped}")
            } else {
                target.to_string()
            };
            tracing::info!("Connecting to remote Turso database at {}", remote_url);
            libsql::Builder::new_remote(remote_url, auth_token)
                .build()
                .await?
        } else {
            // Strip common database URL prefixes for local SQLite
            let sanitized = target
                .strip_prefix("sqlite://")
                .or_else(|| target.strip_prefix("sqlite:"))
                .unwrap_or(target);

            // Strip query params if standard file path
            let file_path = if let Some(stripped) = sanitized.strip_prefix("file:") {
                stripped.split('?').next().unwrap_or(stripped)
            } else {
                sanitized.split('?').next().unwrap_or(sanitized)
            };

            // Ensure parent directory exists if target is a file path
            if file_path != ":memory:" {
                let path = std::path::Path::new(file_path);
                if let Some(parent) = path.parent()
                    && !parent.as_os_str().is_empty()
                {
                    let _ = std::fs::create_dir_all(parent);
                }
            }

            tracing::info!("Opening local SQLite database at {}", file_path);
            libsql::Builder::new_local(file_path).build().await?
        };

        let conn = db.connect()?;
        Self::init(conn).await
    }

    /// Opens an in-memory database (for tests) and runs schema migrations.
    pub async fn open_in_memory() -> Result<Self, DbError> {
        let db = libsql::Builder::new_local(":memory:").build().await?;
        let conn = db.connect()?;
        Self::init(conn).await
    }

    async fn init(conn: Connection) -> Result<Self, DbError> {
        // Enforce foreign keys pragmas
        let _ = conn.execute("PRAGMA foreign_keys = ON;", ()).await;

        // Run schema migrations
        schema::migrate(&conn).await?;

        Ok(Self { conn })
    }

    /// Returns a reference to the active database connection.
    ///
    /// In libSQL, `Connection` is `Clone + Send + Sync`, allowing safe concurrent queries across async tasks.
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Clone handle for the connection.
    pub fn conn_cloned(&self) -> Connection {
        self.conn.clone()
    }

    /// Compatibility helper for existing code migrating from rusqlite lock.
    pub fn lock(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_open_with_sqlite_uri_prefix() {
        let temp_dir = std::env::temp_dir();
        let target = format!("sqlite://{}/test_uri_prefix.db", temp_dir.display());
        let db = Database::open(&target)
            .await
            .expect("open with sqlite:// URI prefix");
        let conn = db.conn();
        let mut rows = conn.query("SELECT 1", ()).await.unwrap();
        let row = rows.next().await.unwrap().unwrap();
        let val: i32 = row.get(0).unwrap();
        assert_eq!(val, 1);
        let _ = std::fs::remove_file(format!("{}/test_uri_prefix.db", temp_dir.display()));
    }

    #[tokio::test]
    async fn test_open_with_file_flags_uri() {
        let temp_dir = std::env::temp_dir();
        let target = format!(
            "file:{}/test_flags.db?mode=rwc&cache=shared",
            temp_dir.display()
        );
        let db = Database::open(&target)
            .await
            .expect("open with file: flags URI");
        let conn = db.conn();
        let mut rows = conn.query("SELECT 42", ()).await.unwrap();
        let row = rows.next().await.unwrap().unwrap();
        let val: i32 = row.get(0).unwrap();
        assert_eq!(val, 42);
        let _ = std::fs::remove_file(format!("{}/test_flags.db", temp_dir.display()));
    }
}
