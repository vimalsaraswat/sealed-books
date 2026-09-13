//! Organization membership model and database queries.

use crate::db::error::DbError;
use crate::db::repository::user::User;
use libsql::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Membership {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub role: String,
    pub status: String,
    pub created_at: String,
}

impl Membership {
    /// Inserts a new membership record into the database.
    pub async fn insert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO organization_memberships (id, organization_id, user_id, role, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
            params![
                self.id.as_str(),
                self.organization_id.as_str(),
                self.user_id.as_str(),
                self.role.as_str(),
                self.status.as_str(),
                self.created_at.as_str()
            ],
        ).await?;
        Ok(())
    }

    /// Finds a user's membership within a specific organization.
    pub async fn find(conn: &Connection, org_id: &str, user_id: &str) -> Result<Self, DbError> {
        let mut rows = conn
            .query(
                "SELECT id, organization_id, user_id, role, status, created_at
                 FROM organization_memberships
                 WHERE organization_id = ?1 AND user_id = ?2;",
                params![org_id, user_id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Membership {
                id: row.get(0)?,
                organization_id: row.get(1)?,
                user_id: row.get(2)?,
                role: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
            })
        } else {
            Err(DbError::EntityNotFound(format!(
                "User {user_id} has no membership in organization {org_id}"
            )))
        }
    }

    /// Lists all members of an organization with their user profiles.
    pub async fn list_by_org(
        conn: &Connection,
        org_id: &str,
    ) -> Result<Vec<(Self, User)>, DbError> {
        let mut rows = conn
            .query(
                "SELECT m.id, m.organization_id, m.user_id, m.role, m.status, m.created_at,
                        u.id, u.email, u.name, u.pubkey, u.eth_address, u.wallet_id, u.created_at
                 FROM organization_memberships m
                 JOIN users u ON m.user_id = u.id
                 WHERE m.organization_id = ?1
                 ORDER BY u.name ASC;",
                params![org_id],
            )
            .await?;

        let mut members = Vec::new();
        while let Some(row) = rows.next().await? {
            members.push((
                Membership {
                    id: row.get(0)?,
                    organization_id: row.get(1)?,
                    user_id: row.get(2)?,
                    role: row.get(3)?,
                    status: row.get(4)?,
                    created_at: row.get(5)?,
                },
                User {
                    id: row.get(6)?,
                    email: row.get(7)?,
                    name: row.get(8)?,
                    pubkey: row.get(9)?,
                    eth_address: row.get(10)?,
                    wallet_id: row.get(11)?,
                    created_at: row.get(12)?,
                },
            ));
        }

        Ok(members)
    }
}
