import type {
  ApiClient,
  Account,
  Period,
  Entry,
  StatementResponse,
  ProposeResponse,
  SealRecord,
  VerificationReport,
  CreateEntryInput,
  CreatePeriodInput,
  CreateAccountInput,
  LoginResponse,
  PendingAuditItem,
  OrgMember,
  Organization,
} from "@sealed-books/ui";

const DEFAULT_API_BASE =
  (import.meta.env.VITE_API_URL as string | undefined) || "";

export function getApiBaseUrl(): string {
  if (typeof window !== "undefined") {
    return localStorage.getItem("sealed_books_api_url") || DEFAULT_API_BASE;
  }
  return DEFAULT_API_BASE;
}

export function setApiBaseUrl(url: string) {
  if (typeof window !== "undefined") {
    if (url) {
      localStorage.setItem("sealed_books_api_url", url);
    } else {
      localStorage.removeItem("sealed_books_api_url");
    }
  }
}

let currentSessionToken: string =
  typeof window !== "undefined"
    ? localStorage.getItem("sealed_books_token") || ""
    : "";

export function setSessionToken(token: string) {
  currentSessionToken = token;
  if (typeof window !== "undefined") {
    if (token) {
      localStorage.setItem("sealed_books_token", token);
    } else {
      localStorage.removeItem("sealed_books_token");
    }
  }
}

export function getSessionToken(): string {
  return currentSessionToken;
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(currentSessionToken
      ? { Authorization: `Bearer ${currentSessionToken}` }
      : {}),
    ...(options?.headers as Record<string, string>),
  };

  const baseUrl = getApiBaseUrl().replace(/\/+$/, "");
  const targetUrl = `${baseUrl}${path}`;

  const res = await fetch(targetUrl, {
    ...options,
    headers,
  });

  if (!res.ok) {
    let errorDetail = `Request failed: ${res.status} ${res.statusText}`;
    try {
      const data = await res.json();
      if (data?.error) errorDetail = data.error;
    } catch {
      // ignore json parse error
    }
    throw new Error(errorDetail);
  }

  return res.json();
}

