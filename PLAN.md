# Sealed Books — Build Plan

A small accounting app that can prove its books were not edited after the fact.

---

## 1. Scope

### Building

1. **A ledger.** Journal entries in SQLite. Date, description, debit account, credit account, amount. Entries must balance.
2. **A close.** Fingerprint the period, require two of three people to approve, publish the result to Hedera Consensus Service.
3. **A check.** Compare today's books against what was published. Green, or red with the exact entry that changed.

### Not building

Invoices. Client acceptance. Payments. Tokens. Invoice financing. Browser-side (WASM) verification. Selective-disclosure proofs. Hedera threshold keys. Multi-currency. Reporting.

All good roadmap. None of it is this build.

---

## 2. Who this is for

Two different groups, and the product treats them differently.

**Signers** are inside the business or its accounts function — whoever prepared the books, plus whoever is accountable for them. They need to understand what they are attesting to, which is why they sign a readable statement rather than a hash.

**Verifiers** are outsiders — the auditor, a lender, a buyer in due diligence, the tax authority. They need to check without trusting the business at all, which is why the record lives somewhere the business does not control.

In smaller businesses the same person often both keeps the books and signs off on them. That is precisely the case where one signature is worthless, and precisely why the two-of-three rule earns its place. Demo approvers should be plausibly different functions: bookkeeper, owner or CFO, external accountant.

---

## 3. Two design decisions, already made

### 3.1 Quorum is published, not network-enforced

The two approvals are **signatures carried inside the published message**. Anyone can verify them forever from the public record.

The rejected alternative was a Hedera 2-of-3 threshold key as the topic's submit key, so the network itself refuses a single-signed seal. That is strictly stronger, and it is confirmed possible — Hedera threshold keys support secp256k1, and Privy exposes compressed secp256k1 public keys. It is also the most expensive and most failure-prone piece in the whole design, because it requires building, freezing, and attaching externally-produced signatures to a Hedera transaction.

Trade accepted: the two-signature rule is enforced by the application, and the evidence of both approvals is permanent and publicly checkable.

This is the documented upgrade path, not a mistake to hide.

### 3.2 Approvers sign a readable statement, not a bare hash

A signature over a naked hash means nothing to a human or an auditor. Approvers sign the canonical encoding of this:

```
Sealed Books — Period Close
Entity:        Acme Trading Pvt Ltd
Period:        2026-08-01 -> 2026-08-31
Entries:       1247
Total debits:  8432150 (INR minor units)
Total credits: 8432150 (INR minor units)
Ledger root:   a4f9...c2
```

Readable to a person, and it binds the root. What they approved is what you can read back to them.

---

## 4. Architecture

```
crates/core        Pure Rust. Canonical encoding, hashing, Merkle root,
                   seal statement, verification logic.
                   NO Hedera dependency. NO network. NO database.

apps/server        Rust. SQLite. Hedera publishing. Privy server API.
                   Holds the Hedera operator key and the Privy app secret.

apps/web           Vite + React. Ledger UI, close flow, approval page,
                   verify button. Privy React SDK.

apps/desktop       Tauri shell. Last, and only if there is room.
```

**Hard rule:** `crates/core` never imports the `hedera` crate. The Hedera SDK pulls in gRPC (`tonic`), `openssl`, and `tokio`; it cannot compile to WASM. Verification takes an anchor record as an _input_ — it does not go and fetch one. This keeps a browser verifier cheap to add later.

---

## 5. Data model (sketch)

```
accounts     id, code, name, type
entries      id, date, description, created_at
lines        id, entry_id, account_id, direction (D/C), amount_minor
periods      id, entity, start_date, end_date, status
seals        id, period_id, root, statement_hash,
             topic_id, sequence_number, consensus_timestamp,
             approver_1_pubkey, approver_1_sig,
             approver_2_pubkey, approver_2_sig
```

Money is **integer minor units**. Never a float, anywhere, for any reason.
Timestamps are **UTC, RFC 3339, fixed precision**.

---

## 6. The published message

Payload written to the HCS topic:

```
version
entity
period_start
period_end
entry_count
total_debits_minor
total_credits_minor
root                (32 bytes)
approver_1_pubkey   (33 bytes, compressed secp256k1)
approver_1_sig      (64 bytes, r||s)
approver_2_pubkey   (33 bytes)
approver_2_sig      (64 bytes)
```

Roughly 430 bytes. The HCS message limit is **1024 bytes**. Comfortable, no chunking needed.

---

## 7. Phases

Sequential. Each is useless without the one before it.

