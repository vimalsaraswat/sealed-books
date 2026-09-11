//! Sealed Books Core: Canonical encoding, hashing, Merkle root, and verification.
//!
//! This crate contains pure Rust computational logic for tamper-evident accounting.
//! It has zero dependencies on network, databases, Hedera, or the operating system,
//! ensuring full deterministic execution and future WebAssembly (WASM) compatibility.

pub mod encode;
pub mod error;
pub mod types;

pub use encode::{CANONICAL_VERSION_V1, encode_entry, encode_statement};
pub use error::CoreError;
pub use types::{Direction, Entry, Line, Period, SealStatement, VerifyResult};
