//! Organization membership model and database queries.

use crate::db::error::DbError;
use crate::db::repository::user::User;
use rusqlite::{Connection, params};
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
    pub fn insert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO organization_memberships (id, organization_id, user_id, role, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
            params![
                &self.id,
                &self.organization_id,
                &self.user_id,
                &self.role,
                &self.status,
                &self.created_at
            ],
        )?;
        Ok(())
    }

    /// Finds a user's membership within a specific organization.
    pub fn find(conn: &Connection, org_id: &str, user_id: &str) -> Result<Self, DbError> {
        conn.query_row(
            "SELECT id, organization_id, user_id, role, status, created_at
             FROM organization_memberships
             WHERE organization_id = ?1 AND user_id = ?2;",
            params![org_id, user_id],
            |row| {
                Ok(Membership {
                    id: row.get(0)?,
                    organization_id: row.get(1)?,
                    user_id: row.get(2)?,
                    role: row.get(3)?,
                    status: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => DbError::EntityNotFound(format!(
                "User {user_id} has no membership in organization {org_id}"
            )),
            other => DbError::Sqlite(other),
        })
    }

    /// Lists all members of an organization with their user profiles.
    pub fn list_by_org(conn: &Connection, org_id: &str) -> Result<Vec<(Self, User)>, DbError> {
        let mut stmt = conn.prepare(
            "SELECT m.id, m.organization_id, m.user_id, m.role, m.status, m.created_at,
                    u.id, u.email, u.name, u.pubkey, u.eth_address, u.created_at
             FROM organization_memberships m
             JOIN users u ON m.user_id = u.id
             WHERE m.organization_id = ?1
             ORDER BY u.name ASC;",
        )?;

        let members = stmt
            .query_map(params![org_id], |row| {
                Ok((
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
                        created_at: row.get(11)?,
                    },
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(members)
    }
}
