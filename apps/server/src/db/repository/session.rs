//! User session model and database queries.

use crate::db::error::DbError;
use libsql::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub user_id: String,
    pub active_organization_id: String,
    pub expires_at: String,
}

impl Session {
    /// Inserts a new user session.
    pub async fn insert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO sessions (token, user_id, active_organization_id, expires_at)
             VALUES (?1, ?2, ?3, ?4);",
            params![
                self.token.as_str(),
                self.user_id.as_str(),
                self.active_organization_id.as_str(),
                self.expires_at.as_str()
            ],
        )
        .await?;
        Ok(())
    }

    /// Finds an active session by bearer token.
    pub async fn find_by_token(conn: &Connection, token: &str) -> Result<Self, DbError> {
        let mut rows = conn
            .query(
                "SELECT token, user_id, active_organization_id, expires_at
                 FROM sessions WHERE token = ?1;",
                params![token],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Session {
                token: row.get(0)?,
                user_id: row.get(1)?,
                active_organization_id: row.get(2)?,
                expires_at: row.get(3)?,
            })
        } else {
            Err(DbError::EntityNotFound(
                "Session not found or expired".into(),
            ))
        }
    }

    /// Updates the active organization ID for this session.
    pub async fn update_active_org(
        conn: &Connection,
        token: &str,
        new_org_id: &str,
    ) -> Result<(), DbError> {
        conn.execute(
            "UPDATE sessions SET active_organization_id = ?1 WHERE token = ?2;",
            params![new_org_id, token],
        )
        .await?;
        Ok(())
    }

    /// Deletes a session upon logout.
    pub async fn delete(conn: &Connection, token: &str) -> Result<(), DbError> {
        conn.execute("DELETE FROM sessions WHERE token = ?1;", params![token])
            .await?;
        Ok(())
    }
}
