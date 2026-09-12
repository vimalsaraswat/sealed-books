//! Accounting period entity model and queries.

use crate::db::error::DbError;
use rusqlite::{Connection, params};
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
    pub fn insert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO periods (id, organization_id, entity, start_date, end_date, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
            params![
                &self.id,
                if self.organization_id.is_empty() { "org_acme" } else { &self.organization_id },
                &self.entity,
                &self.start_date,
                &self.end_date,
                &self.status
            ],
        )?;
        Ok(())
    }

    /// Finds a period by its unique ID.
    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Self, DbError> {
        conn.query_row(
            "SELECT id, organization_id, entity, start_date, end_date, status FROM periods WHERE id = ?1;",
            params![id],
            |row| {
                Ok(PeriodRecord {
                    id: row.get(0)?,
                    organization_id: row.get(1)?,
                    entity: row.get(2)?,
                    start_date: row.get(3)?,
                    end_date: row.get(4)?,
                    status: row.get(5)?,
                })
            },
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => DbError::PeriodNotFound(id.to_string()),
            other => DbError::Sqlite(other),
        })
    }

    /// Lists all periods for a specific organization ordered by start date.
    pub fn list_by_org(conn: &Connection, org_id: &str) -> Result<Vec<Self>, DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, organization_id, entity, start_date, end_date, status FROM periods WHERE organization_id = ?1 ORDER BY start_date;",
        )?;
        let periods = stmt
            .query_map(params![org_id], |row| {
                Ok(PeriodRecord {
                    id: row.get(0)?,
                    organization_id: row.get(1)?,
                    entity: row.get(2)?,
                    start_date: row.get(3)?,
                    end_date: row.get(4)?,
                    status: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(periods)
    }

    /// Lists all periods ordered by start date.
    pub fn list_all(conn: &Connection) -> Result<Vec<Self>, DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, organization_id, entity, start_date, end_date, status FROM periods ORDER BY start_date;",
        )?;
        let periods = stmt
            .query_map([], |row| {
                Ok(PeriodRecord {
                    id: row.get(0)?,
                    organization_id: row.get(1)?,
                    entity: row.get(2)?,
                    start_date: row.get(3)?,
                    end_date: row.get(4)?,
                    status: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(periods)
    }

    /// Marks a period as sealed (immutable).
    pub fn mark_sealed(conn: &Connection, period_id: &str) -> Result<(), DbError> {
        let rows_affected = conn.execute(
            "UPDATE periods SET status = 'sealed' WHERE id = ?1;",
            params![period_id],
        )?;
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

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();
        migrate(&conn).unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO organizations (id, name, base_currency, created_at) VALUES ('org_acme', 'Acme', 'USD', '2026-08-01T00:00:00Z');",
            [],
        ).unwrap();
        conn
    }

    #[test]
    fn test_insert_and_find_by_id() {
        let conn = setup_test_db();
        let pr = PeriodRecord {
            id: "per_2026_08".into(),
            organization_id: "org_acme".into(),
            entity: "Acme Corp".into(),
            start_date: "2026-08-01".into(),
            end_date: "2026-08-31".into(),
            status: "open".into(),
        };

        pr.insert(&conn).unwrap();
        let fetched = PeriodRecord::find_by_id(&conn, "per_2026_08").unwrap();
        assert_eq!(fetched, pr);
    }

    #[test]
    fn test_find_by_id_not_found() {
        let conn = setup_test_db();
        let err = PeriodRecord::find_by_id(&conn, "per_missing").unwrap_err();
        assert!(matches!(err, DbError::PeriodNotFound(_)));
    }

    #[test]
    fn test_list_all_ordered_by_start_date() {
        let conn = setup_test_db();
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

        p2.insert(&conn).unwrap();
        p1.insert(&conn).unwrap();

        let list = PeriodRecord::list_all(&conn).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, "per_2026_08");
        assert_eq!(list[1].id, "per_2026_09");
    }

    #[test]
    fn test_mark_sealed_transitions_status() {
        let conn = setup_test_db();
        let p = PeriodRecord {
            id: "per_2026_08".into(),
            organization_id: "org_acme".into(),
            entity: "Acme Corp".into(),
            start_date: "2026-08-01".into(),
            end_date: "2026-08-31".into(),
            status: "open".into(),
        };
        p.insert(&conn).unwrap();

        PeriodRecord::mark_sealed(&conn, "per_2026_08").unwrap();
        let updated = PeriodRecord::find_by_id(&conn, "per_2026_08").unwrap();
        assert_eq!(updated.status, "sealed");
    }

    #[test]
    fn test_mark_sealed_nonexistent_returns_error() {
        let conn = setup_test_db();
        let err = PeriodRecord::mark_sealed(&conn, "per_missing").unwrap_err();
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

        let core = pr.to_core();
        assert_eq!(core.id, "per_2026_08");
        assert_eq!(core.entity, "Acme Corp");
        assert_eq!(core.start_date, "2026-08-01");
        assert_eq!(core.end_date, "2026-08-31");
    }
}
