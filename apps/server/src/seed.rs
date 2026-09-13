//! Realistic demo ledger data seeder for Sealed Books.
//!
//! Seeds a realistic 1-month ledger for "Acme Trading Pvt Ltd" covering August 2026.
//! Contains 13 accounts and 16 balanced double-entry transactions (all in integer minor units).

use crate::db::error::DbError;
use crate::db::repository::account::Account;
use crate::db::repository::entry::EntryExt;
use crate::db::repository::membership::Membership;
use crate::db::repository::organization::Organization;
use crate::db::repository::period::PeriodRecord;
use crate::db::repository::session::Session;
use crate::db::repository::user::User;
use libsql::Connection;
use sealed_books_core::types::{Direction, Entry, Line};

pub const DEMO_ORG_ID: &str = "org_acme";
pub const DEMO_ORG_NAME: &str = "Acme Trading Pvt Ltd";
pub const DEMO_PERIOD_ID: &str = "per_2026_08";
pub const DEMO_ENTITY_NAME: &str = "Acme Trading Pvt Ltd";
pub const DEMO_START_DATE: &str = "2026-08-01";
pub const DEMO_END_DATE: &str = "2026-08-31";

/// Seeds demo accounts, period, and entries if not already present.
///
/// Returns `Ok(true)` if seeding took place, or `Ok(false)` if data already existed.
pub async fn seed_if_empty(conn: &Connection) -> Result<bool, DbError> {
    seed_auth_tenancy(conn).await?;

    if PeriodRecord::find_by_id(conn, DEMO_PERIOD_ID).await.is_ok() {
        return Ok(false);
    }

    seed_demo_data(conn).await?;
    Ok(true)
}

