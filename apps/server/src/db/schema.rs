//! SQLite DDL schemas, indexes, and migration runner for Sealed Books.

use libsql::Connection;

/// Runs schema creation migrations.
pub async fn migrate(conn: &Connection) -> Result<(), libsql::Error> {
    conn.execute_batch(
        "
        -- Organizations (Tenant isolation boundary)
        CREATE TABLE IF NOT EXISTS organizations (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            base_currency TEXT NOT NULL DEFAULT 'USD',
            created_at TEXT NOT NULL
        );

        -- Global Users (1 identity across organizations)
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            pubkey TEXT NOT NULL,
            eth_address TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        -- Organization Memberships (User role within an organization)
        CREATE TABLE IF NOT EXISTS organization_memberships (
            id TEXT PRIMARY KEY,
            organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            role TEXT NOT NULL CHECK(role IN ('owner', 'admin', 'controller', 'auditor', 'staff')),
            status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'invited', 'suspended')),
            created_at TEXT NOT NULL,
            UNIQUE(organization_id, user_id)
        );

        -- User Sessions (Bearer token auth)
        CREATE TABLE IF NOT EXISTS sessions (
            token TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            active_organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            expires_at TEXT NOT NULL
        );

        -- Authentication OTPs (Passwordless Email + OTP)
        CREATE TABLE IF NOT EXISTS auth_otps (
            email TEXT PRIMARY KEY,
            code TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        -- Chart of accounts (Scoped to organization)
        CREATE TABLE IF NOT EXISTS accounts (
            id TEXT PRIMARY KEY,
            organization_id TEXT NOT NULL DEFAULT 'org_acme' REFERENCES organizations(id),
            code TEXT NOT NULL,
            name TEXT NOT NULL,
            account_type TEXT NOT NULL CHECK(account_type IN ('asset', 'liability', 'equity', 'revenue', 'expense')),
            UNIQUE(organization_id, code)
        );

        -- Accounting periods (Scoped to organization)
        CREATE TABLE IF NOT EXISTS periods (
            id TEXT PRIMARY KEY,
            organization_id TEXT NOT NULL DEFAULT 'org_acme' REFERENCES organizations(id),
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
            dispatch_status TEXT NOT NULL DEFAULT 'draft' CHECK(dispatch_status IN ('draft', 'pending_auditor', 'sealed', 'rejected')),
            auditor_id TEXT REFERENCES users(id),
            auditor_notes TEXT,
            created_at TEXT NOT NULL
        );

        -- Deterministic sealed leaf hashes for pinpointing tampered entries
        CREATE TABLE IF NOT EXISTS seal_leaves (
            period_id TEXT NOT NULL REFERENCES periods(id),
            entry_id TEXT NOT NULL,
            leaf_hash TEXT NOT NULL,
            PRIMARY KEY (period_id, entry_id)
        );
        ",
    )
    .await?;

    // Safe backwards-compatible column migrations for existing database files
    let _ = conn
        .execute(
            "ALTER TABLE accounts ADD COLUMN organization_id TEXT DEFAULT 'org_acme';",
            (),
        )
        .await;
    let _ = conn
        .execute(
            "ALTER TABLE periods ADD COLUMN organization_id TEXT DEFAULT 'org_acme';",
            (),
        )
        .await;
    let _ = conn
        .execute(
            "ALTER TABLE seals ADD COLUMN dispatch_status TEXT DEFAULT 'draft';",
            (),
        )
        .await;
    let _ = conn
        .execute("ALTER TABLE seals ADD COLUMN auditor_id TEXT;", ())
        .await;
    let _ = conn
        .execute("ALTER TABLE seals ADD COLUMN auditor_notes TEXT;", ())
        .await;

    // Apply indexes after table definitions and column migrations
    conn.execute_batch(
        "
        CREATE INDEX IF NOT EXISTS idx_memberships_user ON organization_memberships(user_id);
        CREATE INDEX IF NOT EXISTS idx_memberships_org ON organization_memberships(organization_id);
        CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
        CREATE INDEX IF NOT EXISTS idx_accounts_org ON accounts(organization_id);
        CREATE INDEX IF NOT EXISTS idx_periods_org ON periods(organization_id);
        CREATE INDEX IF NOT EXISTS idx_entries_period_id ON entries(period_id);
        CREATE INDEX IF NOT EXISTS idx_entries_date ON entries(date);
        CREATE INDEX IF NOT EXISTS idx_lines_entry_id ON lines(entry_id);
        CREATE INDEX IF NOT EXISTS idx_lines_account_id ON lines(account_id);
        CREATE INDEX IF NOT EXISTS idx_seal_leaves_period_id ON seal_leaves(period_id);
        ",
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migration_creates_all_tables() {
        let db = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .unwrap();
        let conn = db.connect().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", ()).await.unwrap();
        migrate(&conn).await.unwrap();

        let mut rows = conn
            .query(
                "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;",
                (),
            )
            .await
            .unwrap();

        let mut tables = Vec::new();
        while let Some(row) = rows.next().await.unwrap() {
            let name: String = row.get(0).unwrap();
            tables.push(name);
        }

        assert!(tables.contains(&"organizations".to_string()));
        assert!(tables.contains(&"users".to_string()));
        assert!(tables.contains(&"organization_memberships".to_string()));
        assert!(tables.contains(&"sessions".to_string()));
        assert!(tables.contains(&"auth_otps".to_string()));
        assert!(tables.contains(&"accounts".to_string()));
        assert!(tables.contains(&"periods".to_string()));
        assert!(tables.contains(&"entries".to_string()));
        assert!(tables.contains(&"lines".to_string()));
        assert!(tables.contains(&"seals".to_string()));
        assert!(tables.contains(&"seal_leaves".to_string()));
    }
}
