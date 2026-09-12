//! Accounting period entity model and queries.

use crate::db::error::DbError;
use libsql::{Connection, params};
use sealed_books_core::types::Period;
use serde::{Deserialize, Serialize};

fn default_org_id() -> String {
    "org_acme".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeriodRecord {
    pub id: String,
    #[serde(default = "default_org_id")]
    pub organization_id: String,
    pub entity: String,
    pub start_date: String,
    pub end_date: String,
    pub status: String,
}

impl PeriodRecord {
    /// Converts this database record into a pure domain `Period` for `sealed_books_core`.
    pub fn to_core(&self) -> Period {
        Period {
            id: self.id.clone(),
            entity: self.entity.clone(),
            start_date: self.start_date.clone(),
            end_date: self.end_date.clone(),
        }
    }

    /// Inserts this period into the database.
    pub async fn insert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO periods (id, organization_id, entity, start_date, end_date, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
            params![
                self.id.as_str(),
                if self.organization_id.is_empty() { "org_acme" } else { self.organization_id.as_str() },
                self.entity.as_str(),
                self.start_date.as_str(),
                self.end_date.as_str(),
                self.status.as_str()
            ],
        ).await?;
        Ok(())
    }

    /// Finds a period by its unique ID.
    pub async fn find_by_id(conn: &Connection, id: &str) -> Result<Self, DbError> {
        let mut rows = conn
            .query(
                "SELECT id, organization_id, entity, start_date, end_date, status FROM periods WHERE id = ?1;",
                params![id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(PeriodRecord {
                id: row.get(0)?,
                organization_id: row.get(1)?,
                entity: row.get(2)?,
                start_date: row.get(3)?,
                end_date: row.get(4)?,
                status: row.get(5)?,
            })
        } else {
            Err(DbError::PeriodNotFound(id.to_string()))
        }
    }

    /// Lists all periods for a specific organization ordered by start date.
    pub async fn list_by_org(conn: &Connection, org_id: &str) -> Result<Vec<Self>, DbError> {
        let mut rows = conn
            .query(
                "SELECT id, organization_id, entity, start_date, end_date, status FROM periods WHERE organization_id = ?1 ORDER BY start_date;",
                params![org_id],
            )
            .await?;

        let mut periods = Vec::new();
        while let Some(row) = rows.next().await? {
            periods.push(PeriodRecord {
                id: row.get(0)?,
                organization_id: row.get(1)?,
                entity: row.get(2)?,
                start_date: row.get(3)?,
                end_date: row.get(4)?,
                status: row.get(5)?,
            });
        }
        Ok(periods)
    }

    /// Lists all periods ordered by start date.
    pub async fn list_all(conn: &Connection) -> Result<Vec<Self>, DbError> {
        let mut rows = conn
            .query(
                "SELECT id, organization_id, entity, start_date, end_date, status FROM periods ORDER BY start_date;",
                (),
            )
            .await?;

        let mut periods = Vec::new();
        while let Some(row) = rows.next().await? {
            periods.push(PeriodRecord {
                id: row.get(0)?,
                organization_id: row.get(1)?,
                entity: row.get(2)?,
                start_date: row.get(3)?,
                end_date: row.get(4)?,
                status: row.get(5)?,
            });
        }
        Ok(periods)
    }

    /// Marks a period as sealed (immutable).
    pub async fn mark_sealed(conn: &Connection, period_id: &str) -> Result<(), DbError> {
        let rows_affected = conn
            .execute(
                "UPDATE periods SET status = 'sealed' WHERE id = ?1;",
                params![period_id],
            )
            .await?;

        if rows_affected == 0 {
            return Err(DbError::PeriodNotFound(period_id.to_string()));
        }
        Ok(())
    }
}

impl From<&PeriodRecord> for Period {
    fn from(p: &PeriodRecord) -> Self {
        p.to_core()
    }
}

impl From<PeriodRecord> for Period {
    fn from(p: PeriodRecord) -> Self {
        p.to_core()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::migrate;

    async fn setup_test_db() -> Connection {
        let db = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .unwrap();
        let conn = db.connect().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", ()).await.unwrap();
        migrate(&conn).await.unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO organizations (id, name, base_currency, created_at) VALUES ('org_acme', 'Acme', 'USD', '2026-08-01T00:00:00Z');",
            (),
        ).await.unwrap();
        conn
    }

    #[tokio::test]
    async fn test_insert_and_find_by_id() {
        let conn = setup_test_db().await;
        let pr = PeriodRecord {
            id: "per_2026_08".into(),
            organization_id: "org_acme".into(),
            entity: "Acme Corp".into(),
            start_date: "2026-08-01".into(),
            end_date: "2026-08-31".into(),
            status: "open".into(),
        };

        pr.insert(&conn).await.unwrap();
        let fetched = PeriodRecord::find_by_id(&conn, "per_2026_08")
            .await
            .unwrap();
        assert_eq!(fetched, pr);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let conn = setup_test_db().await;
        let err = PeriodRecord::find_by_id(&conn, "per_missing")
            .await
            .unwrap_err();
        assert!(matches!(err, DbError::PeriodNotFound(_)));
    }

    #[tokio::test]
    async fn test_list_all_ordered_by_start_date() {
        let conn = setup_test_db().await;
        let p2 = PeriodRecord {
            id: "per_2026_09".into(),
            organization_id: "org_acme".into(),
            entity: "Acme Corp".into(),
            start_date: "2026-09-01".into(),
            end_date: "2026-09-30".into(),
            status: "open".into(),
        };
        let p1 = PeriodRecord {
            id: "per_2026_08".into(),
            organization_id: "org_acme".into(),
            entity: "Acme Corp".into(),
            start_date: "2026-08-01".into(),
            end_date: "2026-08-31".into(),
            status: "open".into(),
        };
        p2.insert(&conn).await.unwrap();
        p1.insert(&conn).await.unwrap();

        let list = PeriodRecord::list_all(&conn).await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, "per_2026_08");
        assert_eq!(list[1].id, "per_2026_09");
    }

    #[tokio::test]
    async fn test_mark_sealed_transitions_status() {
        let conn = setup_test_db().await;
        let pr = PeriodRecord {
            id: "per_2026_08".into(),
            organization_id: "org_acme".into(),
            entity: "Acme Corp".into(),
            start_date: "2026-08-01".into(),
            end_date: "2026-08-31".into(),
            status: "open".into(),
        };
        pr.insert(&conn).await.unwrap();

        PeriodRecord::mark_sealed(&conn, "per_2026_08")
            .await
            .unwrap();
        let updated = PeriodRecord::find_by_id(&conn, "per_2026_08")
            .await
            .unwrap();
        assert_eq!(updated.status, "sealed");
    }

    #[tokio::test]
    async fn test_mark_sealed_nonexistent_returns_error() {
        let conn = setup_test_db().await;
        let err = PeriodRecord::mark_sealed(&conn, "per_missing")
            .await
            .unwrap_err();
        assert!(matches!(err, DbError::PeriodNotFound(_)));
    }

    #[test]
    fn test_conversion_to_core_period() {
        let pr = PeriodRecord {
            id: "per_2026_08".into(),
            organization_id: "org_acme".into(),
            entity: "Acme Corp".into(),
            start_date: "2026-08-01".into(),
            end_date: "2026-08-31".into(),
            status: "open".into(),
        };
        let core: Period = pr.to_core();
        assert_eq!(core.id, "per_2026_08");
        assert_eq!(core.entity, "Acme Corp");
        assert_eq!(core.start_date, "2026-08-01");
        assert_eq!(core.end_date, "2026-08-31");
    }
}