export const api: ApiClient = {
  // Authentication & Tenancy
  async getMe(): Promise<LoginResponse> {
    return request<LoginResponse>("/api/auth/me");
  },

  async sendOtp(
    email: string,
  ): Promise<{ status: string; email: string; is_new_user: boolean }> {
    return request<{ status: string; email: string; is_new_user: boolean }>(
      "/api/auth/otp/send",
      {
        method: "POST",
        body: JSON.stringify({ email }),
      },
    );
  },

  async verifyOtp(payload: {
    email: string;
    code: string;
    name?: string;
    organization_name?: string;
  }): Promise<LoginResponse> {
    const res = await request<LoginResponse>("/api/auth/otp/verify", {
      method: "POST",
      body: JSON.stringify(payload),
    });
    setSessionToken(res.token);
    return res;
  },

  async login(
    param: string | { email?: string; userId?: string; token?: string },
  ): Promise<LoginResponse> {
    const payload =
      typeof param === "string"
        ? param.includes("@")
          ? { email: param }
          : { user_id: param }
        : { email: param.email, user_id: param.userId, token: param.token };

    const res = await request<LoginResponse>("/api/auth/login", {
      method: "POST",
      body: JSON.stringify(payload),
    });
    setSessionToken(res.token);
    return res;
  },

  async register(input: {
    email: string;
    name: string;
    organization_name?: string;
  }): Promise<LoginResponse> {
    const res = await request<LoginResponse>("/api/auth/register", {
      method: "POST",
      body: JSON.stringify(input),
    });
    setSessionToken(res.token);
    return res;
  },

  async logout(): Promise<void> {
    try {
      await request<void>("/api/auth/logout", { method: "POST" });
    } catch {
      // ignore
    }
    setSessionToken("");
  },

  async switchOrg(organizationId: string): Promise<LoginResponse> {
    const res = await request<LoginResponse>("/api/auth/switch-org", {
      method: "POST",
      body: JSON.stringify({ organization_id: organizationId }),
    });
    return res;
  },

  async getOrganizations(): Promise<Organization[]> {
    return request<Organization[]>("/api/organizations");
  },

  async createOrganization(
    name: string,
    baseCurrency: string = "USD",
  ): Promise<Organization> {
    return request<Organization>("/api/organizations", {
      method: "POST",
      body: JSON.stringify({ name, base_currency: baseCurrency }),
    });
  },

  async getOrgMembers(orgId: string): Promise<OrgMember[]> {
    return request<OrgMember[]>(`/api/organizations/${orgId}/members`);
  },

  async inviteMember(
    orgId: string,
    email: string,
    role: string,
    name?: string,
  ): Promise<OrgMember> {
    return request<OrgMember>(`/api/organizations/${orgId}/invites`, {
      method: "POST",
      body: JSON.stringify({ email, role, name }),
    });
  },

  // Auditor Queue
  async getAuditorPending(): Promise<PendingAuditItem[]> {
    return request<PendingAuditItem[]>("/api/auditor/pending");
  },

  // General Ledger
  async getAccounts(): Promise<Account[]> {
    return request<Account[]>("/api/accounts");
  },

  async createAccount(payload: CreateAccountInput): Promise<Account> {
    return request<Account>("/api/accounts", {
      method: "POST",
      body: JSON.stringify(payload),
    });
  },

  async getPeriods(): Promise<Period[]> {
    return request<Period[]>("/api/periods");
  },

  async getPeriod(id: string): Promise<Period> {
    return request<Period>(`/api/periods/${id}`);
  },

  async createPeriod(payload: CreatePeriodInput): Promise<Period> {
    return request<Period>("/api/periods", {
      method: "POST",
      body: JSON.stringify(payload),
    });
  },

  async getPeriodEntries(id: string): Promise<Entry[]> {
    return request<Entry[]>(`/api/periods/${id}/entries`);
  },

  async createEntry(
    periodId: string,
    payload: CreateEntryInput,
  ): Promise<Entry> {
    return request<Entry>(`/api/periods/${periodId}/entries`, {
      method: "POST",
      body: JSON.stringify(payload),
    });
  },

  async getPeriodStatement(id: string): Promise<StatementResponse> {
    return request<StatementResponse>(`/api/periods/${id}/statement`);
  },

  async getPeriodSeal(id: string): Promise<SealRecord | null> {
    try {
      return await request<SealRecord>(`/api/periods/${id}/seal`);
    } catch {
      return null;
    }
  },

  // Seal & Close Workflows
  async proposeSeal(id: string): Promise<ProposeResponse> {
    return request<ProposeResponse>(`/api/periods/${id}/seal/propose`, {
      method: "POST",
    });
  },

  async approveSeal(
    id: string,
    payload: { approver_pubkey: string; signature: string },
  ): Promise<{
    period_id: string;
    approver_pubkey: string;
    approvals_collected: number;
    quorum_met: boolean;
  }> {
    return request<{
      period_id: string;
      approver_pubkey: string;
      approvals_collected: number;
      quorum_met: boolean;
    }>(`/api/periods/${id}/seal/approve`, {
      method: "POST",
      body: JSON.stringify(payload),
    });
  },

  async dispatchSeal(periodId: string, auditorId: string): Promise<SealRecord> {
    return request<SealRecord>(`/api/periods/${periodId}/seal/dispatch`, {
      method: "POST",
      body: JSON.stringify({ auditor_id: auditorId }),
    });
  },

  async rejectSeal(periodId: string, notes: string): Promise<SealRecord> {
    return request<SealRecord>(`/api/periods/${periodId}/seal/reject`, {
      method: "POST",
      body: JSON.stringify({ notes }),
    });
  },

  async publishSeal(id: string): Promise<SealRecord> {
    return request<SealRecord>(`/api/periods/${id}/seal/publish`, {
      method: "POST",
    });
  },

  // Verification & Tamper Testing
  async verifyPeriod(id: string): Promise<VerificationReport> {
    return request<VerificationReport>(`/api/periods/${id}/verify`);
  },

  async simulateTamper(
    periodId: string,
    entryId: string,
    newDescription: string,
  ): Promise<{ status: string; entry_id: string; message: string }> {
    return request<{ status: string; entry_id: string; message: string }>(
      `/api/periods/${periodId}/simulate-tamper`,
      {
        method: "POST",
        body: JSON.stringify({
          entry_id: entryId,
          new_description: newDescription,
        }),
      },
    );
  },
};
