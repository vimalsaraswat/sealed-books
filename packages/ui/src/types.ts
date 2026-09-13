export type AccountType =
  | "asset"
  | "liability"
  | "equity"
  | "revenue"
  | "expense";

export interface Account {
  id: string;
  code: string;
  name: string;
  account_type: AccountType;
}

export interface Period {
  id: string;
  entity: string;
  start_date: string;
  end_date: string;
  status: "open" | "sealed" | "audited";
  organization_id?: string;
  created_at?: string;
}

export interface EntryLine {
  id: string;
  account_id: string;
  amount_minor: number;
  direction: "debit" | "credit";
  description?: string | null;
}

export interface Entry {
  id: string;
  period_id: string;
  date: string;
  description: string;
  hash: string;
  sequence_number?: number;
  lines: EntryLine[];
}

export interface CreateLineInput {
  account_id: string;
  amount_minor: number;
  direction: "debit" | "credit";
  description?: string;
}

export interface CreateEntryInput {
  date: string;
  description: string;
  lines: CreateLineInput[];
}

export interface CreatePeriodInput {
  id?: string;
  entity: string;
  start_date: string;
  end_date: string;
  organization_id?: string;
}

export interface CreateAccountInput {
  code: string;
  name: string;
  account_type: AccountType;
  organization_id?: string;
}

/**
 * Canonical seal statement 1:1 with crates/core/src/types.rs SealStatement
 */
export interface SealStatement {
  version: number;
  entity: string;
  period_start: string;
  period_end: string;
  entry_count: number;
  total_debits_minor: number;
  total_credits_minor: number;
  ledger_root: string;
}

/** Backward compatibility alias */
export type PeriodStatement = SealStatement;

/**
 * 1:1 with apps/server/src/api/periods.rs StatementResponse
 */
export interface StatementResponse {
  statement: SealStatement;
  statement_hash_hex: string;
  ledger_root_hex: string;
  human_readable: string;
}

/**
 * 1:1 with apps/server/src/api/seal.rs ProposeResponse
 */
export interface ProposeResponse {
  period_id: string;
  statement: SealStatement;
  statement_hash: string;
  human_readable: string;
}

/**
 * 1:1 with apps/server/src/api/seal.rs ApproveResponse
 */
export interface ApproveResponse {
  period_id: string;
  approver_pubkey: string;
  approvals_collected: number;
  quorum_met: boolean;
}

/**
 * 1:1 with apps/server/src/api/seal.rs PublishResponse
 */
export interface PublishResponse {
  status: string;
  topic_id: string;
  sequence_number: number;
  consensus_timestamp: string;
  transaction_id: string;
  hashscan_url: string;
  payload_bytes_len: number;
}

/**
 * 1:1 with apps/server/src/api/seal.rs SealStateResponse
 */
export interface SealStateResponse {
  period_id: string;
  period_status: string;
  seal: SealRecord | null;
  approvals_collected: number;
  quorum_met: boolean;
}

export interface SealRecord {
  id: string;
  period_id: string;
  root: string;
  statement_hash: string;
  topic_id?: string | null;
  sequence_number?: number | null;
  consensus_timestamp?: string | null;
  approver_1_pubkey?: string | null;
  approver_1_sig?: string | null;
  approver_2_pubkey?: string | null;
  approver_2_sig?: string | null;
  dispatch_status?:
    | "draft"
    | "pending_auditor"
    | "rejected"
    | "sealed"
    | string
    | null;
  auditor_id?: string | null;
  auditor_notes?: string | null;
  created_at: string;
}

export interface VerifiedApprover {
  compressed_pubkey: string;
  eth_address: string;
  signature_valid: boolean;
  index?: number;
  address?: string;
  pubkey?: string;
  signature?: string;
  is_valid?: boolean;
}

export interface SealStatementSummary {
  entity: string;
  period_start: string;
  period_end: string;
  entry_count: number;
  total_debits_minor: number;
  total_credits_minor: number;
  ledger_root: string;
  statement_hash?: string;
}

export interface DatabaseSummary {
  entry_count: number;
  total_debits_minor: number;
  total_credits_minor: number;
  ledger_root: string;
}

export interface DiscrepancyReport {
  kind?: string;
  field?: string;
  database_value?: string;
  on_chain_value?: string;
  description?: string;
  offending_entry_id?: string;
  message?: string;
  expected_hash?: string;
  actual_hash?: string;
}

