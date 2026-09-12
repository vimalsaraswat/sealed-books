//! Account entity model and queries for chart of accounts.

use crate::db::error::DbError;
use rusqlite::{Connection, params};
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
    pub fn insert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO accounts (id, organization_id, code, name, account_type) VALUES (?1, ?2, ?3, ?4, ?5);",
            params![
                &self.id,
                if self.organization_id.is_empty() { "org_acme" } else { &self.organization_id },
                &self.code,
                &self.name,
                &self.account_type
            ],
        )?;
        Ok(())
    }

    /// Finds an account by its unique ID.
    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Self, DbError> {
        conn.query_row(
            "SELECT id, organization_id, code, name, account_type FROM accounts WHERE id = ?1;",
            params![id],
            |row| {
                Ok(Account {
                    id: row.get(0)?,
                    organization_id: row.get(1)?,
                    code: row.get(2)?,
                    name: row.get(3)?,
                    account_type: row.get(4)?,
                })
            },
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => DbError::AccountNotFound(id.to_string()),
            other => DbError::Sqlite(other),
        })
    }

    /// Lists all accounts ordered by account code for a specific organization.
    pub fn list_by_org(conn: &Connection, org_id: &str) -> Result<Vec<Self>, DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, organization_id, code, name, account_type FROM accounts WHERE organization_id = ?1 ORDER BY code;",
        )?;
        let accounts = stmt
            .query_map(params![org_id], |row| {
                Ok(Account {
                    id: row.get(0)?,
                    organization_id: row.get(1)?,
                    code: row.get(2)?,
                    name: row.get(3)?,
                    account_type: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(accounts)
    }

    /// Lists all accounts ordered by account code.
    pub fn list_all(conn: &Connection) -> Result<Vec<Self>, DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, organization_id, code, name, account_type FROM accounts ORDER BY code;",
        )?;
        let accounts = stmt
            .query_map([], |row| {
                Ok(Account {
                    id: row.get(0)?,
                    organization_id: row.get(1)?,
                    code: row.get(2)?,
                    name: row.get(3)?,
                    account_type: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(accounts)
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
        // Insert default organization for tests
        conn.execute(
            "INSERT OR IGNORE INTO organizations (id, name, base_currency, created_at) VALUES ('org_acme', 'Acme', 'USD', '2026-08-01T00:00:00Z');",
            [],
        ).unwrap();
        conn
    }

    #[test]
    fn test_insert_and_find_by_id() {
        let conn = setup_test_db();
        let acc = Account {
            id: "acc_1010".into(),
            organization_id: "org_acme".into(),
            code: "1010".into(),
            name: "Operating Cash".into(),
            account_type: "asset".into(),
        };

        acc.insert(&conn).unwrap();
        let fetched = Account::find_by_id(&conn, "acc_1010").unwrap();
        assert_eq!(fetched, acc);
    }

    #[test]
    fn test_find_by_id_not_found() {
        let conn = setup_test_db();
        let err = Account::find_by_id(&conn, "acc_missing").unwrap_err();
        assert!(matches!(err, DbError::AccountNotFound(_)));
    }

    #[test]
    fn test_duplicate_code_rejected() {
        let conn = setup_test_db();
        let acc1 = Account {
            id: "acc_1".into(),
            organization_id: "org_acme".into(),
            code: "1010".into(),
            name: "Operating Cash".into(),
            account_type: "asset".into(),
        };
        acc1.insert(&conn).unwrap();

        let acc2 = Account {
            id: "acc_2".into(),
            organization_id: "org_acme".into(),
            code: "1010".into(),
            name: "Petty Cash".into(),
            account_type: "asset".into(),
        };
        let err = acc2.insert(&conn).unwrap_err();
        assert!(matches!(err, DbError::Sqlite(_)));
    }

    #[test]
    fn test_invalid_account_type_rejected() {
        let conn = setup_test_db();
        let acc = Account {
            id: "acc_1".into(),
            organization_id: "org_acme".into(),
            code: "9999".into(),
            name: "Weird".into(),
            account_type: "invalid_type".into(),
        };
        let err = acc.insert(&conn).unwrap_err();
        assert!(matches!(err, DbError::Sqlite(_)));
    }

    #[test]
    fn test_list_all_ordered_by_code() {
        let conn = setup_test_db();
        let acc2 = Account {
            id: "acc_2".into(),
            organization_id: "org_acme".into(),
            code: "2010".into(),
            name: "Accounts Payable".into(),
            account_type: "liability".into(),
        };
        let acc1 = Account {
            id: "acc_1".into(),
            organization_id: "org_acme".into(),
            code: "1010".into(),
            name: "Cash".into(),
            account_type: "asset".into(),
        };

        acc2.insert(&conn).unwrap();
        acc1.insert(&conn).unwrap();

        let list = Account::list_all(&conn).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].code, "1010");
        assert_eq!(list[1].code, "2010");
    }
}
