//! On-chain seal audit records and queries.

use crate::db::error::DbError;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealRecord {
    pub id: String,
    pub period_id: String,
    pub root: String,
    pub statement_hash: String,
    pub topic_id: Option<String>,
    pub sequence_number: Option<i64>,
    pub consensus_timestamp: Option<String>,
    pub approver_1_pubkey: Option<String>,
    pub approver_1_sig: Option<String>,
    pub approver_2_pubkey: Option<String>,
    pub approver_2_sig: Option<String>,
    pub created_at: String,
}

impl SealRecord {
    /// Inserts this seal record into the database.
    pub fn insert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO seals (
                id, period_id, root, statement_hash, topic_id, sequence_number,
                consensus_timestamp, approver_1_pubkey, approver_1_sig,
                approver_2_pubkey, approver_2_sig, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12);",
            params![
                &self.id,
                &self.period_id,
                &self.root,
                &self.statement_hash,
                &self.topic_id,
                &self.sequence_number,
                &self.consensus_timestamp,
                &self.approver_1_pubkey,
                &self.approver_1_sig,
                &self.approver_2_pubkey,
                &self.approver_2_sig,
                &self.created_at
            ],
        )?;
        Ok(())
    }

    /// Inserts or updates this seal record (useful for progressive approvals and publishing).
    pub fn upsert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO seals (
                id, period_id, root, statement_hash, topic_id, sequence_number,
                consensus_timestamp, approver_1_pubkey, approver_1_sig,
                approver_2_pubkey, approver_2_sig, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(period_id) DO UPDATE SET
                root = excluded.root,
                statement_hash = excluded.statement_hash,
                topic_id = excluded.topic_id,
                sequence_number = excluded.sequence_number,
                consensus_timestamp = excluded.consensus_timestamp,
                approver_1_pubkey = excluded.approver_1_pubkey,
                approver_1_sig = excluded.approver_1_sig,
                approver_2_pubkey = excluded.approver_2_pubkey,
                approver_2_sig = excluded.approver_2_sig;",
            params![
                &self.id,
                &self.period_id,
                &self.root,
                &self.statement_hash,
                &self.topic_id,
                &self.sequence_number,
                &self.consensus_timestamp,
                &self.approver_1_pubkey,
                &self.approver_1_sig,
                &self.approver_2_pubkey,
                &self.approver_2_sig,
                &self.created_at
            ],
        )?;
        Ok(())
    }

    /// Retrieves a seal record by its associated period ID.
    pub fn find_by_period_id(conn: &Connection, period_id: &str) -> Result<Self, DbError> {
        conn.query_row(
            "SELECT id, period_id, root, statement_hash, topic_id, sequence_number,
                    consensus_timestamp, approver_1_pubkey, approver_1_sig,
                    approver_2_pubkey, approver_2_sig, created_at
             FROM seals WHERE period_id = ?1;",
            params![period_id],
            |row| {
                Ok(SealRecord {
                    id: row.get(0)?,
                    period_id: row.get(1)?,
                    root: row.get(2)?,
                    statement_hash: row.get(3)?,
                    topic_id: row.get(4)?,
                    sequence_number: row.get(5)?,
                    consensus_timestamp: row.get(6)?,
                    approver_1_pubkey: row.get(7)?,
                    approver_1_sig: row.get(8)?,
                    approver_2_pubkey: row.get(9)?,
                    approver_2_sig: row.get(10)?,
                    created_at: row.get(11)?,
                })
            },
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => DbError::SealNotFound(period_id.to_string()),
            other => DbError::Sqlite(other),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repository::period::PeriodRecord;
    use crate::db::schema::migrate;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();
        migrate(&conn).unwrap();
        conn
    }

    #[test]
    fn test_insert_and_find_seal() {
        let conn = setup_test_db();
        // Insert period first (foreign key requirement)
        let period = PeriodRecord {
            id: "per_2026_08".into(),
            entity: "Acme Corp".into(),
            start_date: "2026-08-01".into(),
            end_date: "2026-08-31".into(),
            status: "sealed".into(),
        };
        period.insert(&conn).unwrap();

        let seal = SealRecord {
            id: "seal_01".into(),
            period_id: "per_2026_08".into(),
            root: "abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234".into(),
            statement_hash: "111122223333444455556666777788889999aaaabbbbccccddddeeeeffff0000"
                .into(),
            topic_id: Some("0.0.10462941".into()),
            sequence_number: Some(42),
            consensus_timestamp: Some("1725148800.123456789".into()),
            approver_1_pubkey: Some("02abc...".into()),
            approver_1_sig: Some("sig1...".into()),
            approver_2_pubkey: Some("03def...".into()),
            approver_2_sig: Some("sig2...".into()),
            created_at: "2026-09-01T12:00:00Z".into(),
        };

        seal.insert(&conn).unwrap();
        let loaded = SealRecord::find_by_period_id(&conn, "per_2026_08").unwrap();
        assert_eq!(seal, loaded);
    }

    #[test]
    fn test_find_nonexistent_seal_returns_seal_not_found() {
        let conn = setup_test_db();
        let err = SealRecord::find_by_period_id(&conn, "per_missing").unwrap_err();
        match err {
            DbError::SealNotFound(id) => assert_eq!(id, "per_missing"),
            other => panic!("Expected SealNotFound, got {other:?}"),
        }
    }

    #[test]
    fn test_duplicate_seal_insert_rejected() {
        let conn = setup_test_db();
        let period = PeriodRecord {
            id: "per_2026_08".into(),
            entity: "Acme Corp".into(),
            start_date: "2026-08-01".into(),
            end_date: "2026-08-31".into(),
            status: "sealed".into(),
        };
        period.insert(&conn).unwrap();

        let s1 = SealRecord {
            id: "seal_01".into(),
            period_id: "per_2026_08".into(),
            root: "root1".into(),
            statement_hash: "hash1".into(),
            topic_id: None,
            sequence_number: None,
            consensus_timestamp: None,
            approver_1_pubkey: None,
            approver_1_sig: None,
            approver_2_pubkey: None,
            approver_2_sig: None,
            created_at: "2026-09-01T12:00:00Z".into(),
        };
        let s2 = SealRecord {
            id: "seal_02".into(),
            period_id: "per_2026_08".into(),
            root: "root2".into(),
            statement_hash: "hash2".into(),
            topic_id: None,
            sequence_number: None,
            consensus_timestamp: None,
            approver_1_pubkey: None,
            approver_1_sig: None,
            approver_2_pubkey: None,
            approver_2_sig: None,
            created_at: "2026-09-01T13:00:00Z".into(),
        };

        s1.insert(&conn).unwrap();
        let err = s2.insert(&conn);
        assert!(
            err.is_err(),
            "UNIQUE constraint on period_id must reject multiple seals for same period"
        );
    }

    #[test]
    fn test_seal_upsert_updates_fields() {
        let conn = setup_test_db();
        let period = PeriodRecord {
            id: "per_2026_08".into(),
            entity: "Acme Corp".into(),
            start_date: "2026-08-01".into(),
            end_date: "2026-08-31".into(),
            status: "open".into(),
        };
        period.insert(&conn).unwrap();

        let mut seal = SealRecord {
            id: "seal_prop".into(),
            period_id: "per_2026_08".into(),
            root: "root_initial".into(),
            statement_hash: "hash_initial".into(),
            topic_id: None,
            sequence_number: None,
            consensus_timestamp: None,
            approver_1_pubkey: None,
            approver_1_sig: None,
            approver_2_pubkey: None,
            approver_2_sig: None,
            created_at: "2026-09-01T10:00:00Z".into(),
        };

        // Propose
        seal.upsert(&conn).unwrap();

        // Approve 1
        seal.approver_1_pubkey = Some("pubkey_1".into());
        seal.approver_1_sig = Some("sig_1".into());
        seal.upsert(&conn).unwrap();

        // Check updated
        let loaded = SealRecord::find_by_period_id(&conn, "per_2026_08").unwrap();
        assert_eq!(loaded.approver_1_pubkey.as_deref(), Some("pubkey_1"));
        assert_eq!(loaded.approver_1_sig.as_deref(), Some("sig_1"));
        assert!(loaded.approver_2_pubkey.is_none());
    }
}