/// Seeds all chart of accounts, the August 2026 accounting period, and 16 balanced transactions.
/// Seeds organizations, users, memberships, and test sessions.
pub async fn seed_auth_tenancy(conn: &Connection) -> Result<(), DbError> {
    let org1 = Organization {
        id: DEMO_ORG_ID.into(),
        name: DEMO_ORG_NAME.into(),
        base_currency: "USD".into(),
        created_at: "2026-08-01T00:00:00Z".into(),
    };
    let org2 = Organization {
        id: "org_deloitte".into(),
        name: "Deloitte Advisory LLP".into(),
        base_currency: "USD".into(),
        created_at: "2026-08-01T00:00:00Z".into(),
    };
    let _ = org1.insert(conn).await;
    let _ = org2.insert(conn).await;

    let approver_1_wid = std::env::var("APPROVER_1_WALLET_ID")
        .unwrap_or_else(|_| "pugx9v735lhrtn26245f8glf".to_string());
    let approver_1_addr = std::env::var("APPROVER_1_ADDRESS")
        .unwrap_or_else(|_| "0x7bfb3F9E7316377A263e06200d9823E3E7a862b2".to_string());
    let approver_1_pk = std::env::var("APPROVER_1_PUBKEY").unwrap_or_else(|_| {
        "0x02e923a5f6d0f2378d4717731a56595bb4419bd0ba1939f98336c0d1e5074a250f".to_string()
    });

    let approver_2_wid = std::env::var("APPROVER_2_WALLET_ID")
        .unwrap_or_else(|_| "a4xct73etu19t7zmxlw8zy5p".to_string());
    let approver_2_addr = std::env::var("APPROVER_2_ADDRESS")
        .unwrap_or_else(|_| "0x6e5b360D42B77821E92CC3B38B605CE8C0e98A7D".to_string());
    let approver_2_pk = std::env::var("APPROVER_2_PUBKEY").unwrap_or_else(|_| {
        "0x0317211fe7d675008e28ff3c39941151c2017c0934ef85d4eda48ca38325619131".to_string()
    });

    let approver_3_wid = std::env::var("APPROVER_3_WALLET_ID")
        .unwrap_or_else(|_| "bnacv568tvwz2ootvvv6irvt".to_string());
    let approver_3_addr = std::env::var("APPROVER_3_ADDRESS")
        .unwrap_or_else(|_| "0x8d25F11A383449fBeD5D395f5e8f8E925c4Bd4fa".to_string());
    let approver_3_pk = std::env::var("APPROVER_3_PUBKEY").unwrap_or_else(|_| {
        "0x03e2caec58fb19f16d882dd9ba70e13607483a9977159c18f845ef0d81b88de208".to_string()
    });

    let users = vec![
        User {
            id: "usr_alice".into(),
            email: "alice@acmetrading.com".into(),
            name: "Alice Chen".into(),
            pubkey: approver_1_pk,
            eth_address: approver_1_addr,
            wallet_id: Some(approver_1_wid),
            created_at: "2026-08-01T00:00:00Z".into(),
        },
        User {
            id: "usr_bob".into(),
            email: "bstone@auditfirm.com".into(),
            name: "Bob Smith".into(),
            pubkey: approver_2_pk,
            eth_address: approver_2_addr,
            wallet_id: Some(approver_2_wid),
            created_at: "2026-08-01T00:00:00Z".into(),
        },
        User {
            id: "usr_charlie".into(),
            email: "charlie@acmetrading.com".into(),
            name: "Charlie Davis".into(),
            pubkey: "023c72addb4fdf09af94f0c94d7fe92a386a7e70cf8a1d85916386bb2535c7b1b1".into(),
            eth_address: "0x5cbdd86a2fa8dc4bddd8a8f69dba48572eec07fb".into(),
            wallet_id: None,
            created_at: "2026-08-01T00:00:00Z".into(),
        },
        User {
            id: "usr_diana".into(),
            email: "diana@acmetrading.com".into(),
            name: "Carol Vance".into(),
            pubkey: approver_3_pk,
            eth_address: approver_3_addr,
            wallet_id: Some(approver_3_wid),
            created_at: "2026-08-01T00:00:00Z".into(),
        },
    ];
    for u in users {
        let _ = u.insert(conn).await;
    }

    let memberships = vec![
        Membership {
            id: "mem_acme_alice".into(),
            organization_id: DEMO_ORG_ID.into(),
            user_id: "usr_alice".into(),
            role: "controller".into(),
            status: "active".into(),
            created_at: "2026-08-01T00:00:00Z".into(),
        },
        Membership {
            id: "mem_acme_bob".into(),
            organization_id: DEMO_ORG_ID.into(),
            user_id: "usr_bob".into(),
            role: "auditor".into(),
            status: "active".into(),
            created_at: "2026-08-01T00:00:00Z".into(),
        },
        Membership {
            id: "mem_deloitte_bob".into(),
            organization_id: "org_deloitte".into(),
            user_id: "usr_bob".into(),
            role: "owner".into(),
            status: "active".into(),
            created_at: "2026-08-01T00:00:00Z".into(),
        },
        Membership {
            id: "mem_acme_charlie".into(),
            organization_id: DEMO_ORG_ID.into(),
            user_id: "usr_charlie".into(),
            role: "staff".into(),
            status: "active".into(),
            created_at: "2026-08-01T00:00:00Z".into(),
        },
        Membership {
            id: "mem_acme_diana".into(),
            organization_id: DEMO_ORG_ID.into(),
            user_id: "usr_diana".into(),
            role: "owner".into(),
            status: "active".into(),
            created_at: "2026-08-01T00:00:00Z".into(),
        },
    ];
    for m in memberships {
        let _ = m.insert(conn).await;
    }

    let sessions = vec![
        Session {
            token: "token_alice".into(),
            user_id: "usr_alice".into(),
            active_organization_id: DEMO_ORG_ID.into(),
            expires_at: "2099-01-01T00:00:00Z".into(),
        },
        Session {
            token: "token_bob".into(),
            user_id: "usr_bob".into(),
            active_organization_id: DEMO_ORG_ID.into(),
            expires_at: "2099-01-01T00:00:00Z".into(),
        },
        Session {
            token: "token_charlie".into(),
            user_id: "usr_charlie".into(),
            active_organization_id: DEMO_ORG_ID.into(),
            expires_at: "2099-01-01T00:00:00Z".into(),
        },
        Session {
            token: "token_diana".into(),
            user_id: "usr_diana".into(),
            active_organization_id: DEMO_ORG_ID.into(),
            expires_at: "2099-01-01T00:00:00Z".into(),
        },
    ];
    for s in sessions {
        let _ = s.insert(conn).await;
    }

    Ok(())
}

