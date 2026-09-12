//! Account entity model and queries for chart of accounts.

use crate::db::error::DbError;
use libsql::{Connection, params};
use serde::{Deserialize, Serialize};

fn default_org_id() -> String {
    "org_acme".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    #[serde(default = "default_org_id")]
    pub organization_id: String,
    pub code: String,
    pub name: String,
    pub account_type: String,
}

impl Account {
    /// Inserts this account into the database.
    pub async fn insert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO accounts (id, organization_id, code, name, account_type) VALUES (?1, ?2, ?3, ?4, ?5);",
            params![
                self.id.as_str(),
                if self.organization_id.is_empty() { "org_acme" } else { self.organization_id.as_str() },
                self.code.as_str(),
                self.name.as_str(),
                self.account_type.as_str()
            ],
        ).await?;
        Ok(())
    }

    /// Finds an account by its unique ID.
    pub async fn find_by_id(conn: &Connection, id: &str) -> Result<Self, DbError> {
        let mut rows = conn
            .query(
                "SELECT id, organization_id, code, name, account_type FROM accounts WHERE id = ?1;",
                params![id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Account {
                id: row.get(0)?,
                organization_id: row.get(1)?,
                code: row.get(2)?,
                name: row.get(3)?,
                account_type: row.get(4)?,
            })
        } else {
            Err(DbError::AccountNotFound(id.to_string()))
        }
    }

    /// Lists all accounts ordered by account code for a specific organization.
    pub async fn list_by_org(conn: &Connection, org_id: &str) -> Result<Vec<Self>, DbError> {
        let mut rows = conn
            .query(
                "SELECT id, organization_id, code, name, account_type FROM accounts WHERE organization_id = ?1 ORDER BY code;",
                params![org_id],
            )
            .await?;

        let mut accounts = Vec::new();
        while let Some(row) = rows.next().await? {
            accounts.push(Account {
                id: row.get(0)?,
                organization_id: row.get(1)?,
                code: row.get(2)?,
                name: row.get(3)?,
                account_type: row.get(4)?,
            });
        }
        Ok(accounts)
    }

    /// Lists all accounts ordered by account code.
    pub async fn list_all(conn: &Connection) -> Result<Vec<Self>, DbError> {
        let mut rows = conn
            .query(
                "SELECT id, organization_id, code, name, account_type FROM accounts ORDER BY code;",
                (),
            )
            .await?;

        let mut accounts = Vec::new();
        while let Some(row) = rows.next().await? {
            accounts.push(Account {
                id: row.get(0)?,
                organization_id: row.get(1)?,
                code: row.get(2)?,
                name: row.get(3)?,
                account_type: row.get(4)?,
            });
        }
        Ok(accounts)
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
        // Insert default organization for tests
        conn.execute(
            "INSERT OR IGNORE INTO organizations (id, name, base_currency, created_at) VALUES ('org_acme', 'Acme', 'USD', '2026-08-01T00:00:00Z');",
            (),
        ).await.unwrap();
        conn
    }

    #[tokio::test]
    async fn test_insert_and_find_by_id() {
        let conn = setup_test_db().await;
        let acc = Account {
            id: "acc_1010".into(),
            organization_id: "org_acme".into(),
            code: "1010".into(),
            name: "Operating Cash".into(),
            account_type: "asset".into(),
        };

        acc.insert(&conn).await.unwrap();
        let fetched = Account::find_by_id(&conn, "acc_1010").await.unwrap();
        assert_eq!(fetched, acc);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let conn = setup_test_db().await;
        let err = Account::find_by_id(&conn, "acc_missing").await.unwrap_err();
        assert!(matches!(err, DbError::AccountNotFound(_)));
    }

    #[tokio::test]
    async fn test_duplicate_code_rejected() {
        let conn = setup_test_db().await;
        let acc1 = Account {
            id: "acc_1".into(),
            organization_id: "org_acme".into(),
            code: "1010".into(),
            name: "Operating Cash".into(),
            account_type: "asset".into(),
        };
        acc1.insert(&conn).await.unwrap();

        let acc2 = Account {
            id: "acc_2".into(),
            organization_id: "org_acme".into(),
            code: "1010".into(),
            name: "Petty Cash".into(),
            account_type: "asset".into(),
        };
        let err = acc2.insert(&conn).await.unwrap_err();
        assert!(matches!(err, DbError::Sqlite(_)));
    }

    #[tokio::test]
    async fn test_invalid_account_type_rejected() {
        let conn = setup_test_db().await;
        let acc = Account {
            id: "acc_1".into(),
            organization_id: "org_acme".into(),
            code: "9999".into(),
            name: "Weird".into(),
            account_type: "invalid_type".into(),
        };
        let err = acc.insert(&conn).await.unwrap_err();
        assert!(matches!(err, DbError::Sqlite(_)));
    }

    #[tokio::test]
    async fn test_list_all_ordered_by_code() {
        let conn = setup_test_db().await;
        let acc1 = Account {
            id: "acc_2".into(),
            organization_id: "org_acme".into(),
            code: "2000".into(),
            name: "Accounts Payable".into(),
            account_type: "liability".into(),
        };
        let acc2 = Account {
            id: "acc_1".into(),
            organization_id: "org_acme".into(),
            code: "1010".into(),
            name: "Operating Cash".into(),
            account_type: "asset".into(),
        };
        acc1.insert(&conn).await.unwrap();
        acc2.insert(&conn).await.unwrap();

        let list = Account::list_all(&conn).await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].code, "1010");
        assert_eq!(list[1].code, "2000");
    }
}