export interface VerificationReport {
  period_id: string;
  status:
    | "verified"
    | "tampered"
    | "signature_invalid"
    | "pending_consensus"
    | "unsealed"
    | string;
  is_intact: boolean;
  topic_id: string;
  sequence_number: number;
  consensus_timestamp: string;
  on_chain_statement?: SealStatementSummary;
  database_summary?: DatabaseSummary;
  approver_1?: VerifiedApprover;
  approver_2?: VerifiedApprover;
  approvers?: VerifiedApprover[];
  discrepancy?: DiscrepancyReport | null;
  human_readable_statement?: string;
  hashscan_url?: string;
  // UI aliases
  verified?: boolean;
  running_hash?: string;
  ledger_merkle_root?: string;
  consensus_merkle_root?: string;
  offending_entry_id?: string;
  reason?: string;
}

export type UserRole = "owner" | "admin" | "controller" | "auditor" | "staff";

export interface User {
  id: string;
  email: string;
  name: string;
  pubkey?: string;
  eth_address: string;
  wallet_id?: string;
  role: UserRole;
  is_active: boolean;
  created_at: string;
}

export interface Organization {
  id: string;
  name: string;
  base_currency: string;
  created_at: string;
}

export interface OrgMember {
  id: string;
  organization_id: string;
  user_id: string;
  name: string;
  email: string;
  eth_address: string;
  role: UserRole;
  status?: string;
  created_at: string;
}

export interface UserOrgSummary {
  id: string;
  name: string;
  role: UserRole;
}

export interface LoginResponse {
  token: string;
  user: User;
  organization: Organization;
  active_organization: Organization;
  role: UserRole;
  available_organizations: UserOrgSummary[];
}

export interface PendingAuditItem {
  seal_id: string;
  period_id: string;
  entity: string;
  start_date: string;
  end_date: string;
  root: string;
  statement_hash: string;
  approver_1_pubkey?: string | null;
  approver_1_sig?: string | null;
  created_at: string;
  organization_id: string;
  organization_name: string;
}

export interface ApiClient {
  getMe(): Promise<LoginResponse>;
  sendOtp(
    email: string,
  ): Promise<{ status: string; email: string; is_new_user: boolean }>;
  verifyOtp(payload: {
    email: string;
    code: string;
    name?: string;
    organization_name?: string;
  }): Promise<LoginResponse>;
  login(
    credential: string | { email?: string; userId?: string; token?: string },
  ): Promise<LoginResponse>;
  register(input: {
    email: string;
    name: string;
    organization_name?: string;
  }): Promise<LoginResponse>;
  logout(): Promise<void>;
  switchOrg(organizationId: string): Promise<LoginResponse>;
  getOrganizations(): Promise<Organization[]>;
  createOrganization(
    name: string,
    baseCurrency?: string,
  ): Promise<Organization>;
  getOrgMembers(orgId: string): Promise<OrgMember[]>;
  inviteMember(
    orgId: string,
    email: string,
    role: string,
    name?: string,
  ): Promise<OrgMember>;
  getAuditorPending(): Promise<PendingAuditItem[]>;
  getAccounts(): Promise<Account[]>;
  createAccount(payload: CreateAccountInput): Promise<Account>;
  getPeriods(): Promise<Period[]>;
  getPeriod(id: string): Promise<Period>;
  createPeriod(payload: CreatePeriodInput): Promise<Period>;
  getPeriodEntries(id: string): Promise<Entry[]>;
  createEntry(periodId: string, payload: CreateEntryInput): Promise<Entry>;
  getPeriodStatement(id: string): Promise<StatementResponse>;
  getPeriodSeal(id: string): Promise<SealRecord | null>;
  proposeSeal(id: string): Promise<ProposeResponse>;
  approveSeal(
    id: string,
    payload?: { approver_pubkey?: string; signature?: string; wallet_id?: string },
  ): Promise<ApproveResponse>;
  dispatchSeal(periodId: string, auditorId: string): Promise<SealRecord>;
  rejectSeal(periodId: string, notes: string): Promise<SealRecord>;
  publishSeal(id: string): Promise<SealRecord>;
  verifyPeriod(id: string): Promise<VerificationReport>;
  simulateTamper(
    periodId: string,
    entryId: string,
    newDescription: string,
  ): Promise<{ status: string; entry_id: string; message: string }>;
}

export type NavTab = "overview" | "ledger" | "financials" | "audit" | "team";
