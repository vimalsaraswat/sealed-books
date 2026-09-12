//! Sealed Books Core: Canonical encoding, hashing, Merkle root, and verification.
//!
//! This crate contains pure Rust computational logic for tamper-evident accounting.
//! It has zero dependencies on network, databases, Hedera, or the operating system,
//! ensuring full deterministic execution and future WebAssembly (WASM) compatibility.

pub mod encode;
pub mod error;
pub mod hash;
pub mod merkle;
pub mod types;
pub mod verify;

pub use encode::{
    CANONICAL_VERSION_V1, decode_seal_payload, decode_statement, encode_entry, encode_seal_payload,
    encode_statement,
};
pub use error::CoreError;
pub use hash::{hash_entry, hash_entry_unchecked, statement_hash};
pub use merkle::{build_statement, merkle_root, sort_entries_canonically};
pub use types::{Direction, Entry, Line, Period, SealPayload, SealStatement, VerifyResult};
pub use verify::{verify, verify_with_baseline, verify_with_known_hashes};
