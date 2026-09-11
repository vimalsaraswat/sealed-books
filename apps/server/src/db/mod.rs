//! Database connection lifecycle and schema management for Sealed Books Server.

pub mod error;
pub mod repository;
pub mod schema;

use rusqlite::Connection;
use std::sync::{Arc, Mutex, MutexGuard};

pub use error::DbError;
pub use repository::*;
pub use schema::migrate;

/// Thread-safe handle to the SQLite database connection.
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Opens or creates a SQLite database file at `path` and runs schema migrations.
    pub fn open(path: &str) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    /// Opens an in-memory SQLite database (for tests) and runs schema migrations.
    pub fn open_in_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        Self::init(conn)
    }

    fn init(conn: Connection) -> Result<Self, DbError> {
        // Enforce foreign keys and concurrency pragmas
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        )?;

        // Run schema migrations
        schema::migrate(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Acquires a lock on the database connection.
    pub fn lock(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().expect("SQLite mutex poisoned")
    }
}
