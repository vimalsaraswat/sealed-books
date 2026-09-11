//! Journal entry posting, line items management, and canonical roundtrip queries.

use crate::db::error::DbError;
use crate::db::repository::period::PeriodRecord;
use rusqlite::{Connection, params};
use sealed_books_core::types::{Direction, Entry, Line};

/// Extension trait providing database persistence and querying for `sealed_books_core::Entry`.
pub trait EntryExt {
    /// Posts this balanced journal entry to an open accounting period in the database.
    fn post_to(&self, conn: &mut Connection, period_id: &str) -> Result<(), DbError>;

    /// Finds a specific entry by its unique ID.
    fn find_by_id(conn: &Connection, id: &str) -> Result<Entry, DbError>;

    /// Retrieves all journal entries and lines for a period, ordered deterministically.
    fn find_by_period(conn: &Connection, period_id: &str) -> Result<Vec<Entry>, DbError>;
}

impl EntryExt for Entry {
    fn post_to(&self, conn: &mut Connection, period_id: &str) -> Result<(), DbError> {
        // 1. Verify period exists and is open
        let period = PeriodRecord::find_by_id(conn, period_id)?;
        if period.status == "sealed" {
            return Err(DbError::PeriodSealed(period_id.to_string()));
        }

        // 2. Verify date is within period range
        if self.date < period.start_date || self.date > period.end_date {
            return Err(DbError::Core(
                sealed_books_core::CoreError::EntryOutOfPeriodRange {
                    entry_id: self.id.clone(),
                    entry_date: self.date.clone(),
                    period_start: period.start_date,
                    period_end: period.end_date,
                },
            ));
        }

        // 3. Verify double-entry accounting invariants
        self.validate()?;

        // 4. Atomic database insertion
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT INTO entries (id, period_id, date, description, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5);",
            params![
                &self.id,
                period_id,
                &self.date,
                &self.description,
                &self.created_at
            ],
        )?;

