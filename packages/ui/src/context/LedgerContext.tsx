import {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  type ReactNode,
} from "react";
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
  NavTab,
} from "../types";
import { useAuth } from "./AuthContext";

export interface LedgerContextValue {
  periods: Period[];
  selectedPeriodId: string | null;
  currentPeriod: Period | null;
  accounts: Account[];
  entries: Entry[];
  statement: StatementResponse | null;
  seal: SealRecord | null;
  proposalStatement: ProposeResponse | null;
  verificationReport: VerificationReport | null;
  isVerifying: boolean;
  highlightedEntryId: string | null;
  isLoadingLedger: boolean;
  activeTab: NavTab;
  setActiveTab: (tab: NavTab) => void;
  setSelectedPeriodId: (id: string | null) => void;
  setHighlightedEntryId: (id: string | null) => void;
  loadPeriodData: (periodId: string) => Promise<void>;
  loadAccountsAndPeriods: () => Promise<void>;
  handlePeriodChange: (newId: string) => Promise<void>;
  handleOpenCloseDialog: () => Promise<ProposeResponse | null>;
  handleApproveSeal: (data?: {
    approver_pubkey?: string;
    signature?: string;
    wallet_id?: string;
  }) => Promise<void>;
  handleDispatchSeal: (auditorId: string) => Promise<void>;
  handleAuditorReject: (notes: string) => Promise<void>;
  handleAuditorApproveAndPublish: (data?: {
    approver_pubkey?: string;
    signature?: string;
    wallet_id?: string;
  }) => Promise<SealRecord>;
  handlePublishSeal: () => Promise<SealRecord>;
  handleVerify: () => Promise<VerificationReport | null>;
  handleLocateOffendingEntry: (entryId: string) => void;
  handleCreateEntry: (entry: CreateEntryInput) => Promise<void>;
  handleCreatePeriod: (periodInput: CreatePeriodInput) => Promise<void>;
  handleCreateAccount: (accountInput: CreateAccountInput) => Promise<void>;
  handleSimulateTamper: (entryId: string, newDesc: string) => Promise<void>;
}

const LedgerContext = createContext<LedgerContextValue | null>(null);

export interface LedgerProviderProps {
  api: ApiClient;
  children: ReactNode;
}

