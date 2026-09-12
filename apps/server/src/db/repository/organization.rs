//! Organization tenant model and database queries.

use crate::db::error::DbError;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub base_currency: String,
    pub created_at: String,
}

impl Organization {
    /// Inserts this organization into the database.
    pub fn insert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO organizations (id, name, base_currency, created_at) VALUES (?1, ?2, ?3, ?4);",
            params![&self.id, &self.name, &self.base_currency, &self.created_at],
        )?;
        Ok(())
    }

    /// Finds an organization by its unique ID.
    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Self, DbError> {
        conn.query_row(
            "SELECT id, name, base_currency, created_at FROM organizations WHERE id = ?1;",
            params![id],
            |row| {
                Ok(Organization {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    base_currency: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => {
                DbError::EntityNotFound(format!("Organization not found: {id}"))
            }
            other => DbError::Sqlite(other),
        })
    }

    /// Lists all organizations that the given user belongs to, including their role.
    pub fn list_for_user(conn: &Connection, user_id: &str) -> Result<Vec<(Self, String)>, DbError> {
        let mut stmt = conn.prepare(
            "SELECT o.id, o.name, o.base_currency, o.created_at, m.role
             FROM organizations o
             JOIN organization_memberships m ON o.id = m.organization_id
             WHERE m.user_id = ?1 AND m.status = 'active'
             ORDER BY o.name ASC;",
        )?;

        let orgs = stmt
            .query_map(params![user_id], |row| {
                Ok((
                    Organization {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        base_currency: row.get(2)?,
                        created_at: row.get(3)?,
                    },
                    row.get(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(orgs)
    }
}