        for line in &self.lines {
            let direction_str = match line.direction {
                Direction::Debit => "debit",
                Direction::Credit => "credit",
            };

            tx.execute(
                "INSERT INTO lines (id, entry_id, account_id, direction, amount_minor, description)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
                params![
                    &line.id,
                    &self.id,
                    &line.account_id,
                    direction_str,
                    line.amount_minor as i64,
                    &line.description
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    fn find_by_id(conn: &Connection, id: &str) -> Result<Entry, DbError> {
        let (date, description, created_at) = conn
            .query_row(
                "SELECT date, description, created_at FROM entries WHERE id = ?1;",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => DbError::EntryNotFound(id.to_string()),
                other => DbError::Sqlite(other),
            })?;

        let mut line_stmt = conn.prepare(
            "SELECT id, account_id, direction, amount_minor, description
             FROM lines
             WHERE entry_id = ?1
             ORDER BY rowid ASC;",
        )?;

        let lines = line_stmt
            .query_map(params![id], |row| {
                let dir_str: String = row.get(2)?;
                let direction = match dir_str.as_str() {
                    "debit" => Direction::Debit,
                    "credit" => Direction::Credit,
                    _ => Direction::Debit,
                };
                let amount_minor: i64 = row.get(3)?;

                Ok(Line {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    direction,
                    amount_minor: amount_minor as u64,
                    description: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Entry {
            id: id.to_string(),
            date,
            description,
            created_at,
            lines,
        })
    }

    fn find_by_period(conn: &Connection, period_id: &str) -> Result<Vec<Entry>, DbError> {
        // Check period exists
        let _ = PeriodRecord::find_by_id(conn, period_id)?;

        // Fetch entries ordered by date, created_at, id
        let mut entry_stmt = conn.prepare(
            "SELECT id, date, description, created_at
             FROM entries
             WHERE period_id = ?1
             ORDER BY date ASC, created_at ASC, id ASC;",
        )?;

        let entry_rows = entry_stmt
            .query_map(params![period_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut entries = Vec::with_capacity(entry_rows.len());

        let mut line_stmt = conn.prepare(
            "SELECT id, account_id, direction, amount_minor, description
             FROM lines
             WHERE entry_id = ?1
             ORDER BY rowid ASC;",
        )?;

        for (id, date, description, created_at) in entry_rows {
            let lines = line_stmt
                .query_map(params![&id], |row| {
                    let dir_str: String = row.get(2)?;
                    let direction = match dir_str.as_str() {
                        "debit" => Direction::Debit,
                        "credit" => Direction::Credit,
                        _ => Direction::Debit,
                    };
                    let amount_minor: i64 = row.get(3)?;

                    Ok(Line {
                        id: row.get(0)?,
                        account_id: row.get(1)?,
                        direction,
                        amount_minor: amount_minor as u64,
                        description: row.get(4)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repository::account::Account;
    use crate::db::repository::period::PeriodRecord;
    use crate::db::schema::migrate;
    use sealed_books_core::hash::hash_entry;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();
        migrate(&conn).unwrap();
        conn
    }

    fn sample_period() -> PeriodRecord {
        PeriodRecord {
            id: "per_aug_2026".into(),
            entity: "Acme Trading Pvt Ltd".into(),
            start_date: "2026-08-01".into(),
            end_date: "2026-08-31".into(),
            status: "open".into(),
        }
    }

    fn seed_accounts(conn: &Connection) {
        Account {
            id: "acc_bank".into(),
            code: "1010".into(),
            name: "Bank Checking".into(),
            account_type: "asset".into(),
        }
        .insert(conn)
        .unwrap();

        Account {
            id: "acc_sales".into(),
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
        sample_period().insert(&conn).unwrap();

        let original_entry = Entry {
            id: "ent_001".into(),
            date: "2026-08-15".into(),
            description: "Product Sale Invoice #101".into(),
            created_at: "2026-08-15T10:00:00Z".into(),
            lines: vec![
                Line::new(
                    "l1",
                    "acc_bank",
                    Direction::Debit,
                    750000,
                    Some("Wire".into()),
                )
                .unwrap(),
                Line::new("l2", "acc_sales", Direction::Credit, 750000, None).unwrap(),
            ],
        };

        // 1. Calculate hash BEFORE saving to database
        let original_hash = hash_entry(&original_entry).expect("hash original");

        // 2. Save entry to SQLite using the EntryExt method directly!
        original_entry
            .post_to(&mut conn, "per_aug_2026")
            .expect("post entry");

        // 3. Reload entry from SQLite
        let reloaded_entries =
            <Entry as EntryExt>::find_by_period(&conn, "per_aug_2026").expect("get entries");
        assert_eq!(reloaded_entries.len(), 1);

        let reloaded_entry = &reloaded_entries[0];

        // 4. Calculate hash AFTER reloading from database
        let reloaded_hash = hash_entry(reloaded_entry).expect("hash reloaded");

        // Invariant check: The hashes must match byte-for-byte!
        assert_eq!(
            original_hash, reloaded_hash,
            "CRITICAL INVARIANT FAILED: DB reload must preserve exact canonical hash"
        );
    }

    #[test]
    fn test_find_by_id_success_and_not_found() {
        let mut conn = setup_test_db();
        seed_accounts(&conn);
        sample_period().insert(&conn).unwrap();

        let entry = Entry {
            id: "ent_find_me".into(),
            date: "2026-08-20".into(),
            description: "Target entry".into(),
            created_at: "2026-08-20T14:00:00Z".into(),
            lines: vec![
                Line::new("l1", "acc_bank", Direction::Debit, 5000, None).unwrap(),
                Line::new("l2", "acc_sales", Direction::Credit, 5000, None).unwrap(),
            ],
        };

        entry.post_to(&mut conn, "per_aug_2026").unwrap();

        // 1. Find existing
        let found = <Entry as EntryExt>::find_by_id(&conn, "ent_find_me").unwrap();
        assert_eq!(found.id, "ent_find_me");
        assert_eq!(found.description, "Target entry");
        assert_eq!(found.lines.len(), 2);

        // 2. Find nonexistent
        let err = <Entry as EntryExt>::find_by_id(&conn, "ent_not_exist").unwrap_err();
        match err {
            DbError::EntryNotFound(id) => assert_eq!(id, "ent_not_exist"),
            other => panic!("Expected EntryNotFound, got {other:?}"),
        }
    }

    #[test]
    fn test_post_unbalanced_entry_fails() {
        let mut conn = setup_test_db();
        seed_accounts(&conn);
        sample_period().insert(&conn).unwrap();

        let bad_entry = Entry {
            id: "ent_bad".into(),
            date: "2026-08-15".into(),
            description: "Unbalanced Entry".into(),
            created_at: "2026-08-15T10:00:00Z".into(),
            lines: vec![
                Line::new("l1", "acc_bank", Direction::Debit, 1000, None).unwrap(),
                Line::new("l2", "acc_sales", Direction::Credit, 999, None).unwrap(),
            ],
        };

        let err = bad_entry.post_to(&mut conn, "per_aug_2026").unwrap_err();
        match err {
            DbError::Core(sealed_books_core::CoreError::UnbalancedEntry { .. }) => {}
            other => panic!("Expected CoreError::UnbalancedEntry, got {other:?}"),
        }
    }

    #[test]
    fn test_post_to_sealed_period_rejected() {
        let mut conn = setup_test_db();
        seed_accounts(&conn);

        let mut period = sample_period();
        period.status = "sealed".into();
        period.insert(&conn).unwrap();

        let entry = Entry {
            id: "ent_late".into(),
            date: "2026-08-15".into(),
            description: "Late entry".into(),
            created_at: "2026-08-15T10:00:00Z".into(),
            lines: vec![
                Line::new("l1", "acc_bank", Direction::Debit, 1000, None).unwrap(),
                Line::new("l2", "acc_sales", Direction::Credit, 1000, None).unwrap(),
            ],
        };

        let err = entry.post_to(&mut conn, "per_aug_2026").unwrap_err();
        match err {
            DbError::PeriodSealed(id) => assert_eq!(id, "per_aug_2026"),
            other => panic!("Expected DbError::PeriodSealed, got {other:?}"),
        }
    }
}
