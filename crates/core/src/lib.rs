//! Sealed Books Core: Canonical encoding, hashing, Merkle root, and verification.
//!
//! This crate contains pure Rust computational logic for tamper-evident accounting.
//! It has zero dependencies on network, databases, Hedera, or the operating system,
//! ensuring full deterministic execution and future WebAssembly (WASM) compatibility.

pub mod encode;
pub mod error;
pub mod hash;
pub mod types;

pub use encode::{CANONICAL_VERSION_V1, encode_entry, encode_statement};
pub use error::CoreError;
pub use hash::{hash_entry, hash_entry_unchecked, statement_hash};
pub use types::{Direction, Entry, Line, Period, SealStatement, VerifyResult};
