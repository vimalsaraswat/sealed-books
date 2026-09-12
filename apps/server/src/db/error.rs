//! Database and repository error types for Sealed Books Server.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Core validation error: {0}")]
    Core(#[from] sealed_books_core::CoreError),

    #[error("Period '{0}' not found")]
    PeriodNotFound(String),

    #[error("Period '{0}' is sealed and cannot be modified")]
    PeriodSealed(String),

    #[error("Account '{0}' not found")]
    AccountNotFound(String),

    #[error("Entry '{0}' not found")]
    EntryNotFound(String),

    #[error("Seal for period '{0}' not found")]
    SealNotFound(String),

    #[error("Seal for period '{0}' already exists")]
    SealAlreadyExists(String),

    #[error("Entity not found: {0}")]
    EntityNotFound(String),
}
