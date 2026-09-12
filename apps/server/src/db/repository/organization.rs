//! Organization tenant model and database queries.

use crate::db::error::DbError;
use libsql::{Connection, params};
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
    pub async fn insert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO organizations (id, name, base_currency, created_at) VALUES (?1, ?2, ?3, ?4);",
            params![
                self.id.as_str(),
                self.name.as_str(),
                self.base_currency.as_str(),
                self.created_at.as_str()
            ],
        ).await?;
        Ok(())
    }

    /// Finds an organization by its unique ID.
    pub async fn find_by_id(conn: &Connection, id: &str) -> Result<Self, DbError> {
        let mut rows = conn
            .query(
                "SELECT id, name, base_currency, created_at FROM organizations WHERE id = ?1;",
                params![id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Organization {
                id: row.get(0)?,
                name: row.get(1)?,
                base_currency: row.get(2)?,
                created_at: row.get(3)?,
            })
        } else {
            Err(DbError::EntityNotFound(format!(
                "Organization not found: {id}"
            )))
        }
    }

    /// Lists all organizations that the given user belongs to, including their role.
    pub async fn list_for_user(
        conn: &Connection,
        user_id: &str,
    ) -> Result<Vec<(Self, String)>, DbError> {
        let mut rows = conn
            .query(
                "SELECT o.id, o.name, o.base_currency, o.created_at, m.role
                 FROM organizations o
                 JOIN organization_memberships m ON o.id = m.organization_id
                 WHERE m.user_id = ?1 AND m.status = 'active'
                 ORDER BY o.name ASC;",
                params![user_id],
            )
            .await?;

        let mut orgs = Vec::new();
        while let Some(row) = rows.next().await? {
            orgs.push((
                Organization {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    base_currency: row.get(2)?,
                    created_at: row.get(3)?,
                },
                row.get(4)?,
            ));
        }

        Ok(orgs)
    }
}