### Phase 0 — Kill the unknowns

Throwaway code. Delete it afterwards.

- [ ] Get a Hedera testnet account (portal.hedera.com). Use an **ECDSA** key.
- [ ] From Rust: create a topic, submit one message. Confirm it on HashScan.
- [ ] Read that message back from the mirror node REST endpoint. The payload is base64.
- [ ] Create a Privy wallet. Read its `public_key`. Sign a hash with `secp256k1_sign`. Verify the signature locally.

Nothing else starts until all four work.

### Phase 1 — The fingerprint (`crates/core`)

No database, no network, no Privy. See `crates/core/README.md` for the encoding rules.

- [ ] Canonical encoding: version byte, fixed field order, length-prefixed, integer money, UTC timestamps.
- [ ] `hash_entry(entry) -> [u8; 32]`
- [ ] `merkle_root(leaves) -> [u8; 32]`
- [ ] `build_statement(period, entries) -> SealStatement`
- [ ] `statement_hash(statement) -> [u8; 32]`
- [ ] `verify(statement, entries) -> VerifyResult` — `Intact`, or `Changed` naming the entry

Tests that must pass:

- Same entry hashes identically, every time.
- Changing any single field changes the hash.
- Changing one entry changes the root.
- Save to DB and reload does not change the hash.

This is the foundation. If it is shaky, nothing above it means anything.

### Phase 2 — The books (`apps/server`)

- [ ] Schema and migrations.
- [ ] Post an entry. Reject unbalanced entries — debits must equal credits.
- [ ] List entries for a period.
- [ ] Seed one realistic month for a small trading business. Not `foo` and `bar`; the demo reads better and bugs surface faster.

### Phase 3 — The seal

- [ ] Provision three Privy wallets for the organisation's approvers.
- [ ] Attach a Privy **policy**. Hard track qualification requirement — do not leave it to the end.
- [ ] `POST /seal/propose` — builds the statement from the current period, returns it plus its hash.
- [ ] Approval endpoint — records one approver's signature against a proposed seal.
- [ ] `POST /seal/publish` — refuses with fewer than two distinct approvers; otherwise assembles the payload and submits it to the topic.
- [ ] Store topic id, sequence number, consensus timestamp in `seals`.

### Phase 4 — The check

- [ ] Fetch `GET /api/v1/topics/{topicId}/messages/{sequenceNumber}` from the testnet mirror node. Plain HTTP, no SDK.
- [ ] Base64-decode and parse the payload.
- [ ] Recompute the root from the database as it stands right now.
- [ ] Match → green.
- [ ] Mismatch → recompute each leaf hash, find and name the entry that moved.
- [ ] Verify both signatures against the recorded approver public keys, so "who approved this" is proven rather than remembered.

### Phase 5 — The face (`apps/web`)

Plain. No design system.

- [ ] Entries table.
- [ ] **Close Period** — shows the statement, waits for approvals.
- [ ] Approval page — Privy email login, statement in plain English, Approve button.
- [ ] **Verify** — green tick, or red naming the offending entry.

### Phase 6 — Prove it works

- [ ] A script that quietly edits a sealed entry directly in SQLite.
- [ ] Run the demo end to end, twice.
- [ ] README, architecture diagram, demo video.

### If there is room

- [ ] Point the existing Tauri shell at the app.

---

## 8. Demo script

1. Show a month of books for a small business.
2. Close the period. Two people approve by email login. The seal is published.
3. Show the record on HashScan — public, timestamped, not ours.
4. Run the tamper script. One entry is quietly altered.
5. Hit Verify. Red. It names the exact entry.

That is the whole pitch. Anything that does not serve these five steps is cuttable.

---

## 9. Verified facts

Checked against primary sources. Do not re-research these.

