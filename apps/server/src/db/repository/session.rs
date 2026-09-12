//! User session model and database queries.

use crate::db::error::DbError;
use rusqlite::{Connection, params};
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
    pub fn insert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO sessions (token, user_id, active_organization_id, expires_at)
             VALUES (?1, ?2, ?3, ?4);",
            params![
                &self.token,
                &self.user_id,
                &self.active_organization_id,
                &self.expires_at
            ],
        )?;
        Ok(())
    }

    /// Finds an active session by bearer token.
    pub fn find_by_token(conn: &Connection, token: &str) -> Result<Self, DbError> {
        conn.query_row(
            "SELECT token, user_id, active_organization_id, expires_at
             FROM sessions WHERE token = ?1;",
            params![token],
            |row| {
                Ok(Session {
                    token: row.get(0)?,
                    user_id: row.get(1)?,
                    active_organization_id: row.get(2)?,
                    expires_at: row.get(3)?,
                })
            },
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => {
                DbError::EntityNotFound("Session not found or expired".into())
            }
            other => DbError::Sqlite(other),
        })
    }

    /// Updates the active organization ID for this session.
    pub fn update_active_org(
        conn: &Connection,
        token: &str,
        new_org_id: &str,
    ) -> Result<(), DbError> {
        conn.execute(
            "UPDATE sessions SET active_organization_id = ?1 WHERE token = ?2;",
            params![new_org_id, token],
        )?;
        Ok(())
    }

    /// Deletes a session upon logout.
    pub fn delete(conn: &Connection, token: &str) -> Result<(), DbError> {
        conn.execute("DELETE FROM sessions WHERE token = ?1;", params![token])?;
        Ok(())
    }
}
