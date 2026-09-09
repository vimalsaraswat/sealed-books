# sealed-books-core

The part of Sealed Books that turns accounting records into a fingerprint, and
later checks that fingerprint again.

Everything else in the project — the database, the server, the web UI, Hedera,
Privy — is plumbing around this crate.

## What lives here

- **Canonical encoding.** Turning a journal entry or a seal statement into an
  exact, repeatable sequence of bytes.
- **Hashing.** Entry → leaf hash. Leaves → Merkle root.
- **The seal statement.** The human-readable summary of a period close that
  approvers actually sign.
- **Verification.** Given a statement and a set of entries, decide whether they
  still match, and if not, say exactly which entry moved.

## What does not live here

No database. No network. No file I/O. No Hedera. No Privy. No clock.

`verify()` takes the published record as an **input**. It does not go and fetch
one.

### Why this rule matters

The Hedera SDK pulls in gRPC (`tonic`), `openssl`, and `tokio`. It cannot
compile to WebAssembly. Keeping this crate free of it means a browser-based
verifier stays a small job later instead of a rewrite.

It also means every rule in here is testable with no test fixtures, no network,
and no waiting.

## Encoding rules

These are not style preferences. Breaking any one of them silently breaks
verification, usually months later and only on someone else's machine.

1. **Fixed field order.** Never derive byte order from a map, a struct literal
   you might reorder, or anything alphabetical-by-accident.
2. **Length-prefixed fields.** So `("ab", "c")` can never encode identically to
   `("a", "bc")`.
3. **Money is an integer in minor units.** Never a float. Not once, not
   anywhere, not "just for display" inside a hashed struct.
4. **Timestamps are UTC, RFC 3339, fixed precision.** No local time. No
   variable fractional seconds.
5. **A version byte leads every encoding.** The day the format changes, old
   seals must still verify.

If you find yourself reaching for `serde_json::to_vec` to produce bytes that get
hashed, stop. That is the bug this section exists to prevent.

## Tests that must always pass

- The same entry hashes to the same value, every time, in any process.
- Changing any single field changes the hash.
- Changing one entry in a period changes the root.
- Writing to the database and reading it back does not change the hash.
- Verification of an unmodified period succeeds.
- Verification of a period with one altered entry fails **and names that entry**.

The last one is the demo. It is also the product.

## Shape of the API

Sketch, not final:

```rust
pub fn hash_entry(entry: &Entry) -> [u8; 32];
pub fn merkle_root(leaves: &[[u8; 32]]) -> [u8; 32];

pub fn build_statement(period: &Period, entries: &[Entry]) -> SealStatement;
pub fn statement_hash(statement: &SealStatement) -> [u8; 32];

pub fn verify(statement: &SealStatement, entries: &[Entry]) -> VerifyResult;
```

`VerifyResult` is either `Intact`, or `Changed` carrying enough detail to point
at the offending entry by id. A boolean is not good enough — "something changed"
is not a useful thing to tell an auditor.

## What this crate proves, and what it does not

It proves that a set of records is **byte-for-byte what it was** when the
statement was built.

It does not prove the records were correct, complete, or honest in the first
place. Garbage in, garbage out. Converting "trust our audit log" into a fact
anyone can check is the entire, deliberate scope.

## Notes

- The workspace is on `edition = "2024"`, which needs a recent Rust toolchain.
- The package is currently named `core`, which collides with Rust's built-in
  `core` library and will need renaming (`sealed-books-core`) before other
  crates depend on it.

## Status

Scaffolding only. See `PLAN.md` at the repository root, Phase 1.
