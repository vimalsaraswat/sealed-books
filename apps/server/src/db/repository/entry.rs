//! Journal entry repository: SQLite persistence and conversion to/from `sealed_books_core::types::Entry`.

use crate::db::error::DbError;
use rusqlite::{Connection, params};
use sealed_books_core::types::{Direction, Entry, Line};

/// Extension trait providing database queries for `Entry`.
pub trait EntryExt {
    /// Persists this entry and all of its balanced lines in a single database transaction.
    ///
    /// Validates double-entry balance, verifies that the target period is not sealed,
    /// and ensures all referenced account IDs exist.
    fn post_to(&self, conn: &Connection, period_id: &str) -> Result<(), DbError>;

    /// Loads all entries and lines for a specific accounting period,
    /// returned in deterministic order (by entry timestamp, then entry ID, lines by line ID).
    fn find_by_period(conn: &Connection, period_id: &str) -> Result<Vec<Entry>, DbError>;

    /// Loads a single entry by its unique ID.
    fn find_by_id(conn: &Connection, id: &str) -> Result<Entry, DbError>;
}

impl EntryExt for Entry {
    fn post_to(&self, conn: &Connection, period_id: &str) -> Result<(), DbError> {
        // 1. Validate double-entry balance and at least 1 debit and credit
        self.validate()?;

        // 2. Check that target period exists and is open
        let period_status: String = conn
            .query_row(
                "SELECT status FROM periods WHERE id = ?1;",
                params![period_id],
                |row| row.get(0),
            )
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => {
                    DbError::PeriodNotFound(period_id.to_string())
                }
                other => DbError::Sqlite(other),
            })?;

        if period_status == "sealed" {
            return Err(DbError::PeriodSealed(period_id.to_string()));
        }

        // 3. Verify all account IDs exist in the chart of accounts
        for line in &self.lines {
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM accounts WHERE id = ?1);",
                    params![&line.account_id],
                    |row| row.get(0),
                )
                .map_err(DbError::Sqlite)?;

            if !exists {
                return Err(DbError::AccountNotFound(line.account_id.clone()));
            }
        }

        // 4. Atomic transaction: insert entry header and all lines
        conn.execute(
            "INSERT INTO entries (id, period_id, date, description, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5);",
            params![
                &self.id,
                period_id,
                &self.date,
                &self.description,
                &self.created_at,
            ],
        )
        .map_err(DbError::Sqlite)?;

        let mut line_stmt = conn
            .prepare(
                "INSERT INTO lines (id, entry_id, account_id, direction, amount_minor, description)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
            )
            .map_err(DbError::Sqlite)?;

        for line in &self.lines {
            let dir_str = match line.direction {
                Direction::Debit => "debit",
                Direction::Credit => "credit",
            };

            line_stmt
                .execute(params![
                    &line.id,
                    &self.id,
                    &line.account_id,
                    dir_str,
                    line.amount_minor as i64,
                    &line.description,
                ])
                .map_err(DbError::Sqlite)?;
        }

        Ok(())
    }

    fn find_by_period(conn: &Connection, period_id: &str) -> Result<Vec<Entry>, DbError> {
        let mut entry_stmt = conn
            .prepare(
                "SELECT id, date, description, created_at
                 FROM entries
                 WHERE period_id = ?1
                 ORDER BY created_at ASC, id ASC;",
            )
            .map_err(DbError::Sqlite)?;

        let entry_rows = entry_stmt
            .query_map(params![period_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(DbError::Sqlite)?;

        let mut entries = Vec::new();

        for entry_res in entry_rows {
            let (id, date, description, created_at) = entry_res.map_err(DbError::Sqlite)?;

            let lines = load_lines_for_entry(conn, &id)?;

            entries.push(Entry {
                id,
                date,
                description,
                created_at,
                lines,
            });
        }

        Ok(entries)
    }

    fn find_by_id(conn: &Connection, id: &str) -> Result<Entry, DbError> {
        let (entry_id, date, description, created_at) = conn
            .query_row(
                "SELECT id, date, description, created_at
                 FROM entries
                 WHERE id = ?1;",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => DbError::EntryNotFound(id.to_string()),
                other => DbError::Sqlite(other),
            })?;

        let lines = load_lines_for_entry(conn, &entry_id)?;

        Ok(Entry {
            id: entry_id,
            date,
            description,
            created_at,
            lines,
        })
    }
}

