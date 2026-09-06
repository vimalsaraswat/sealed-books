# Sealed Books

A small accounting app that can prove its records haven't been quietly changed.

## The problem

Anyone can open their accounting software and edit an old entry. No trace, no
proof it happened, no proof it didn't. Auditors, banks, and even the business
owner just have to trust the numbers as they look today.

## What it does

When you close a month, or send an invoice, the app takes a fingerprint
(a hash) of that record and publishes it to Hedera — a network no single
person, including us, controls. That timestamp can't be moved or faked later.

Later, anyone can hit **Verify**. The app checks today's records against what
was actually published back then.

- Match → nothing was touched since.
- Mismatch → it points at exactly what changed.

## From the user's side

**Closing the books:** you do your normal month-end, click "Close Period," done. Nothing changes in how you work day to day. Months later, if an auditor, a bank, or you need proof nothing was edited, hit Verify and get a plain yes or no.

**Sending an invoice:** you send it like always. The client gets a simple
link, signs in with just their email — no wallet, no crypto talk on their
screen — and taps Accept or Dispute. That's their whole part. Now the invoice
has proof both sides agreed, not just one side's word.

## What this is not

- Not a wallet app. Nobody using it sees a seed phrase, a token, or the word
  "blockchain."
- Not invoice financing — yet. That's a real next step, not part of this.
- Not a full accounting suite. Just journal entries, invoices, and proof they
  weren't altered after the fact.

## Why blockchain, not just a database

Because a company keeping its own proof on its own server can also edit that
proof. The point is a clock and a record nobody involved can quietly rewind.

## Built with

- **Tauri + Rust + SQLite** — the actual accounting app; works like any
  normal desktop app
- **Hedera Consensus Service** — the tamper-proof timestamp
- **Privy** — lets a client sign off with just an email, no wallet knowledge
  needed
