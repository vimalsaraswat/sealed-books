//! On-chain seal audit records and queries.

use crate::db::error::DbError;
use crate::db::repository::period::PeriodRecord;
use libsql::{Connection, params};
use serde::{Deserialize, Serialize};

fn default_dispatch_status() -> String {
    "draft".to_string()
}

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
    #[serde(default = "default_dispatch_status")]
    pub dispatch_status: String,
    pub auditor_id: Option<String>,
    pub auditor_notes: Option<String>,
    pub created_at: String,
}

impl SealRecord {
    /// Helper to map database rows to (SealRecord, PeriodRecord)
    fn map_seal_and_period(row: &libsql::Row) -> Result<(SealRecord, PeriodRecord), libsql::Error> {
        Ok((
            SealRecord {
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
                dispatch_status: row.get(11)?,
                auditor_id: row.get(12)?,
                auditor_notes: row.get(13)?,
                created_at: row.get(14)?,
            },
            PeriodRecord {
                id: row.get(15)?,
                organization_id: row.get(16)?,
                entity: row.get(17)?,
                start_date: row.get(18)?,
                end_date: row.get(19)?,
                status: row.get(20)?,
            },
        ))
    }

    /// Inserts this seal record into the database.
    pub async fn insert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO seals (
                id, period_id, root, statement_hash, topic_id, sequence_number,
                consensus_timestamp, approver_1_pubkey, approver_1_sig,
                approver_2_pubkey, approver_2_sig, dispatch_status, auditor_id,
                auditor_notes, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15);",
            params![
                self.id.as_str(),
                self.period_id.as_str(),
                self.root.as_str(),
                self.statement_hash.as_str(),
                self.topic_id.as_deref(),
                self.sequence_number,
                self.consensus_timestamp.as_deref(),
                self.approver_1_pubkey.as_deref(),
                self.approver_1_sig.as_deref(),
                self.approver_2_pubkey.as_deref(),
                self.approver_2_sig.as_deref(),
                if self.dispatch_status.is_empty() {
                    "draft"
                } else {
                    self.dispatch_status.as_str()
                },
                self.auditor_id.as_deref(),
                self.auditor_notes.as_deref(),
                self.created_at.as_str()
            ],
        )
        .await?;
        Ok(())
    }

    /// Inserts or updates this seal record (useful for progressive approvals and publishing).
    pub async fn upsert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO seals (
                id, period_id, root, statement_hash, topic_id, sequence_number,
                consensus_timestamp, approver_1_pubkey, approver_1_sig,
                approver_2_pubkey, approver_2_sig, dispatch_status, auditor_id,
                auditor_notes, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
             ON CONFLICT(period_id) DO UPDATE SET
                root = excluded.root,
                statement_hash = excluded.statement_hash,
                topic_id = excluded.topic_id,
                sequence_number = excluded.sequence_number,
                consensus_timestamp = excluded.consensus_timestamp,
                approver_1_pubkey = excluded.approver_1_pubkey,
                approver_1_sig = excluded.approver_1_sig,
                approver_2_pubkey = excluded.approver_2_pubkey,
                approver_2_sig = excluded.approver_2_sig,
                dispatch_status = excluded.dispatch_status,
                auditor_id = excluded.auditor_id,
                auditor_notes = excluded.auditor_notes;",
            params![
                self.id.as_str(),
                self.period_id.as_str(),
                self.root.as_str(),
                self.statement_hash.as_str(),
                self.topic_id.as_deref(),
                self.sequence_number,
                self.consensus_timestamp.as_deref(),
                self.approver_1_pubkey.as_deref(),
                self.approver_1_sig.as_deref(),
                self.approver_2_pubkey.as_deref(),
                self.approver_2_sig.as_deref(),
                if self.dispatch_status.is_empty() {
                    "draft"
                } else {
                    self.dispatch_status.as_str()
                },
                self.auditor_id.as_deref(),
                self.auditor_notes.as_deref(),
                self.created_at.as_str()
            ],
        )
        .await?;
        Ok(())
    }

    /// Dispatches a seal to an external auditor for review.
    pub async fn dispatch_to_auditor(
        conn: &Connection,
        period_id: &str,
        auditor_id: &str,
    ) -> Result<(), DbError> {
        let rows = conn.execute(
            "UPDATE seals SET dispatch_status = 'pending_auditor', auditor_id = ?1 WHERE period_id = ?2;",
            params![auditor_id, period_id],
        ).await?;
        if rows == 0 {
            return Err(DbError::SealNotFound(period_id.to_string()));
        }
        Ok(())
    }

    /// Rejects an audit proposal and returns feedback notes to the controller.
    pub async fn reject_audit(
        conn: &Connection,
        period_id: &str,
        notes: &str,
    ) -> Result<(), DbError> {
        let rows = conn.execute(
            "UPDATE seals SET dispatch_status = 'rejected', auditor_notes = ?1 WHERE period_id = ?2;",
            params![notes, period_id],
        ).await?;
        if rows == 0 {
            return Err(DbError::SealNotFound(period_id.to_string()));
        }
        Ok(())
    }

    /// Retrieves a seal record by its associated period ID.
    pub async fn find_by_period_id(conn: &Connection, period_id: &str) -> Result<Self, DbError> {
        let mut rows = conn
            .query(
                "SELECT id, period_id, root, statement_hash, topic_id, sequence_number,
                        consensus_timestamp, approver_1_pubkey, approver_1_sig,
                        approver_2_pubkey, approver_2_sig, dispatch_status, auditor_id,
                        auditor_notes, created_at
                 FROM seals WHERE period_id = ?1;",
                params![period_id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
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
                dispatch_status: row.get(11)?,
                auditor_id: row.get(12)?,
                auditor_notes: row.get(13)?,
                created_at: row.get(14)?,
            })
        } else {
            Err(DbError::SealNotFound(period_id.to_string()))
        }
    }

    /// Lists all periods currently pending review by an auditor.
    pub async fn list_pending_audits(
        conn: &Connection,
        auditor_id_opt: Option<&str>,
    ) -> Result<Vec<(Self, PeriodRecord)>, DbError> {
        let mut results = Vec::new();
        if let Some(auditor_id) = auditor_id_opt {
            let mut rows = conn.query(
                "SELECT s.id, s.period_id, s.root, s.statement_hash, s.topic_id, s.sequence_number,
                        s.consensus_timestamp, s.approver_1_pubkey, s.approver_1_sig,
                        s.approver_2_pubkey, s.approver_2_sig, s.dispatch_status, s.auditor_id,
                        s.auditor_notes, s.created_at,
                        p.id, p.organization_id, p.entity, p.start_date, p.end_date, p.status
                 FROM seals s
                 JOIN periods p ON s.period_id = p.id
                 WHERE s.dispatch_status = 'pending_auditor' AND (s.auditor_id = ?1 OR s.auditor_id IS NULL)
                 ORDER BY s.created_at DESC;",
                params![auditor_id],
            ).await?;
            while let Some(row) = rows.next().await? {
                results.push(Self::map_seal_and_period(&row)?);
            }
        } else {
            let mut rows = conn.query(
                "SELECT s.id, s.period_id, s.root, s.statement_hash, s.topic_id, s.sequence_number,
                        s.consensus_timestamp, s.approver_1_pubkey, s.approver_1_sig,
                        s.approver_2_pubkey, s.approver_2_sig, s.dispatch_status, s.auditor_id,
                        s.auditor_notes, s.created_at,
                        p.id, p.organization_id, p.entity, p.start_date, p.end_date, p.status
                 FROM seals s
                 JOIN periods p ON s.period_id = p.id
                 WHERE s.dispatch_status = 'pending_auditor'
                 ORDER BY s.created_at DESC;",
                (),
            ).await?;
            while let Some(row) = rows.next().await? {
                results.push(Self::map_seal_and_period(&row)?);
            }
        }
        Ok(results)
    }

    /// Persists baseline leaf hashes for an accounting period.
    pub async fn save_leaves(
        conn: &Connection,
        period_id: &str,
        leaves: &[(String, [u8; 32])],
    ) -> Result<(), DbError> {
        for (entry_id, leaf_bytes) in leaves {
            let hash_hex = format!("0x{}", hex::encode(leaf_bytes));
            conn.execute(
                "INSERT OR REPLACE INTO seal_leaves (period_id, entry_id, leaf_hash) VALUES (?1, ?2, ?3);",
                params![period_id, entry_id.as_str(), hash_hex.as_str()],
            ).await?;
        }
        Ok(())
    }

    /// Retrieves all saved baseline leaf hashes for an accounting period.
    pub async fn get_leaves(
        conn: &Connection,
        period_id: &str,
    ) -> Result<Vec<(String, [u8; 32])>, DbError> {
        let mut rows = conn
            .query(
                "SELECT entry_id, leaf_hash FROM seal_leaves WHERE period_id = ?1 ORDER BY entry_id ASC;",
                params![period_id],
            )
            .await?;

        let mut list = Vec::new();
        while let Some(row) = rows.next().await? {
            let entry_id: String = row.get(0)?;
            let leaf_hex: String = row.get(1)?;
            let clean = leaf_hex.trim_start_matches("0x");
            if let Ok(bytes) = hex::decode(clean) {
                if bytes.len() != 32 {
                    continue;
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                list.push((entry_id, arr));
            }
        }

        Ok(list)
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
        conn.execute(
            "INSERT INTO periods (id, organization_id, entity, start_date, end_date, status) VALUES ('per_2026_08', 'org_acme', 'Acme Corp', '2026-08-01', '2026-08-31', 'open');",
            (),
        ).await.unwrap();
        conn
    }

    #[tokio::test]
    async fn test_insert_and_find_seal() {
        let conn = setup_test_db().await;
        let seal = SealRecord {
            id: "seal_1".into(),
            period_id: "per_2026_08".into(),
            root: "0x1234".into(),
            statement_hash: "0x5678".into(),
            topic_id: Some("0.0.12345".into()),
            sequence_number: Some(42),
            consensus_timestamp: Some("2026-09-01T12:00:00Z".into()),
            approver_1_pubkey: Some("0xpub1".into()),
            approver_1_sig: Some("0xsig1".into()),
            approver_2_pubkey: None,
            approver_2_sig: None,
            dispatch_status: "draft".into(),
            auditor_id: None,
            auditor_notes: None,
            created_at: "2026-09-01T12:00:00Z".into(),
        };

        seal.insert(&conn).await.unwrap();
        let fetched = SealRecord::find_by_period_id(&conn, "per_2026_08")
            .await
            .unwrap();
        assert_eq!(fetched, seal);
    }

    #[tokio::test]
    async fn test_find_nonexistent_seal_returns_seal_not_found() {
        let conn = setup_test_db().await;
        let err = SealRecord::find_by_period_id(&conn, "per_nonexistent")
            .await
            .unwrap_err();
        assert!(matches!(err, DbError::SealNotFound(_)));
    }

    #[tokio::test]
    async fn test_duplicate_seal_insert_rejected() {
        let conn = setup_test_db().await;
        let seal1 = SealRecord {
            id: "seal_1".into(),
            period_id: "per_2026_08".into(),
            root: "0x1111".into(),
            statement_hash: "0x2222".into(),
            topic_id: None,
            sequence_number: None,
            consensus_timestamp: None,
            approver_1_pubkey: None,
            approver_1_sig: None,
            approver_2_pubkey: None,
            approver_2_sig: None,
            dispatch_status: "draft".into(),
            auditor_id: None,
            auditor_notes: None,
            created_at: "2026-09-01T12:00:00Z".into(),
        };
        seal1.insert(&conn).await.unwrap();

        let seal2 = SealRecord {
            id: "seal_2".into(),
            period_id: "per_2026_08".into(),
            root: "0x3333".into(),
            statement_hash: "0x4444".into(),
            topic_id: None,
            sequence_number: None,
            consensus_timestamp: None,
            approver_1_pubkey: None,
            approver_1_sig: None,
            approver_2_pubkey: None,
            approver_2_sig: None,
            dispatch_status: "draft".into(),
            auditor_id: None,
            auditor_notes: None,
            created_at: "2026-09-01T13:00:00Z".into(),
        };
        let err = seal2.insert(&conn).await.unwrap_err();
        assert!(matches!(err, DbError::Sqlite(_)));
    }

    #[tokio::test]
    async fn test_seal_upsert_updates_fields() {
        let conn = setup_test_db().await;
        let mut seal = SealRecord {
            id: "seal_1".into(),
            period_id: "per_2026_08".into(),
            root: "0x1111".into(),
            statement_hash: "0x2222".into(),
            topic_id: None,
            sequence_number: None,
            consensus_timestamp: None,
            approver_1_pubkey: Some("0xpub1".into()),
            approver_1_sig: Some("0xsig1".into()),
            approver_2_pubkey: None,
            approver_2_sig: None,
            dispatch_status: "draft".into(),
            auditor_id: None,
            auditor_notes: None,
            created_at: "2026-09-01T12:00:00Z".into(),
        };
        seal.insert(&conn).await.unwrap();

        // Update with approver 2 and Hedera publication info
        seal.approver_2_pubkey = Some("0xpub2".into());
        seal.approver_2_sig = Some("0xsig2".into());
        seal.topic_id = Some("0.0.99999".into());
        seal.sequence_number = Some(7);
        seal.consensus_timestamp = Some("2026-09-01T15:00:00Z".into());

        seal.upsert(&conn).await.unwrap();

        let fetched = SealRecord::find_by_period_id(&conn, "per_2026_08")
            .await
            .unwrap();
        assert_eq!(fetched.approver_2_pubkey, Some("0xpub2".into()));
        assert_eq!(fetched.sequence_number, Some(7));
        assert_eq!(fetched.topic_id, Some("0.0.99999".into()));
    }

    #[tokio::test]
    async fn test_seal_leaves_save_and_get() {
        let conn = setup_test_db().await;
        let leaves = vec![
            ("ent_01".to_string(), [1u8; 32]),
            ("ent_02".to_string(), [2u8; 32]),
        ];

        SealRecord::save_leaves(&conn, "per_2026_08", &leaves)
            .await
            .unwrap();
        let loaded = SealRecord::get_leaves(&conn, "per_2026_08").await.unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].0, "ent_01");
        assert_eq!(loaded[0].1, [1u8; 32]);
        assert_eq!(loaded[1].0, "ent_02");
        assert_eq!(loaded[1].1, [2u8; 32]);
    }
}
