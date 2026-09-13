<p align="center">
  <img src="./sealed-books-logo.png" width="120" height="120" alt="Sealed Books Logo" style="border-radius: 24px;" />
</p>

# Sealed Books

**Sealed Books** is a dual-custody cryptographic accounting ledger available as both a **web application** and a **native desktop app (Tauri v2)**. It enforces statutory accounting period-close finality using **Privy Server Wallets**, **SHA-256 Merkle Trees**, and the **Hedera Consensus Service (HCS)**.

If anyone quietly modifies an entry directly in SQLite—changing an amount, altering a memo, backdating, or deleting a record—the zero-trust verification engine flags the ledger as **TAMPERED** and isolates the exact offending transaction.

---

## 🚀 Key Features

- **Strict Double-Entry Accounting**: Integer minor-unit arithmetic (never floats) with strict debit/credit trial balance invariance.
- **Deterministic SHA-256 Merkle Seal**: Hashes canonicalized journal entries into a binary Merkle root and canonical statement hash.
- **Four-Eyes Principle (2-of-2 Multi-Sig Quorum)**:
  - **Approver 1 (Controller)**: Signs the statement pre-hash using an organization-backed **Privy Server Wallet**.
  - **Approver 2 (Statutory Auditor)**: Independently reviews the trial balance and signs with their own Privy wallet.
  - **Anti-Self-Approval**: Enforces strict separation of duties (one person cannot sign both roles).
- **Hedera Consensus Anchoring**: The 303-byte dual-signed payload is permanently broadcast to Hedera Topic `#0.0.10462941` on Testnet.
- **Zero-Trust Verification Engine**: Queries the public Hedera Mirror Node over HTTP to independently verify database records against on-chain consensus.
- **Tamper Simulation Sandbox**: Built-in interactive lab to inject database mutations and watch the engine isolate fraudulent records.
- **Autonomous Agent Audits (Bazantic MCP)**: Exposes the `/verify` endpoint as a Model Context Protocol (MCP) tool and recipe for autonomous AI auditor agents.
- **Web & Desktop Native**: Available as a modern web app (React 19 + Tailwind CSS v4) and a lightweight cross-platform desktop application powered by **Tauri v2**.

---

## 🏛️ How It Works

```
 ┌──────────────────────┐         ┌──────────────────────┐
 │ Financial Controller │         │  Statutory Auditor   │
 │ (Privy Server Wallet)│         │ (Privy Server Wallet)│
 └──────────┬───────────┘         └──────────┬───────────┘
            │ Sign Approver 1                │ Sign Approver 2
            └────────────────┬───────────────┘
                             ▼
              [ 2-of-2 Dual-Custody Quorum ]
                             ▼
          [ SHA-256 Merkle Root & Statement Hash ]
                             ▼
          [ Hedera Consensus Service Topic #0.0.10462941 ]
                             │
                             ▼ (Public Mirror Node Verification)
         [ Zero-Trust Verifier / Bazantic MCP Agent ]
```

---

## ⚡ Quick Start

### Prerequisites

- **Rust toolchain** (1.85+)
- **Node.js** (v20+ or v24 LTS)
- **pnpm** (v9+)

### 1. Clone & Setup Environment

```bash
git clone https://github.com/vimalsaraswat/ethonline2026.git
cd ethonline2026
cp .env.example .env
```

Your `.env` includes Hedera testnet topic credentials and Privy server wallet IDs.

### 2. Run Tests

```bash
cargo test --workspace
```

_(Runs all 72 cryptographic, consensus, and accounting invariance tests)_

### 3. Start Backend Server

```bash
cargo run -p sealed-books-server
```

Runs at `http://127.0.0.1:3000` with auto-migrated SQLite storage and pre-seeded demo accounts.

### 4. Run the Web App

```bash
pnpm install
pnpm --filter @sealed-books/web dev
```

Open [http://localhost:5173](http://localhost:5173) in your browser.

### 5. Run Native Desktop App (Tauri v2)

```bash
pnpm --filter @sealed-books/desktop dev
```

---

## 🧪 Testing the Tamper Protection

1. In the app, select the sealed August 2026 period (`2026-08-01` → `2026-08-31`).
2. Go to **Audit & Consensus** and click **Verify Ledger Integrity** → verified against Hedera mirror node (all green).
3. Scroll down to **Integrity Testing Sandbox** and click **Open Tamper Simulator**.
4. Click **Inject Database Mutation** to silently alter an entry directly in SQLite storage.
5. Click **Run Verification Engine Now** → The engine instantly flags the fraud:
   ```text
   STATUS: ✕ FRAUD ALERT: Tampered ledger entry detected.
   Offending Entry: ent_2026_08_005
   Local Merkle Root:     0x8a1f... (Mismatch)
   Hedera Consensus Root: 0xca71... (Anchored on Topic #0.0.10462941)
   ```

---

## 🤖 Bazantic MCP & Recipe Integration

The `/api/periods/{id}/verify` endpoint is agentified as an MCP tool:

- **MCP Tool**: `verify_period_seal`
- **Recipe**: Given a period ID, queries the endpoint, pulls the canonical statement from the Hedera Mirror Node, and returns whether the ledger is intact or which transaction was altered.

---

## 📦 Project Structure

```
├── crates/
│   ├── core/           # Double-entry ledger logic, SHA-256 Merkle tree & statement serialization
│   └── hedera/         # Hedera HCS publisher & mirror node verification client
├── apps/
│   ├── server/         # Rust Axum backend, SQLite database, Privy wallet signer & auth
│   ├── web/            # Vite + React 19 web app (Tailwind CSS v4 + Shadcn UI)
│   └── desktop/        # Tauri v2 cross-platform desktop application
└── packages/
    └── ui/             # Shared UI components, domain modals, and ledger state
```

---

## 📄 License

MIT License.