/// Seeds all chart of accounts, the August 2026 accounting period, and 16 balanced transactions.
pub async fn seed_demo_data(conn: &Connection) -> Result<(), DbError> {
    seed_auth_tenancy(conn).await?;

    // 1. Chart of Accounts
    let accounts = vec![
        Account {
            organization_id: DEMO_ORG_ID.into(),
            id: "acc_1010".into(),
            code: "1010".into(),
            name: "HDFC Current Account".into(),
            account_type: "asset".into(),
        },
        Account {
            organization_id: DEMO_ORG_ID.into(),
            id: "acc_1020".into(),
            code: "1020".into(),
            name: "Accounts Receivable".into(),
            account_type: "asset".into(),
        },
        Account {
            organization_id: DEMO_ORG_ID.into(),
            id: "acc_1030".into(),
            code: "1030".into(),
            name: "Merchandise Inventory".into(),
            account_type: "asset".into(),
        },
        Account {
            organization_id: DEMO_ORG_ID.into(),
            id: "acc_1040".into(),
            code: "1040".into(),
            name: "Office Security Deposit".into(),
            account_type: "asset".into(),
        },
        Account {
            organization_id: DEMO_ORG_ID.into(),
            id: "acc_2010".into(),
            code: "2010".into(),
            name: "Accounts Payable - Suppliers".into(),
            account_type: "liability".into(),
        },
        Account {
            organization_id: DEMO_ORG_ID.into(),
            id: "acc_2020".into(),
            code: "2020".into(),
            name: "Sales Tax & Duties Payable".into(),
            account_type: "liability".into(),
        },
        Account {
            organization_id: DEMO_ORG_ID.into(),
            id: "acc_3010".into(),
            code: "3010".into(),
            name: "Share Capital".into(),
            account_type: "equity".into(),
        },
        Account {
            organization_id: DEMO_ORG_ID.into(),
            id: "acc_4010".into(),
            code: "4010".into(),
            name: "Wholesale Sales Revenue".into(),
            account_type: "revenue".into(),
        },
        Account {
            organization_id: DEMO_ORG_ID.into(),
            id: "acc_5010".into(),
            code: "5010".into(),
            name: "Cost of Goods Sold".into(),
            account_type: "expense".into(),
        },
        Account {
            organization_id: DEMO_ORG_ID.into(),
            id: "acc_5020".into(),
            code: "5020".into(),
            name: "Office & Warehouse Rent".into(),
            account_type: "expense".into(),
        },
        Account {
            organization_id: DEMO_ORG_ID.into(),
            id: "acc_5030".into(),
            code: "5030".into(),
            name: "Employee Salaries".into(),
            account_type: "expense".into(),
        },
        Account {
            organization_id: DEMO_ORG_ID.into(),
            id: "acc_5040".into(),
            code: "5040".into(),
            name: "Cloud Hosting & SaaS Subscriptions".into(),
            account_type: "expense".into(),
        },
        Account {
            organization_id: DEMO_ORG_ID.into(),
            id: "acc_5050".into(),
            code: "5050".into(),
            name: "Bank Charges & Wire Fees".into(),
            account_type: "expense".into(),
        },
    ];

    for account in accounts {
        // Insert account if it doesn't already exist
        if Account::find_by_id(conn, &account.id).await.is_err() {
            account.insert(conn).await?;
        }
    }

    // 2. Accounting Period
    let period = PeriodRecord {
        id: DEMO_PERIOD_ID.into(),
        organization_id: DEMO_ORG_ID.into(),
        entity: DEMO_ENTITY_NAME.into(),
        start_date: DEMO_START_DATE.into(),
        end_date: DEMO_END_DATE.into(),
        status: "open".into(),
    };
    period.insert(conn).await?;

    // 3. Balanced Journal Entries
    let entries = get_demo_entries()?;

    for entry in entries {
        entry.post_to(conn, DEMO_PERIOD_ID).await?;
    }

    Ok(())
}

/// Constructs the list of 16 balanced double-entry transactions for August 2026.
#[allow(clippy::vec_init_then_push)]
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

    async fn setup_test_db() -> Connection {
        let db = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .unwrap();
        let conn = db.connect().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", ()).await.unwrap();
        migrate(&conn).await.unwrap();
        conn
    }

    #[tokio::test]
    async fn test_seed_demo_data_succeeds_and_is_idempotent() {
        let conn = setup_test_db().await;

        // 1. Initial seed should succeed
        let seeded = seed_if_empty(&conn).await.expect("seed demo data");
        assert!(seeded, "First seed must report true");

        // Verify period exists
        let period = PeriodRecord::find_by_id(&conn, DEMO_PERIOD_ID)
            .await
            .expect("period exists");
        assert_eq!(period.entity, DEMO_ENTITY_NAME);
        assert_eq!(period.status, "open");

        // Verify accounts exist
        let accounts = Account::list_all(&conn).await.expect("list accounts");
        assert_eq!(accounts.len(), 13);

        // Verify entries exist and count is 16
        let entries = <Entry as EntryExt>::find_by_period(&conn, DEMO_PERIOD_ID)
            .await
            .expect("get entries");
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
        let seeded_again = seed_if_empty(&conn).await.expect("second seed check");
        assert!(
            !seeded_again,
            "Second seed must report false (already seeded)"
        );

        // Entry count must still be 16
        let entries_after = <Entry as EntryExt>::find_by_period(&conn, DEMO_PERIOD_ID)
            .await
            .expect("get entries");
        assert_eq!(entries_after.len(), 16);
    }

    #[tokio::test]
    async fn test_seeded_data_generates_valid_merkle_root_and_statement() {
        let conn = setup_test_db().await;
        seed_if_empty(&conn).await.expect("seed demo data");

        let period_rec = PeriodRecord::find_by_id(&conn, DEMO_PERIOD_ID)
            .await
            .unwrap();
        let core_period = period_rec.to_core();

        let entries = <Entry as EntryExt>::find_by_period(&conn, DEMO_PERIOD_ID)
            .await
            .unwrap();
        assert_eq!(entries.len(), 16);

        // Build statement using crates/core
        let statement = build_statement(&core_period, &entries).expect("build statement");

        assert_eq!(statement.entity, DEMO_ENTITY_NAME);
        assert_eq!(statement.period_start, DEMO_START_DATE);
        assert_eq!(statement.period_end, DEMO_END_DATE);
        assert_eq!(statement.entry_count, 16);
        assert_eq!(statement.total_debits_minor, 12185000);
        assert_eq!(statement.total_credits_minor, 12185000);

        // Calculate statement prehash
        let hash = statement_hash(&statement);
        assert_ne!(hash, [0u8; 32]);
    }
}
