//! Error types for Sealed Books Core.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum CoreError {
    #[error("Journal entry '{entry_id}' has fewer than 2 lines (found {line_count})")]
    InvalidLineCount { entry_id: String, line_count: usize },

    #[error(
        "Journal entry '{entry_id}' is unbalanced: debits ({debits_minor}) != credits ({credits_minor})"
    )]
    UnbalancedEntry {
        entry_id: String,
        debits_minor: u64,
        credits_minor: u64,
    },

    #[error("Line '{line_id}' has an invalid amount of 0 (amounts must be positive minor units)")]
    ZeroAmountLine { line_id: String },

    #[error("Invalid date format '{0}': expected YYYY-MM-DD")]
    InvalidDateFormat(String),

    #[error("Invalid period: start date '{start}' is after end date '{end}'")]
    InvalidPeriodRange { start: String, end: String },

    #[error(
        "Entry '{entry_id}' date '{entry_date}' is outside period range '{period_start}' -> '{period_end}'"
    )]
    EntryOutOfPeriodRange {
        entry_id: String,
        entry_date: String,
        period_start: String,
        period_end: String,
    },

    #[error("Serialization error: {0}")]
    SerializationError(String),
}
