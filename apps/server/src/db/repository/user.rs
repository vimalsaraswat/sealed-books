//! User identity model and database queries.

use crate::db::error::DbError;
use libsql::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub pubkey: String,
    pub eth_address: String,
    pub wallet_id: Option<String>,
    pub created_at: String,
}

impl User {
    /// Inserts a new user into the database.
    pub async fn insert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO users (id, email, name, pubkey, eth_address, wallet_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
            params![
                self.id.as_str(),
                self.email.as_str(),
                self.name.as_str(),
                self.pubkey.as_str(),
                self.eth_address.as_str(),
                self.wallet_id.as_deref(),
                self.created_at.as_str()
            ],
        )
        .await?;
        Ok(())
    }

    /// Finds a user by their unique user ID.
    pub async fn find_by_id(conn: &Connection, id: &str) -> Result<Self, DbError> {
        let mut rows = conn
            .query(
                "SELECT id, email, name, pubkey, eth_address, wallet_id, created_at FROM users WHERE id = ?1;",
                params![id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(User {
                id: row.get(0)?,
                email: row.get(1)?,
                name: row.get(2)?,
                pubkey: row.get(3)?,
                eth_address: row.get(4)?,
                wallet_id: row.get(5)?,
                created_at: row.get(6)?,
            })
        } else {
            Err(DbError::EntityNotFound(format!("User not found: {id}")))
        }
    }

    /// Finds a user by their unique email address.
    pub async fn find_by_email(conn: &Connection, email: &str) -> Result<Self, DbError> {
        let mut rows = conn
            .query(
                "SELECT id, email, name, pubkey, eth_address, wallet_id, created_at FROM users WHERE email = ?1;",
                params![email],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(User {
                id: row.get(0)?,
                email: row.get(1)?,
                name: row.get(2)?,
                pubkey: row.get(3)?,
                eth_address: row.get(4)?,
                wallet_id: row.get(5)?,
                created_at: row.get(6)?,
            })
        } else {
            Err(DbError::EntityNotFound(format!(
                "User with email '{email}' not found"
            )))
        }
    }

    /// Lists all registered users.
    pub async fn list_all(conn: &Connection) -> Result<Vec<Self>, DbError> {
        let mut rows = conn
            .query(
                "SELECT id, email, name, pubkey, eth_address, wallet_id, created_at FROM users ORDER BY name ASC;",
                (),
            )
            .await?;

        let mut users = Vec::new();
        while let Some(row) = rows.next().await? {
            users.push(User {
                id: row.get(0)?,
                email: row.get(1)?,
                name: row.get(2)?,
                pubkey: row.get(3)?,
                eth_address: row.get(4)?,
                wallet_id: row.get(5)?,
                created_at: row.get(6)?,
            });
        }
        Ok(users)
    }

    /// Updates a user's wallet_id and eth_address.
    pub async fn update_wallet(
        conn: &Connection,
        user_id: &str,
        wallet_id: &str,
        eth_address: &str,
    ) -> Result<(), DbError> {
        conn.execute(
            "UPDATE users SET wallet_id = ?1, eth_address = ?2 WHERE id = ?3;",
            params![wallet_id, eth_address, user_id],
        )
        .await?;
        Ok(())
    }
}
