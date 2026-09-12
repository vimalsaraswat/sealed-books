//! User identity model and database queries.

use crate::db::error::DbError;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub pubkey: String,
    pub eth_address: String,
    pub created_at: String,
}

impl User {
    /// Inserts a new user into the database.
    pub fn insert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO users (id, email, name, pubkey, eth_address, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
            params![
                &self.id,
                &self.email,
                &self.name,
                &self.pubkey,
                &self.eth_address,
                &self.created_at
            ],
        )?;
        Ok(())
    }

    /// Finds a user by their unique user ID.
    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Self, DbError> {
        conn.query_row(
            "SELECT id, email, name, pubkey, eth_address, created_at FROM users WHERE id = ?1;",
            params![id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    name: row.get(2)?,
                    pubkey: row.get(3)?,
                    eth_address: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => {
                DbError::EntityNotFound(format!("User not found: {id}"))
            }
            other => DbError::Sqlite(other),
        })
    }

    /// Finds a user by their unique email address.
    pub fn find_by_email(conn: &Connection, email: &str) -> Result<Self, DbError> {
        conn.query_row(
            "SELECT id, email, name, pubkey, eth_address, created_at FROM users WHERE email = ?1;",
            params![email],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    name: row.get(2)?,
                    pubkey: row.get(3)?,
                    eth_address: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => {
                DbError::EntityNotFound(format!("User with email '{email}' not found"))
            }
            other => DbError::Sqlite(other),
        })
    }

    /// Lists all registered users.
    pub fn list_all(conn: &Connection) -> Result<Vec<Self>, DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, email, name, pubkey, eth_address, created_at FROM users ORDER BY name ASC;",
        )?;
        let users = stmt
            .query_map([], |row| {
                Ok(User {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    name: row.get(2)?,
                    pubkey: row.get(3)?,
                    eth_address: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(users)
    }
}
