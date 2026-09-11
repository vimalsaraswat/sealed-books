//! SQLite DDL schemas, indexes, and migration runner for Sealed Books.

use rusqlite::Connection;

/// Runs schema creation migrations.
pub fn migrate(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        -- Chart of accounts
        CREATE TABLE IF NOT EXISTS accounts (
            id TEXT PRIMARY KEY,
            code TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            account_type TEXT NOT NULL CHECK(account_type IN ('asset', 'liability', 'equity', 'revenue', 'expense'))
        );

        -- Accounting periods
        CREATE TABLE IF NOT EXISTS periods (
            id TEXT PRIMARY KEY,
            entity TEXT NOT NULL,
            start_date TEXT NOT NULL,
            end_date TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'open' CHECK(status IN ('open', 'sealed'))
        );

        -- Balanced journal transactions
        CREATE TABLE IF NOT EXISTS entries (
            id TEXT PRIMARY KEY,
            period_id TEXT NOT NULL REFERENCES periods(id),
            date TEXT NOT NULL,
            description TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        -- Atomic debit/credit legs of a transaction
        CREATE TABLE IF NOT EXISTS lines (
            id TEXT PRIMARY KEY,
            entry_id TEXT NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
            account_id TEXT NOT NULL REFERENCES accounts(id),
            direction TEXT NOT NULL CHECK(direction IN ('debit', 'credit')),
            amount_minor INTEGER NOT NULL CHECK(amount_minor > 0),
            description TEXT
        );

        -- Published period close seals
        CREATE TABLE IF NOT EXISTS seals (
            id TEXT PRIMARY KEY,
            period_id TEXT NOT NULL UNIQUE REFERENCES periods(id),
            root TEXT NOT NULL,
            statement_hash TEXT NOT NULL,
            topic_id TEXT,
            sequence_number INTEGER,
            consensus_timestamp TEXT,
            approver_1_pubkey TEXT,
            approver_1_sig TEXT,
            approver_2_pubkey TEXT,
            approver_2_sig TEXT,
            created_at TEXT NOT NULL
        );

        -- Indexes for fast query performance
        CREATE INDEX IF NOT EXISTS idx_entries_period_id ON entries(period_id);
        CREATE INDEX IF NOT EXISTS idx_entries_date ON entries(date);
        CREATE INDEX IF NOT EXISTS idx_lines_entry_id ON lines(entry_id);
        CREATE INDEX IF NOT EXISTS idx_lines_account_id ON lines(account_id);
        ",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_creates_all_tables() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();
        migrate(&conn).unwrap();

        let tables: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;")
                .unwrap();
            let rows = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            rows
        };

        assert!(tables.contains(&"accounts".to_string()));
        assert!(tables.contains(&"periods".to_string()));
        assert!(tables.contains(&"entries".to_string()));
        assert!(tables.contains(&"lines".to_string()));
        assert!(tables.contains(&"seals".to_string()));
    }
}
