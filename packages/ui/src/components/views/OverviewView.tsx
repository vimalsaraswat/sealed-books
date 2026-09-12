import { useMemo } from "react";
import {
  Lock,
  FileSpreadsheet,
  Clock,
  AlertTriangle,
  ShieldCheck,
  ArrowUpRight,
  Plus,
  Scale,
  BookOpen,
  ArrowRight,
  Award,
  CheckCircle2,
  CircleDot,
  Hash,
  Activity,
  Layers,
} from "lucide-react";
import { Badge } from "../ui/badge";
import { Button } from "../ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "../ui/card";
import { formatCurrency } from "../../lib/utils";
import { useAuth } from "../../context/AuthContext";
import { useLedger } from "../../context/LedgerContext";
import { useModals } from "../../context/ModalContext";

export function OverviewView() {
  const { userRole } = useAuth();
  const {
    currentPeriod,
    entries,
    accounts,
    seal,
    isVerifying,
    setActiveTab,
    handleOpenCloseDialog,
    handleVerify,
  } = useLedger();

  const {
    setIsAddEntryOpen,
    setIsCloseModalOpen,
    setIsAuditorModalOpen,
    setIsVerifyModalOpen,
    setIsCertificateOpen,
  } = useModals();

  const isSealed = currentPeriod?.status === "sealed";
  const isAuditor = userRole === "auditor";
  const isPendingAuditor = seal?.dispatch_status === "pending_auditor";
  const isRejected = seal?.dispatch_status === "rejected";

  // Compute total debits and credits across all entries in the period
  const { totalDebits, totalCredits, isBalanced, variance } = useMemo(() => {
    let debits = 0;
    let credits = 0;
    for (const entry of entries) {
      for (const line of entry.lines) {
        if (line.direction === "debit") {
          debits += line.amount_minor;
        } else if (line.direction === "credit") {
          credits += line.amount_minor;
        }
      }
    }
    const diff = debits - credits;
    return {
      totalDebits: debits,
      totalCredits: credits,
      isBalanced: diff === 0,
      variance: diff,
    };
  }, [entries]);

  const recentEntries = useMemo(() => {
    return [...entries].slice(0, 5);
  }, [entries]);

  const accountMap = useMemo(() => {
    const map = new Map<string, string>();
    for (const acc of accounts) {
      map.set(acc.id, `${acc.code} ${acc.name}`);
    }
    return map;
  }, [accounts]);

  const handleProposeClose = async () => {
    const propStmt = await handleOpenCloseDialog();
    if (propStmt) {
      setIsCloseModalOpen(true);
    }
  };

  const handleRunVerification = async () => {
    const rep = await handleVerify();
    if (rep) {
      setIsVerifyModalOpen(true);
    }
  };

  const handleOpenCertificate = async () => {
    if (!seal) return;
    const rep = await handleVerify();
    if (rep) {
      setIsCertificateOpen(true);
    }
  };

  if (!currentPeriod) {
    return (
      <div className="py-20 text-center space-y-3">
        <div className="size-12 rounded-full bg-muted flex items-center justify-center mx-auto text-muted-foreground">
          <Layers className="size-6" />
        </div>
        <h3 className="text-base font-semibold text-foreground">
          No Accounting Period Active
        </h3>
        <p className="text-sm text-muted-foreground max-w-sm mx-auto">
          Please select or open an accounting period from the top navigation.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* =========================================================
          1. CONTEXTUAL PERIOD ACTION BANNER
          ========================================================= */}
      <Card
        className={`border shadow-xs transition-colors ${
          isSealed
            ? "border-emerald-500/30 bg-emerald-500/5 dark:bg-emerald-950/20"
            : isPendingAuditor
            ? "border-amber-500/30 bg-amber-500/5 dark:bg-amber-950/20"
            : "border-primary/20 bg-primary/5 dark:bg-primary/10"
        }`}
      >
        <CardContent className="p-5 sm:p-6">
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
            <div className="space-y-1">
              <div className="flex items-center gap-2.5 flex-wrap">
                <h2 className="text-lg font-bold tracking-tight text-foreground">
                  {currentPeriod.entity}
                </h2>
                <Badge variant={isSealed ? "sealed" : "default"}>
                  {isSealed ? (
                    <span className="flex items-center gap-1 font-mono text-xs">
                      <Lock className="size-3" /> SEALED
                    </span>
                  ) : (
                    <span className="flex items-center gap-1 font-mono text-xs">
                      <FileSpreadsheet className="size-3" /> OPEN
                    </span>
                  )}
                </Badge>
                {isPendingAuditor && !isSealed && (
                  <Badge
                    variant="outline"
                    className="border-amber-500/30 text-amber-600 dark:text-amber-400 bg-amber-500/10 gap-1 font-mono text-xs"
                  >
                    <Clock className="size-3" />
                    <span>AUDIT IN PROGRESS</span>
                  </Badge>
                )}
                {isRejected && !isSealed && (
                  <Badge
                    variant="outline"
                    className="border-destructive/30 text-destructive bg-destructive/10 gap-1 font-mono text-xs"
                  >
                    <AlertTriangle className="size-3" />
                    <span>CHANGES REQUESTED</span>
                  </Badge>
                )}
              </div>
              <p className="text-xs text-muted-foreground font-mono">
                Period: {currentPeriod.start_date} to {currentPeriod.end_date} • ID:{" "}
                {currentPeriod.id}
              </p>
            </div>

            {/* Contextual Action Buttons */}
            <div className="flex items-center gap-2.5 flex-wrap">
              {isSealed ? (
                <>
                  {seal?.consensus_timestamp && (
                    <a
                      href={`https://hashscan.io/testnet/transaction/${seal.consensus_timestamp}`}
                      target="_blank"
                      rel="noreferrer"
                      className="inline-flex items-center gap-1.5 text-xs text-primary hover:underline bg-background border border-primary/20 px-3 py-1.5 rounded-md font-mono h-9 shadow-xs"
                    >
                      <span>HCS #{seal.sequence_number || 1}</span>
                      <ArrowUpRight className="size-3.5" />
                    </a>
                  )}
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleOpenCertificate}
                    className="text-xs gap-1.5 h-9"
                  >
                    <Award className="size-3.5 text-primary" />
                    <span>Certificate</span>
                  </Button>
                  <Button
                    variant="default"
                    size="sm"
                    onClick={handleRunVerification}
                    disabled={isVerifying}
                    className="text-xs gap-1.5 h-9 bg-emerald-600 hover:bg-emerald-700 text-white dark:bg-emerald-600 dark:hover:bg-emerald-500"
                  >
                    <ShieldCheck className="size-4" />
                    <span>{isVerifying ? "Verifying..." : "Verify on Hedera"}</span>
                  </Button>
                </>
              ) : isPendingAuditor ? (
                isAuditor ? (
                  <Button
                    variant="default"
                    size="sm"
                    onClick={() => setIsAuditorModalOpen(true)}
                    className="text-xs gap-1.5 h-9 bg-emerald-600 hover:bg-emerald-700 text-white animate-pulse"
                  >
                    <ShieldCheck className="size-4" />
                    <span>Review & Co-Sign Period</span>
                  </Button>
                ) : (
                  <div className="flex items-center gap-2 text-xs text-amber-600 dark:text-amber-400 font-mono bg-amber-500/10 border border-amber-500/20 px-3 py-1.5 rounded-md">
                    <Clock className="size-3.5" />
                    <span>Awaiting Auditor Signature</span>
                  </div>
                )
              ) : (
                <>
                  {!isAuditor && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setIsAddEntryOpen(true)}
                      className="text-xs gap-1.5 h-9"
                    >
                      <Plus className="size-3.5" />
                      <span>New Journal Entry</span>
                    </Button>
                  )}
                  {!isAuditor && isBalanced && entries.length > 0 && (
                    <Button
                      variant="default"
                      size="sm"
                      onClick={handleProposeClose}
                      className="text-xs gap-1.5 h-9"
                    >
                      <Lock className="size-3.5" />
                      <span>Propose Period Close</span>
                    </Button>
                  )}
                </>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* =========================================================
          2. KPI METRIC STRIP (Pencil & Paper Pattern)
          ========================================================= */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Total Debits */}
        <Card className="border-border shadow-xs">
          <CardHeader className="p-4 pb-1">
            <CardDescription className="text-xs font-medium uppercase font-mono tracking-wider">
              Total Debits
            </CardDescription>
            <CardTitle className="text-2xl font-bold font-mono tracking-tight text-foreground">
              {formatCurrency(totalDebits)}
            </CardTitle>
          </CardHeader>
          <CardContent className="p-4 pt-1">
            <p className="text-[11px] text-muted-foreground font-mono">
              Sum of debit postings
            </p>
          </CardContent>
        </Card>

        {/* Total Credits */}
        <Card className="border-border shadow-xs">
          <CardHeader className="p-4 pb-1">
            <CardDescription className="text-xs font-medium uppercase font-mono tracking-wider">
              Total Credits
            </CardDescription>
            <CardTitle className="text-2xl font-bold font-mono tracking-tight text-foreground">
              {formatCurrency(totalCredits)}
            </CardTitle>
          </CardHeader>
          <CardContent className="p-4 pt-1">
            <p className="text-[11px] text-muted-foreground font-mono">
              Sum of credit postings
            </p>
          </CardContent>
        </Card>

        {/* Balance Equilibrium */}
        <Card className="border-border shadow-xs">
          <CardHeader className="p-4 pb-1">
            <CardDescription className="text-xs font-medium uppercase font-mono tracking-wider">
              Equilibrium Status
            </CardDescription>
            <div className="flex items-center gap-2 pt-0.5">
              {isBalanced ? (
                <div className="flex items-center gap-1.5 text-emerald-600 dark:text-emerald-400 font-bold text-lg font-mono">
                  <CheckCircle2 className="size-5" />
                  <span>BALANCED</span>
                </div>
              ) : (
                <div className="flex items-center gap-1.5 text-destructive font-bold text-lg font-mono">
                  <AlertTriangle className="size-5" />
                  <span>{formatCurrency(Math.abs(variance))}</span>
                </div>
              )}
            </div>
          </CardHeader>
          <CardContent className="p-4 pt-1">
            <p className="text-[11px] text-muted-foreground font-mono">
              {isBalanced
                ? "Debits == Credits (Zero Variance)"
                : "Ledger out of balance"}
            </p>
          </CardContent>
        </Card>

        {/* Transaction Volume */}
        <Card className="border-border shadow-xs">
          <CardHeader className="p-4 pb-1">
            <CardDescription className="text-xs font-medium uppercase font-mono tracking-wider">
              Transactions & Accounts
            </CardDescription>
            <CardTitle className="text-2xl font-bold font-mono tracking-tight text-foreground">
              {entries.length}{" "}
              <span className="text-xs font-normal text-muted-foreground">entries</span>
            </CardTitle>
          </CardHeader>
          <CardContent className="p-4 pt-1">
            <p className="text-[11px] text-muted-foreground font-mono">
              Across {accounts.length} chart accounts
            </p>
          </CardContent>
        </Card>
      </div>

      {/* =========================================================
          3. PERIOD MILESTONE ATTESTATION STEPPER
          ========================================================= */}
      <Card className="border-border shadow-xs">
        <CardHeader className="p-5 pb-3">
          <CardTitle className="text-sm font-semibold text-foreground flex items-center gap-2">
            <Activity className="size-4 text-primary" />
            <span>Cryptographic Close Lifecycle</span>
          </CardTitle>
          <CardDescription className="text-xs text-muted-foreground">
            Multi-party attestation progress for {currentPeriod.id}
          </CardDescription>
        </CardHeader>
        <CardContent className="p-5 pt-1">
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
            {/* Step 1 */}
            <div className="p-3 rounded-lg border border-border/60 bg-muted/20 flex items-start gap-3">
              <CheckCircle2 className="size-5 text-emerald-500 shrink-0 mt-0.5" />
              <div className="space-y-0.5">
                <p className="text-xs font-semibold text-foreground">1. Period Setup</p>
                <p className="text-[11px] text-muted-foreground">
                  {currentPeriod.start_date} to {currentPeriod.end_date}
                </p>
              </div>
            </div>

            {/* Step 2 */}
            <div className="p-3 rounded-lg border border-border/60 bg-muted/20 flex items-start gap-3">
              {entries.length > 0 ? (
                <CheckCircle2 className="size-5 text-emerald-500 shrink-0 mt-0.5" />
              ) : (
                <CircleDot className="size-5 text-amber-500 shrink-0 mt-0.5" />
              )}
              <div className="space-y-0.5">
                <p className="text-xs font-semibold text-foreground">2. Post Entries</p>
                <p className="text-[11px] text-muted-foreground">
                  {entries.length} balanced transactions
                </p>
              </div>
            </div>

            {/* Step 3 */}
            <div className="p-3 rounded-lg border border-border/60 bg-muted/20 flex items-start gap-3">
              {isSealed || isPendingAuditor ? (
                <CheckCircle2 className="size-5 text-emerald-500 shrink-0 mt-0.5" />
              ) : (
                <CircleDot className="size-5 text-muted-foreground shrink-0 mt-0.5" />
              )}
              <div className="space-y-0.5">
                <p className="text-xs font-semibold text-foreground">3. Controller Close</p>
                <p className="text-[11px] text-muted-foreground">
                  {isSealed || isPendingAuditor ? "Approver 1 signed" : "Pending proposal"}
                </p>
              </div>
            </div>

            {/* Step 4 */}
            <div className="p-3 rounded-lg border border-border/60 bg-muted/20 flex items-start gap-3">
              {isSealed ? (
                <CheckCircle2 className="size-5 text-emerald-500 shrink-0 mt-0.5" />
              ) : isPendingAuditor ? (
                <Clock className="size-5 text-amber-500 shrink-0 mt-0.5" />
              ) : (
                <CircleDot className="size-5 text-muted-foreground shrink-0 mt-0.5" />
              )}
              <div className="space-y-0.5">
                <p className="text-xs font-semibold text-foreground">4. Hedera Seal</p>
                <p className="text-[11px] text-muted-foreground">
                  {isSealed ? "Anchored to HCS" : "Requires 2-of-2"}
                </p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* =========================================================
          4. RECENT ACTIVITY & QUICK ENTRY DRILL-DOWN
          ========================================================= */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left 2 Cols: Recent Transactions */}
        <div className="lg:col-span-2 space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-semibold text-foreground flex items-center gap-2">
              <BookOpen className="size-4 text-primary" />
              <span>Recent Journal Transactions</span>
            </h3>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setActiveTab("ledger")}
              className="text-xs gap-1 text-primary hover:text-primary hover:bg-primary/10"
            >
              <span>View All in General Ledger</span>
              <ArrowRight className="size-3" />
            </Button>
          </div>

          <div className="rounded-lg border border-border bg-card overflow-hidden shadow-xs divide-y divide-border/60">
            {recentEntries.length === 0 ? (
              <div className="p-8 text-center text-xs text-muted-foreground">
                No entries recorded for this period yet.
              </div>
            ) : (
              recentEntries.map((entry) => {
                const totalDebitMinor = entry.lines
                  .filter((l) => l.direction === "debit")
                  .reduce((sum, l) => sum + l.amount_minor, 0);

                return (
                  <div
                    key={entry.id}
                    onClick={() => setActiveTab("ledger")}
                    className="p-3.5 hover:bg-muted/40 transition-colors cursor-pointer flex items-center justify-between gap-4"
                  >
                    <div className="min-w-0 space-y-1">
                      <div className="flex items-center gap-2">
                        <span className="text-xs font-bold font-mono text-foreground">
                          {entry.date}
                        </span>
                        <span className="text-[11px] font-mono text-muted-foreground flex items-center gap-0.5">
                          <Hash className="size-2.5" />
                          {entry.id}
                        </span>
                      </div>
                      <p className="text-xs text-foreground font-medium truncate max-w-md">
                        {entry.description}
                      </p>
                      <div className="flex items-center gap-2 text-[10px] text-muted-foreground font-mono truncate">
                        {entry.lines.slice(0, 2).map((l, i) => (
                          <span key={i} className="truncate">
                            {accountMap.get(l.account_id) || "Account"} ({l.direction})
                          </span>
                        ))}
                        {entry.lines.length > 2 && (
                          <span>+{entry.lines.length - 2} more lines</span>
                        )}
                      </div>
                    </div>

                    <div className="text-right shrink-0">
                      <span className="text-xs font-bold font-mono text-foreground">
                        {formatCurrency(totalDebitMinor)}
                      </span>
                      <p className="text-[10px] text-muted-foreground font-mono">balanced</p>
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>

        {/* Right Col: Quick Financials & Consensus Links */}
        <div className="space-y-4">
          {/* Financial Statements Card */}
          <Card className="border-border shadow-xs">
            <CardHeader className="p-4 pb-2">
              <CardTitle className="text-xs font-semibold uppercase font-mono tracking-wider flex items-center gap-1.5">
                <Scale className="size-3.5 text-primary" />
                <span>Financial Statements</span>
              </CardTitle>
            </CardHeader>
            <CardContent className="p-4 pt-1 space-y-3">
              <p className="text-xs text-muted-foreground">
                Examine debit and credit balances across active asset, liability, and equity accounts.
              </p>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setActiveTab("financials")}
                className="w-full text-xs justify-between"
              >
                <span>Open Trial Balance & Accounts</span>
                <ArrowRight className="size-3" />
              </Button>
            </CardContent>
          </Card>

          {/* Hedera Consensus Card */}
          <Card className="border-border shadow-xs">
            <CardHeader className="p-4 pb-2">
              <CardTitle className="text-xs font-semibold uppercase font-mono tracking-wider flex items-center gap-1.5">
                <ShieldCheck className="size-3.5 text-primary" />
                <span>Hedera Consensus Anchor</span>
              </CardTitle>
            </CardHeader>
            <CardContent className="p-4 pt-1 space-y-3">
              <div className="p-2 rounded bg-muted/50 font-mono text-[11px] space-y-1">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Topic:</span>
                  <span className="text-foreground font-semibold">0.0.10462941</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Quorum:</span>
                  <span className="text-foreground">2-of-2 secp256k1</span>
                </div>
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setActiveTab("audit")}
                className="w-full text-xs justify-between"
              >
                <span>Audit & Consensus Center</span>
                <ArrowRight className="size-3" />
              </Button>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