| Fact                                                                                                                             | Source                                |
| -------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------- | --- | --- | ------------------------ | --- | ----------------------------- | ------------------ |
| `hedera` crate on crates.io, v0.43.0, Apache-2.0                                                                                 | crates.io                             |
| `TopicCreateTransaction`, `TopicMessageSubmitTransaction`, `Client`, `KeyList`, `PrivateKey`, `PublicKey`, `TopicId` all present | docs.rs/hedera/0.43.0                 |
| SDK depends on `tonic`, `openssl`, `hyper-openssl`, `tokio` — native-only, not WASM                                              | docs.rs dependency list               |
| Building from crates.io needs `protoc` and OpenSSL dev headers                                                                   | hiero-sdk-rust README                 |
| HCS message max size 1024 bytes; whole transaction max 6KB including signatures                                                  | Hedera protobuf reference             |
| Mirror node: `GET /api/v1/topics/{topicId}/messages` and `.../messages/{sequenceNumber}`                                         | Hedera REST API reference             |
| Testnet mirror base URL: `https://testnet.mirrornode.hedera.com`                                                                 | Hedera REST API reference             |
| Mirror node returns `message` **base64-encoded**, plus `consensus_timestamp`, `sequence_number`, `running_hash`                  | Hedera REST API reference             |
| Hedera requires the **33-byte compressed** form for secp256k1 public keys                                                        | Hedera protobuf `Key` reference       |
| Hedera threshold keys support both Ed25519 and secp256k1                                                                         | Hedera docs, "Create a threshold key" |
| Privy wallet object exposes `public_key`: "the compressed, raw public key for the wallet along the chain cryptographic curve"    | Privy REST API reference              |
| Privy `secp256k1_sign` signs a raw hash; also available client-side via the React SDK EIP-1193 provider                          | Privy docs                            |
| Privy returns a 65-byte signature (`r                                                                                            |                                       | s   |     | v`). Hedera wants 64 (`r |     | s`) — **strip the last byte** | Privy docs example |
| Privy **policies** are generally available; **key quorums** are gated                                                            | Privy docs                            |
| Privy REST API uses Basic auth with the app secret — **server-side only**, never in the browser or the desktop binary            | Privy REST API reference              |

---

## 10. Open questions

Resolve these during Phase 0, not later.

1. **Is `public_key` actually populated for `chain_type: ethereum` wallets?** The field is optional in the schema. If null, recover the public key from a signature using the `v` recovery byte instead.
2. **Which hash does Hedera sign over for secp256k1?** Only matters if the threshold-key upgrade is attempted. Not needed for this build.
3. **Can a policy be attached to a user-owned embedded wallet, or do approver wallets need to be organisation-owned wallets created server-side?** Organisation-owned wallets with policies may match the track language better ("organization wallets, policies, team permissions"). Decide early — it changes the approval flow.

---

## 11. Package naming

Settled. Directory names stay short; package names are globally unambiguous.

| Directory                | Rust crate                                              | npm package             |
| ------------------------ | ------------------------------------------------------- | ----------------------- |
| `crates/core`            | `sealed-books-core`                                     | —                       |
| `apps/server`            | `sealed-books-server`                                   | —                       |
| `apps/desktop/src-tauri` | `sealed-books-desktop` (lib `sealed_books_desktop_lib`) | —                       |
| `apps/desktop`           | —                                                       | `@sealed-books/desktop` |
| `apps/web`               | —                                                       | `@sealed-books/web`     |
| `packages/ui`            | —                                                       | `@sealed-books/ui`      |
| repository root          | workspace                                               | `sealed-books`          |

Shared Rust metadata (`version`, `edition`, `license`, `authors`) is inherited
from `[workspace.package]`. Shared dependencies live in
`[workspace.dependencies]` — add a crate there once, then use
`thing.workspace = true` in each member.

### Also settled during the rename

- `[profile.release]` moved from `apps/desktop/src-tauri/Cargo.toml` to the workspace root. Cargo silently ignores profiles declared in workspace members, so the Tauri release optimisations were never being applied. Note this now also applies `panic = "abort"` to the server; drop that line if unwinding is wanted there.
- `Cargo.lock` is no longer gitignored. The workspace builds binaries, so the lockfile belongs in version control.
- The Tauri crate moved from edition 2021 to the workspace's edition 2024. `cargo check --workspace` passes.

### Still open

- `crates/core` and `apps/server` are empty scaffolding (`cargo new` boilerplate).
- `packages/ui` is scaffolded as a standalone Vite app (own `index.html`, `main.tsx`, `vite.config.ts`) rather than a library. Fine to leave, but it is not importable as a shared component package in its current shape.

---

## 12. Honest limits

State these plainly rather than letting a judge find them.

- This proves **immutability since close**, not correctness. Garbage in, garbage out. Converting "trust our audit log" into a public mathematical fact is the entire and deliberate scope.
- The two-signature rule is enforced by the application, not by the network. The signatures themselves are permanent and publicly verifiable. Network enforcement via a threshold submit key is the documented next step.
- Nothing sensitive is published. Only hashes, counts, control totals, public keys, and signatures ever leave the machine.

---

## 13. Target

**Privy — Best B2B financial product ($2,500).** Single track, integrated properly.

Hedera is in the build because HCS is the right tool for independent timestamping and ordering, not because it pays — there is no HCS track this cycle.