/// Helper function to retrieve all lines for an entry in deterministic order (by line ID).
fn load_lines_for_entry(conn: &Connection, entry_id: &str) -> Result<Vec<Line>, DbError> {
    let mut line_stmt = conn
        .prepare(
            "SELECT id, account_id, direction, amount_minor, description
             FROM lines
             WHERE entry_id = ?1
             ORDER BY id ASC;",
        )
        .map_err(DbError::Sqlite)?;

    let line_rows = line_stmt
        .query_map(params![entry_id], |row| {
            let id: String = row.get(0)?;
            let account_id: String = row.get(1)?;
            let dir_str: String = row.get(2)?;
            let amount_minor: i64 = row.get(3)?;
            let description: Option<String> = row.get(4)?;

            let direction = match dir_str.as_str() {
                "debit" => Direction::Debit,
                "credit" => Direction::Credit,
                _ => {
                    return Err(rusqlite::Error::InvalidColumnType(
                        2,
                        "direction".into(),
                        rusqlite::types::Type::Text,
                    ));
                }
            };

            Ok(Line {
                id,
                account_id,
                direction,
                amount_minor: amount_minor as u64,
                description,
            })
        })
        .map_err(DbError::Sqlite)?;

    let mut lines = Vec::new();
    for lr in line_rows {
        lines.push(lr.map_err(DbError::Sqlite)?);
    }

    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repository::account::Account;
    use crate::db::repository::period::PeriodRecord;
    use crate::db::schema::migrate;
    use sealed_books_core::hash_entry;

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

    fn sample_period() -> PeriodRecord {
        PeriodRecord {
            id: "per_aug_2026".into(),
            organization_id: "org_acme".into(),
            entity: "Acme Trading Pvt Ltd".into(),
            start_date: "2026-08-01".into(),
            end_date: "2026-08-31".into(),
            status: "open".into(),
        }
    }

    fn seed_accounts(conn: &Connection) {
        Account {
            id: "acc_bank".into(),
            organization_id: "org_acme".into(),
            code: "1010".into(),
            name: "Bank Checking".into(),
            account_type: "asset".into(),
        }
        .insert(conn)
        .unwrap();

        Account {
            id: "acc_sales".into(),
            organization_id: "org_acme".into(),
            code: "4010".into(),
            name: "Sales Revenue".into(),
            account_type: "revenue".into(),
        }
        .insert(conn)
        .unwrap();
    }

    #[test]
    fn test_save_to_db_and_reload_preserves_canonical_hash() {
        let mut conn = setup_test_db();
        seed_accounts(&conn);
        let period = sample_period();
        period.insert(&mut conn).unwrap();

        let original_entry = Entry {
            id: "ent_001".into(),
            date: "2026-08-15".into(),
            description: "Online product sale".into(),
            created_at: "2026-08-15T10:00:00Z".into(),
            lines: vec![
                Line::new("l_01", "acc_bank", Direction::Debit, 5000, None).unwrap(),
                Line::new("l_02", "acc_sales", Direction::Credit, 5000, None).unwrap(),
            ],
        };

        let original_hash = hash_entry(&original_entry).unwrap();

        original_entry.post_to(&conn, &period.id).unwrap();

        let loaded_entry = Entry::find_by_id(&conn, "ent_001").unwrap();
        let loaded_hash = hash_entry(&loaded_entry).unwrap();

        assert_eq!(
            original_hash, loaded_hash,
            "Re-loaded database entry hash must match original in-memory hash exactly"
        );
    }

    #[test]
    fn test_reject_unbalanced_entry() {
        let mut conn = setup_test_db();
        seed_accounts(&conn);
        let period = sample_period();
        period.insert(&mut conn).unwrap();

        let unbalanced = Entry {
            id: "ent_bad".into(),
            date: "2026-08-15".into(),
            description: "Unbalanced".into(),
            created_at: "2026-08-15T10:00:00Z".into(),
            lines: vec![
                Line::new("l_01", "acc_bank", Direction::Debit, 5000, None).unwrap(),
                Line::new("l_02", "acc_sales", Direction::Credit, 4000, None).unwrap(),
            ],
        };

        let res = unbalanced.post_to(&conn, &period.id);
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), DbError::Core(_)));
    }

    #[test]
    fn test_reject_posting_to_sealed_period() {
        let mut conn = setup_test_db();
        seed_accounts(&conn);
        let mut period = sample_period();
        period.status = "sealed".into();
        period.insert(&mut conn).unwrap();

        let entry = Entry {
            id: "ent_002".into(),
            date: "2026-08-15".into(),
            description: "Sale".into(),
            created_at: "2026-08-15T10:00:00Z".into(),
            lines: vec![
                Line::new("l_01", "acc_bank", Direction::Debit, 5000, None).unwrap(),
                Line::new("l_02", "acc_sales", Direction::Credit, 5000, None).unwrap(),
            ],
        };

        let res = entry.post_to(&conn, &period.id);
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), DbError::PeriodSealed(_)));
    }

    #[test]
    fn test_reject_nonexistent_account() {
        let mut conn = setup_test_db();
        let period = sample_period();
        period.insert(&mut conn).unwrap();

        let entry = Entry {
            id: "ent_003".into(),
            date: "2026-08-15".into(),
            description: "Sale".into(),
            created_at: "2026-08-15T10:00:00Z".into(),
            lines: vec![
                Line::new("l_01", "acc_fake_999", Direction::Debit, 5000, None).unwrap(),
                Line::new("l_02", "acc_fake_999", Direction::Credit, 5000, None).unwrap(),
            ],
        };

        let res = entry.post_to(&conn, &period.id);
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), DbError::AccountNotFound(_)));
    }
}