export function LedgerProvider({ api, children }: LedgerProviderProps) {
  const { activeOrg, refreshPendingAudits } = useAuth();

  const [periods, setPeriods] = useState<Period[]>([]);
  const [selectedPeriodId, setSelectedPeriodId] = useState<string | null>(null);
  const [currentPeriod, setCurrentPeriod] = useState<Period | null>(null);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [entries, setEntries] = useState<Entry[]>([]);
  const [statement, setStatement] = useState<StatementResponse | null>(null);
  const [seal, setSeal] = useState<SealRecord | null>(null);

  const [proposalStatement, setProposalStatement] =
    useState<ProposeResponse | null>(null);
  const [verificationReport, setVerificationReport] =
    useState<VerificationReport | null>(null);
  const [isVerifying, setIsVerifying] = useState(false);
  const [highlightedEntryId, setHighlightedEntryId] = useState<string | null>(
    null,
  );
  const [isLoadingLedger, setIsLoadingLedger] = useState(true);
  const [activeTab, setActiveTab] = useState<NavTab>("overview");

  // Load detailed data for a specific period
  const loadPeriodData = useCallback(
    async (periodId: string) => {
      try {
        const [periodData, entriesData, statementData, sealData] =
          await Promise.all([
            api.getPeriod(periodId),
            api.getPeriodEntries(periodId),
            api.getPeriodStatement(periodId).catch(() => null),
            api.getPeriodSeal(periodId).catch(() => null),
          ]);

        setCurrentPeriod(periodData);
        setEntries(entriesData);
        setStatement(statementData);
        setSeal(sealData);
      } catch (err) {
        console.error("Failed to load period data:", err);
      }
    },
    [api],
  );

  // Load accounts and periods for current organization
  const loadAccountsAndPeriods = useCallback(async () => {
    setIsLoadingLedger(true);
    try {
      const [accountsData, periodsData] = await Promise.all([
        api.getAccounts().catch(() => []),
        api.getPeriods().catch(() => []),
      ]);

      setAccounts(accountsData);
      setPeriods(periodsData);

      if (periodsData.length > 0) {
        const active =
          periodsData.find((p: Period) => p.status === "open") || periodsData[0];
        setSelectedPeriodId(active.id);
        await loadPeriodData(active.id);
      } else {
        setSelectedPeriodId(null);
        setCurrentPeriod(null);
        setEntries([]);
        setStatement(null);
        setSeal(null);
      }
    } catch (err) {
      console.error("Failed to load ledger accounts and periods:", err);
    } finally {
      setIsLoadingLedger(false);
    }
  }, [api, loadPeriodData]);

  // Reload data whenever active organization changes
  useEffect(() => {
    if (activeOrg) {
      loadAccountsAndPeriods();
    }
  }, [activeOrg, loadAccountsAndPeriods]);

  const handlePeriodChange = useCallback(
    async (newId: string) => {
      setSelectedPeriodId(newId);
      await loadPeriodData(newId);
    },
    [loadPeriodData],
  );

  const handleOpenCloseDialog = useCallback(async () => {
    if (!currentPeriod) return null;
    try {
      const res = await api.proposeSeal(currentPeriod.id);
      setProposalStatement(res);
      return res;
    } catch (err) {
      console.error("Failed to propose seal:", err);
      return null;
    }
  }, [api, currentPeriod]);

  const handleApproveSeal = useCallback(
    async (data?: {
      approver_pubkey?: string;
      signature?: string;
      wallet_id?: string;
    }) => {
      if (!currentPeriod) return;
      await api.approveSeal(currentPeriod.id, data);
      await loadPeriodData(currentPeriod.id);
    },
    [api, currentPeriod, loadPeriodData],
  );

  const handleDispatchSeal = useCallback(
    async (auditorId: string) => {
      if (!currentPeriod) return;
      await api.dispatchSeal(currentPeriod.id, auditorId);
      await loadPeriodData(currentPeriod.id);
      await refreshPendingAudits();
    },
    [api, currentPeriod, loadPeriodData, refreshPendingAudits],
  );

  const handleAuditorReject = useCallback(
    async (notes: string) => {
      if (!currentPeriod) return;
      await api.rejectSeal(currentPeriod.id, notes);
      await loadPeriodData(currentPeriod.id);
      await refreshPendingAudits();
    },
    [api, currentPeriod, loadPeriodData, refreshPendingAudits],
  );

  const handleAuditorApproveAndPublish = useCallback(
    async (data?: {
      approver_pubkey?: string;
      signature?: string;
      wallet_id?: string;
    }): Promise<SealRecord> => {
      if (!currentPeriod) throw new Error("No active period");
      await api.approveSeal(currentPeriod.id, data);
      const published = await api.publishSeal(currentPeriod.id);
      await loadPeriodData(currentPeriod.id);
      await refreshPendingAudits();
      return published;
    },
    [api, currentPeriod, loadPeriodData, refreshPendingAudits],
  );

  const handlePublishSeal = useCallback(async (): Promise<SealRecord> => {
    if (!currentPeriod) throw new Error("No active period");
    const published = await api.publishSeal(currentPeriod.id);
    await loadPeriodData(currentPeriod.id);
    return published;
  }, [api, currentPeriod, loadPeriodData]);

  const handleVerify = useCallback(async (): Promise<VerificationReport | null> => {
    if (!currentPeriod) return null;
    setIsVerifying(true);
    try {
      const report = await api.verifyPeriod(currentPeriod.id);
      setVerificationReport(report);
      return report;
    } catch (err: any) {
      console.error("Verification failed:", err);
      alert(err.message || "Verification request failed");
      return null;
    } finally {
      setIsVerifying(false);
    }
  }, [api, currentPeriod]);

  const handleLocateOffendingEntry = useCallback((entryId: string) => {
    setHighlightedEntryId(entryId);
    window.scrollTo({ top: 400, behavior: "smooth" });
  }, []);

  const handleCreateEntry = useCallback(
    async (entry: CreateEntryInput) => {
      if (!currentPeriod) return;
      await api.createEntry(currentPeriod.id, entry);
      await loadPeriodData(currentPeriod.id);
    },
    [api, currentPeriod, loadPeriodData],
  );

  const handleCreatePeriod = useCallback(
    async (periodInput: CreatePeriodInput) => {
      const created = await api.createPeriod(periodInput);
      const updatedPeriods = await api.getPeriods();
      setPeriods(updatedPeriods);
      setSelectedPeriodId(created.id);
      await loadPeriodData(created.id);
    },
    [api, loadPeriodData],
  );

  const handleCreateAccount = useCallback(
    async (accountInput: CreateAccountInput) => {
      await api.createAccount(accountInput);
      const updated = await api.getAccounts();
      setAccounts(updated);
    },
    [api],
  );

  const handleSimulateTamper = useCallback(
    async (entryId: string, newDesc: string) => {
      if (!currentPeriod) return;
      await api.simulateTamper(currentPeriod.id, entryId, newDesc);
      await loadPeriodData(currentPeriod.id);
    },
    [api, currentPeriod, loadPeriodData],
  );

  return (
    <LedgerContext.Provider
      value={{
        periods,
        selectedPeriodId,
        currentPeriod,
        accounts,
        entries,
        statement,
        seal,
        proposalStatement,
        verificationReport,
        isVerifying,
        highlightedEntryId,
        isLoadingLedger,
        activeTab,
        setActiveTab,
        setSelectedPeriodId,
        setHighlightedEntryId,
        loadPeriodData,
        loadAccountsAndPeriods,
        handlePeriodChange,
        handleOpenCloseDialog,
        handleApproveSeal,
        handleDispatchSeal,
        handleAuditorReject,
        handleAuditorApproveAndPublish,
        handlePublishSeal,
        handleVerify,
        handleLocateOffendingEntry,
        handleCreateEntry,
        handleCreatePeriod,
        handleCreateAccount,
        handleSimulateTamper,
      }}
    >
      {children}
    </LedgerContext.Provider>
  );
}

export function useLedger(): LedgerContextValue {
  const context = useContext(LedgerContext);
  if (!context) {
    throw new Error("useLedger must be used within a LedgerProvider");
  }
  return context;
}
