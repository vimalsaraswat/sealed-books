//! Account entity model and queries for chart of accounts.

use crate::db::error::DbError;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub code: String,
    pub name: String,
    pub account_type: String,
}

impl Account {
    /// Inserts this account into the database.
    pub fn insert(&self, conn: &Connection) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO accounts (id, code, name, account_type) VALUES (?1, ?2, ?3, ?4);",
            params![&self.id, &self.code, &self.name, &self.account_type],
        )?;
        Ok(())
    }

    /// Finds an account by its unique ID.
    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Self, DbError> {
        conn.query_row(
            "SELECT id, code, name, account_type FROM accounts WHERE id = ?1;",
            params![id],
            |row| {
                Ok(Account {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    name: row.get(2)?,
                    account_type: row.get(3)?,
                })
            },
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => DbError::AccountNotFound(id.to_string()),
            other => DbError::Sqlite(other),
        })
    }

    /// Lists all accounts ordered by account code.
    pub fn list_all(conn: &Connection) -> Result<Vec<Self>, DbError> {
        let mut stmt =
            conn.prepare("SELECT id, code, name, account_type FROM accounts ORDER BY code;")?;
        let accounts = stmt
            .query_map([], |row| {
                Ok(Account {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    name: row.get(2)?,
                    account_type: row.get(3)?,
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
        conn
    }

    #[test]
    fn test_insert_and_find_by_id() {
        let conn = setup_test_db();
        let acc = Account {
            id: "acc_cash".into(),
            code: "1000".into(),
            name: "Petty Cash".into(),
            account_type: "asset".into(),
        };

        acc.insert(&conn).unwrap();
        let loaded = Account::find_by_id(&conn, "acc_cash").unwrap();
        assert_eq!(acc, loaded);
    }

    #[test]
    fn test_find_by_id_not_found() {
        let conn = setup_test_db();
        let err = Account::find_by_id(&conn, "acc_missing").unwrap_err();
        match err {
            DbError::AccountNotFound(id) => assert_eq!(id, "acc_missing"),
            other => panic!("Expected AccountNotFound, got {other:?}"),
        }
    }

    #[test]
    fn test_list_all_ordered_by_code() {
        let conn = setup_test_db();
        let a1 = Account {
            id: "acc_rev".into(),
            code: "4000".into(),
            name: "Revenue".into(),
            account_type: "revenue".into(),
        };
        let a2 = Account {
            id: "acc_asset".into(),
            code: "1000".into(),
            name: "Cash".into(),
            account_type: "asset".into(),
        };

        // Insert in reverse order
        a1.insert(&conn).unwrap();
        a2.insert(&conn).unwrap();

        let list = Account::list_all(&conn).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].code, "1000");
        assert_eq!(list[1].code, "4000");
    }

    #[test]
    fn test_duplicate_code_rejected() {
        let conn = setup_test_db();
        let a1 = Account {
            id: "acc_1".into(),
            code: "1000".into(),
            name: "Cash".into(),
            account_type: "asset".into(),
        };
        let a2 = Account {
            id: "acc_2".into(),
            code: "1000".into(),
            name: "Bank".into(),
            account_type: "asset".into(),
        };

        a1.insert(&conn).unwrap();
        let err = a2.insert(&conn);
        assert!(err.is_err(), "Duplicate account code must be rejected");
    }

    #[test]
    fn test_invalid_account_type_rejected() {
        let conn = setup_test_db();
        let a = Account {
            id: "acc_invalid".into(),
            code: "9999".into(),
            name: "Invalid Type".into(),
            account_type: "not_a_valid_type".into(),
        };

        let err = a.insert(&conn);
        assert!(
            err.is_err(),
            "CHECK constraint must reject invalid account_type"
        );
    }
}
