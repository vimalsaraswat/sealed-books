//! Realistic demo ledger data seeder for Sealed Books.
//!
//! Seeds a realistic 1-month ledger for "Acme Trading Pvt Ltd" covering August 2026.
//! Contains 13 accounts and 16 balanced double-entry transactions (all in integer minor units).

use crate::db::error::DbError;
use crate::db::repository::account::Account;
use crate::db::repository::entry::EntryExt;
use crate::db::repository::period::PeriodRecord;
use rusqlite::Connection;
use sealed_books_core::types::{Direction, Entry, Line};

pub const DEMO_PERIOD_ID: &str = "per_2026_08";
pub const DEMO_ENTITY_NAME: &str = "Acme Trading Pvt Ltd";
pub const DEMO_START_DATE: &str = "2026-08-01";
pub const DEMO_END_DATE: &str = "2026-08-31";

/// Seeds demo accounts, period, and entries if not already present.
///
/// Returns `Ok(true)` if seeding took place, or `Ok(false)` if data already existed.
pub fn seed_if_empty(conn: &mut Connection) -> Result<bool, DbError> {
    if PeriodRecord::find_by_id(conn, DEMO_PERIOD_ID).is_ok() {
        return Ok(false);
    }

    seed_demo_data(conn)?;
    Ok(true)
}

/// Seeds all chart of accounts, the August 2026 accounting period, and 16 balanced transactions.
pub fn seed_demo_data(conn: &mut Connection) -> Result<(), DbError> {
    // 1. Chart of Accounts
    let accounts = vec![
        Account {
            id: "acc_1010".into(),
            code: "1010".into(),
            name: "HDFC Current Account".into(),
            account_type: "asset".into(),
        },
        Account {
            id: "acc_1020".into(),
            code: "1020".into(),
            name: "Accounts Receivable".into(),
            account_type: "asset".into(),
        },
        Account {
            id: "acc_1030".into(),
            code: "1030".into(),
            name: "Merchandise Inventory".into(),
            account_type: "asset".into(),
        },
        Account {
            id: "acc_1040".into(),
            code: "1040".into(),
            name: "Office Security Deposit".into(),
            account_type: "asset".into(),
        },
        Account {
            id: "acc_2010".into(),
            code: "2010".into(),
            name: "Accounts Payable - Suppliers".into(),
            account_type: "liability".into(),
        },
        Account {
            id: "acc_2020".into(),
            code: "2020".into(),
            name: "Sales Tax & Duties Payable".into(),
            account_type: "liability".into(),
        },
        Account {
            id: "acc_3010".into(),
            code: "3010".into(),
            name: "Share Capital".into(),
            account_type: "equity".into(),
        },
        Account {
            id: "acc_4010".into(),
            code: "4010".into(),
            name: "Wholesale Sales Revenue".into(),
            account_type: "revenue".into(),
        },
        Account {
            id: "acc_5010".into(),
            code: "5010".into(),
            name: "Cost of Goods Sold".into(),
            account_type: "expense".into(),
        },
        Account {
            id: "acc_5020".into(),
            code: "5020".into(),
            name: "Office & Warehouse Rent".into(),
            account_type: "expense".into(),
        },
        Account {
            id: "acc_5030".into(),
            code: "5030".into(),
            name: "Employee Salaries".into(),
            account_type: "expense".into(),
        },
        Account {
            id: "acc_5040".into(),
            code: "5040".into(),
            name: "Cloud Hosting & SaaS Subscriptions".into(),
            account_type: "expense".into(),
        },
        Account {
            id: "acc_5050".into(),
            code: "5050".into(),
            name: "Bank Charges & Wire Fees".into(),
            account_type: "expense".into(),
        },
    ];

    for account in accounts {
        // Insert account if it doesn't already exist
        if Account::find_by_id(conn, &account.id).is_err() {
            account.insert(conn)?;
        }
    }

    // 2. Accounting Period
    let period = PeriodRecord {
        id: DEMO_PERIOD_ID.into(),
        entity: DEMO_ENTITY_NAME.into(),
        start_date: DEMO_START_DATE.into(),
        end_date: DEMO_END_DATE.into(),
        status: "open".into(),
    };
    period.insert(conn)?;

    // 3. Balanced Journal Entries
    let entries = get_demo_entries()?;

    for entry in entries {
        entry.post_to(conn, DEMO_PERIOD_ID)?;
    }

    Ok(())
}

/// Constructs the list of 16 balanced double-entry transactions for August 2026.
pub fn get_demo_entries() -> Result<Vec<Entry>, DbError> {
    let mut entries = Vec::with_capacity(16);

    // 01: Initial equity capital infusion ($50,000.00)
    entries.push(Entry {
        id: "ent_2026_08_001".into(),
        date: "2026-08-01".into(),
        description: "Initial equity capital infusion from founders".into(),
        created_at: "2026-08-01T09:00:00Z".into(),
        lines: vec![
            Line::new(
                "l_001_d",
                "acc_1010",
                Direction::Debit,
                5000000,
                Some("Wire transfer from promoters".into()),
            )?,
            Line::new(
                "l_001_c",
                "acc_3010",
                Direction::Credit,
                5000000,
                Some("Common stock issued".into()),
            )?,
        ],
    });

    // 02: Commercial warehouse lease security deposit ($3,000.00)
    entries.push(Entry {
        id: "ent_2026_08_002".into(),
        date: "2026-08-02".into(),
        description: "Commercial warehouse lease security deposit".into(),
        created_at: "2026-08-02T10:30:00Z".into(),
        lines: vec![
            Line::new(
                "l_002_d",
                "acc_1040",
                Direction::Debit,
                300000,
                Some("Refundable deposit".into()),
            )?,
            Line::new(
                "l_002_c",
                "acc_1010",
                Direction::Credit,
                300000,
                Some("HDFC wire payment".into()),
            )?,
        ],
    });

    // 03: Inventory procurement from Apex Electronics ($12,000.00)
    entries.push(Entry {
        id: "ent_2026_08_003".into(),
        date: "2026-08-03".into(),
        description: "Inventory procurement from Apex Electronics - PO #881".into(),
        created_at: "2026-08-03T11:15:00Z".into(),
        lines: vec![
            Line::new(
                "l_003_d",
                "acc_1030",
                Direction::Debit,
                1200000,
                Some("Electronics batch #881".into()),
            )?,
            Line::new(
                "l_003_c",
                "acc_2010",
                Direction::Credit,
                1200000,
                Some("Supplier invoice 30-day net".into()),
            )?,
        ],
    });

    // 04: Monthly warehouse lease payment ($1,500.00)
    entries.push(Entry {
        id: "ent_2026_08_05".into(),
        date: "2026-08-05".into(),
        description: "Monthly warehouse lease payment for August 2026".into(),
        created_at: "2026-08-05T09:45:00Z".into(),
        lines: vec![
            Line::new(
                "l_004_d",
                "acc_5020",
                Direction::Debit,
                150000,
                Some("August rent".into()),
            )?,
            Line::new(
                "l_004_c",
                "acc_1010",
                Direction::Credit,
                150000,
                Some("ACH auto-debit".into()),
            )?,
        ],
    });

    // 05: Wholesale shipment to Metro Retailers ($8,500.00)
    entries.push(Entry {
        id: "ent_2026_08_008a".into(),
        date: "2026-08-08".into(),
        description: "Wholesale shipment to Metro Retailers - Invoice #INV-1001".into(),
        created_at: "2026-08-08T14:20:00Z".into(),
        lines: vec![
            Line::new(
                "l_005_d",
                "acc_1020",
                Direction::Debit,
                850000,
                Some("Billed on 14-day credit".into()),
            )?,
            Line::new(
                "l_005_c",
                "acc_4010",
                Direction::Credit,
                850000,
                Some("Wholesale sale revenue".into()),
            )?,
        ],
    });

    // 06: Cost of goods sold recognition for INV-1001 ($5,500.00)
    entries.push(Entry {
        id: "ent_2026_08_008b".into(),
        date: "2026-08-08".into(),
        description: "Cost of goods sold recognition for INV-1001".into(),
        created_at: "2026-08-08T14:25:00Z".into(),
        lines: vec![
            Line::new(
                "l_006_d",
                "acc_5010",
                Direction::Debit,
                550000,
                Some("Inventory relief at cost".into()),
            )?,
            Line::new(
                "l_006_c",
                "acc_1030",
                Direction::Credit,
                550000,
                Some("Merchandise inventory reduction".into()),
            )?,
        ],
    });

    // 07: Wholesale shipment to Nexus Systems ($6,200.00)
    entries.push(Entry {
        id: "ent_2026_08_012a".into(),
        date: "2026-08-12".into(),
        description: "Wholesale shipment to Nexus Systems - Invoice #INV-1002".into(),
        created_at: "2026-08-12T13:00:00Z".into(),
        lines: vec![
            Line::new(
                "l_007_d",
                "acc_1020",
                Direction::Debit,
                620000,
                Some("Billed on 14-day credit".into()),
            )?,
            Line::new(
                "l_007_c",
                "acc_4010",
                Direction::Credit,
                620000,
                Some("Wholesale sale revenue".into()),
            )?,
        ],
    });

    // 08: Cost of goods sold recognition for INV-1002 ($4,000.00)
    entries.push(Entry {
        id: "ent_2026_08_012b".into(),
        date: "2026-08-12".into(),
        description: "Cost of goods sold recognition for INV-1002".into(),
        created_at: "2026-08-12T13:05:00Z".into(),
        lines: vec![
            Line::new(
                "l_008_d",
                "acc_5010",
                Direction::Debit,
                400000,
                Some("Inventory relief at cost".into()),
            )?,
            Line::new(
                "l_008_c",
                "acc_1030",
                Direction::Credit,
                400000,
                Some("Merchandise inventory reduction".into()),
            )?,
        ],
    });

    // 09: Partial wire payment to Apex Electronics on PO #881 ($6,000.00)
    entries.push(Entry {
        id: "ent_2026_08_015".into(),
        date: "2026-08-15".into(),
        description: "Partial wire payment to Apex Electronics on PO #881".into(),
        created_at: "2026-08-15T10:00:00Z".into(),
        lines: vec![
            Line::new(
                "l_009_d",
                "acc_2010",
                Direction::Debit,
                600000,
                Some("Accounts payable reduction".into()),
            )?,
            Line::new(
                "l_009_c",
                "acc_1010",
                Direction::Credit,
                600000,
                Some("HDFC outward wire #90812".into()),
            )?,
        ],
    });

    // 10: Customer wire receipt from Metro Retailers ($8,500.00)
    entries.push(Entry {
        id: "ent_2026_08_018".into(),
        date: "2026-08-18".into(),
        description: "Customer wire receipt from Metro Retailers for INV-1001".into(),
        created_at: "2026-08-18T16:30:00Z".into(),
        lines: vec![
            Line::new(
                "l_010_d",
                "acc_1010",
                Direction::Debit,
                850000,
                Some("HDFC inward wire settlement".into()),
            )?,
            Line::new(
                "l_010_c",
                "acc_1020",
                Direction::Credit,
                850000,
                Some("Clear receivables INV-1001".into()),
            )?,
        ],
    });

    // 11: Cloud hosting and SaaS subscriptions ($425.00)
    entries.push(Entry {
        id: "ent_2026_08_022".into(),
        date: "2026-08-22".into(),
        description: "AWS cloud infrastructure and SaaS subscriptions".into(),
        created_at: "2026-08-22T08:15:00Z".into(),
        lines: vec![
            Line::new(
                "l_011_d",
                "acc_5040",
                Direction::Debit,
                42500,
                Some("Monthly AWS + Quickbooks".into()),
            )?,
            Line::new(
                "l_011_c",
                "acc_1010",
                Direction::Credit,
                42500,
                Some("Debit card charge".into()),
            )?,
        ],
    });

    // 12: Customer wire receipt from Nexus Systems ($6,200.00)
    entries.push(Entry {
        id: "ent_2026_08_025".into(),
        date: "2026-08-25".into(),
        description: "Customer wire receipt from Nexus Systems for INV-1002".into(),
        created_at: "2026-08-25T11:45:00Z".into(),
        lines: vec![
            Line::new(
                "l_012_d",
                "acc_1010",
                Direction::Debit,
                620000,
                Some("HDFC inward wire settlement".into()),
            )?,
            Line::new(
                "l_012_c",
                "acc_1020",
                Direction::Credit,
                620000,
                Some("Clear receivables INV-1002".into()),
            )?,
        ],
    });

    // 13: Direct counter sale to QuickStore Retail ($3,400.00)
    entries.push(Entry {
        id: "ent_2026_08_028a".into(),
        date: "2026-08-28".into(),
        description: "Direct counter sale to QuickStore Retail".into(),
        created_at: "2026-08-28T15:10:00Z".into(),
        lines: vec![
            Line::new(
                "l_013_d",
                "acc_1010",
                Direction::Debit,
                340000,
                Some("Instant UPI/Wire payment".into()),
            )?,
            Line::new(
                "l_013_c",
                "acc_4010",
                Direction::Credit,
                340000,
                Some("Sales revenue direct".into()),
            )?,
        ],
    });

    // 14: Cost of goods sold recognition for QuickStore sale ($2,100.00)
    entries.push(Entry {
        id: "ent_2026_08_028b".into(),
        date: "2026-08-28".into(),
        description: "Cost of goods sold recognition for QuickStore sale".into(),
        created_at: "2026-08-28T15:12:00Z".into(),
        lines: vec![
            Line::new(
                "l_014_d",
                "acc_5010",
                Direction::Debit,
                210000,
                Some("Inventory relief at cost".into()),
            )?,
            Line::new(
                "l_014_c",
                "acc_1030",
                Direction::Credit,
                210000,
                Some("Merchandise inventory reduction".into()),
            )?,
        ],
    });

    // 15: Staff payroll disbursements for August 2026 ($4,500.00)
    entries.push(Entry {
        id: "ent_2026_08_030".into(),
        date: "2026-08-30".into(),
        description: "Staff payroll disbursements for August 2026".into(),
        created_at: "2026-08-30T17:00:00Z".into(),
        lines: vec![
            Line::new(
                "l_015_d",
                "acc_5030",
                Direction::Debit,
                450000,
                Some("Monthly staff salaries".into()),
            )?,
            Line::new(
                "l_015_c",
                "acc_1010",
                Direction::Credit,
                450000,
                Some("Direct bank batch transfer".into()),
            )?,
        ],
    });

    // 16: Monthly banking maintenance and wire transfer fees ($25.00)
    entries.push(Entry {
        id: "ent_2026_08_031".into(),
        date: "2026-08-31".into(),
        description: "Monthly banking maintenance and wire transfer fees".into(),
        created_at: "2026-08-31T23:30:00Z".into(),
        lines: vec![
            Line::new(
                "l_016_d",
                "acc_5050",
                Direction::Debit,
                2500,
                Some("Bank charges deducted".into()),
            )?,
            Line::new(
                "l_016_c",
                "acc_1010",
                Direction::Credit,
                2500,
                Some("HDFC monthly fee debit".into()),
            )?,
        ],
    });

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::migrate;
    use sealed_books_core::hash::statement_hash;
    use sealed_books_core::merkle::build_statement;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();
        migrate(&conn).unwrap();
        conn
    }

    #[test]
    fn test_seed_demo_data_succeeds_and_is_idempotent() {
        let mut conn = setup_test_db();

        // 1. Initial seed should succeed
        let seeded = seed_if_empty(&mut conn).expect("seed demo data");
        assert!(seeded, "First seed must report true");

        // Verify period exists
        let period = PeriodRecord::find_by_id(&conn, DEMO_PERIOD_ID).expect("period exists");
        assert_eq!(period.entity, DEMO_ENTITY_NAME);
        assert_eq!(period.status, "open");

        // Verify accounts exist
        let accounts = Account::list_all(&conn).expect("list accounts");
        assert_eq!(accounts.len(), 13);

        // Verify entries exist and count is 16
        let entries =
            <Entry as EntryExt>::find_by_period(&conn, DEMO_PERIOD_ID).expect("get entries");
        assert_eq!(entries.len(), 16);

        // Verify total debits and credits
        let total_debits: u64 = entries
            .iter()
            .flat_map(|e| e.lines.iter())
            .filter(|l| l.direction == Direction::Debit)
            .map(|l| l.amount_minor)
            .sum();

        let total_credits: u64 = entries
            .iter()
            .flat_map(|e| e.lines.iter())
            .filter(|l| l.direction == Direction::Credit)
            .map(|l| l.amount_minor)
            .sum();

        assert_eq!(total_debits, 12185000);
        assert_eq!(total_credits, 12185000);

        // 2. Second seed invocation should be a no-op (idempotent)
        let seeded_again = seed_if_empty(&mut conn).expect("second seed check");
        assert!(
            !seeded_again,
            "Second seed must report false (already seeded)"
        );

        // Entry count must still be 16
        let entries_after =
            <Entry as EntryExt>::find_by_period(&conn, DEMO_PERIOD_ID).expect("get entries");
        assert_eq!(entries_after.len(), 16);
    }

    #[test]
    fn test_seeded_data_generates_valid_merkle_root_and_statement() {
        let mut conn = setup_test_db();
        seed_if_empty(&mut conn).expect("seed demo data");

        let period_rec = PeriodRecord::find_by_id(&conn, DEMO_PERIOD_ID).unwrap();
        let core_period = period_rec.to_core();

        let entries = <Entry as EntryExt>::find_by_period(&conn, DEMO_PERIOD_ID).unwrap();
        assert_eq!(entries.len(), 16);

        // Build statement using crates/core
        let statement = build_statement(&core_period, &entries).expect("build statement");

        assert_eq!(statement.entity, DEMO_ENTITY_NAME);
        assert_eq!(statement.period_start, DEMO_START_DATE);
        assert_eq!(statement.period_end, DEMO_END_DATE);
        assert_eq!(statement.entry_count, 16);
        assert_eq!(statement.total_debits_minor, 12185000);
        assert_eq!(statement.total_credits_minor, 12185000);

        // Calculate statement prehash (for Privy signing in Phase 3)
        let hash = statement_hash(&statement);
        assert_ne!(hash, [0u8; 32]);
    }
}
