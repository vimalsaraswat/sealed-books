import { useMemo, useState } from "react";
import { useLedger } from "../../context/LedgerContext";
import { useAuth } from "../../context/AuthContext";
import { useModals } from "../../context/ModalContext";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "../ui/card";
import { Button } from "../ui/button";
import { Badge } from "../ui/badge";
import { formatCurrency, formatDate } from "../../lib/utils";
import {
  TrendingUp,
  TrendingDown,
  Scale,
  Hash,
  ShieldCheck,
  Award,
  Lock,
  Plus,
  ArrowRight,
  Clock,
  ArrowUpRight,
  BookOpen,
  FileSpreadsheet,
  CheckCircle2,
  CircleDot,
  Layers,
  AlertTriangle,
} from "lucide-react";

export function OverviewView() {
  const {
    currentPeriod,
    entries,
    accounts,
    seal,
    setActiveTab,
    handleOpenCloseDialog,
    handleVerify,
  } = useLedger();

  const { userRole } = useAuth();
  const [isVerifying, setIsVerifying] = useState(false);

  const {
    setIsAddEntryOpen,
    setIsCloseModalOpen,
    setIsAuditorModalOpen,
    setIsVerifyModalOpen,
    setIsCertificateOpen,
  } = useModals();

  const isSealed = currentPeriod?.status === "sealed";
  const isAuditor = userRole === "auditor";
  const hasControllerSigned = Boolean(
    seal?.approver_1_pubkey && seal?.approver_1_sig,
  );
  const isPendingAuditor =
    seal?.dispatch_status === "pending_auditor" ||
    (hasControllerSigned && !isSealed);
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

  const handleProposeClose = async () => {
    const propStmt = await handleOpenCloseDialog();
    if (propStmt) {
      setIsCloseModalOpen(true);
    }
  };

  const handleRunVerification = async () => {
    try {
      setIsVerifying(true);
      const rep = await handleVerify();
      if (rep) {
        setIsVerifyModalOpen(true);
      }
    } finally {
      setIsVerifying(false);
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
                Period: {currentPeriod.start_date} to {currentPeriod.end_date} •
                ID: {currentPeriod.id}
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
                    <span>
                      {isVerifying ? "Verifying..." : "Verify on Hedera"}
                    </span>
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
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <TrendingUp className="size-3.5 text-blue-500" />
              <span>Cumulative turnover</span>
            </div>
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
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <TrendingDown className="size-3.5 text-emerald-500" />
              <span>Cumulative turnover</span>
            </div>
          </CardContent>
        </Card>

        {/* Trial Balance Invariance */}
        <Card
          className={`border shadow-xs ${
            isBalanced
              ? "border-emerald-500/20 bg-emerald-500/5 dark:bg-emerald-950/10"
              : "border-destructive/20 bg-destructive/5 dark:bg-destructive/10"
          }`}
        >
          <CardHeader className="p-4 pb-1">
            <CardDescription className="text-xs font-medium uppercase font-mono tracking-wider">
              Trial Balance Delta
            </CardDescription>
            <CardTitle
              className={`text-2xl font-bold font-mono tracking-tight ${
                isBalanced
                  ? "text-emerald-600 dark:text-emerald-400"
                  : "text-destructive"
              }`}
            >
              {isBalanced ? "$0.00" : formatCurrency(variance)}
            </CardTitle>
          </CardHeader>
          <CardContent className="p-4 pt-1">
            <div className="flex items-center gap-1.5 text-xs font-medium">
              <Scale
                className={`size-3.5 ${
                  isBalanced ? "text-emerald-500" : "text-destructive"
                }`}
              />
              <span
                className={
                  isBalanced
                    ? "text-emerald-600 dark:text-emerald-400"
                    : "text-destructive"
                }
              >
                {isBalanced
                  ? "Zero Invariance Satisfied"
                  : "Unbalanced Ledger!"}
              </span>
            </div>
          </CardContent>
        </Card>

        {/* Total Ledger Entries */}
        <Card className="border-border shadow-xs">
          <CardHeader className="p-4 pb-1">
            <CardDescription className="text-xs font-medium uppercase font-mono tracking-wider">
              Recorded Entries
            </CardDescription>
            <CardTitle className="text-2xl font-bold font-mono tracking-tight text-foreground">
              {entries.length}
            </CardTitle>
          </CardHeader>
          <CardContent className="p-4 pt-1">
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <Hash className="size-3.5 text-primary" />
              <span>Double-entry batches</span>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* =========================================================
          3. PERIOD STATUS & AUDIT PIPELINE STEPPER
          ========================================================= */}
      <Card className="border-border shadow-xs">
        <CardHeader className="p-4 pb-2">
          <CardTitle className="text-sm font-semibold tracking-tight text-foreground">
            Cryptographic Sealing Pipeline
          </CardTitle>
          <CardDescription className="text-xs text-muted-foreground">
            Current stage in the four-eyes dual-signature closing workflow.
          </CardDescription>
        </CardHeader>
        <CardContent className="p-4 pt-1">
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
            {/* Step 1 */}
            <div className="p-3 rounded-lg border border-border/60 bg-muted/20 flex items-start gap-3">
              <CheckCircle2 className="size-5 text-emerald-500 shrink-0 mt-0.5" />
              <div className="space-y-0.5">
                <p className="text-xs font-semibold text-foreground">
                  1. Chart of Accounts
                </p>
                <p className="text-[11px] text-muted-foreground">
                  {accounts.length} configured accounts
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
                <p className="text-xs font-semibold text-foreground">
                  2. Post Entries
                </p>
                <p className="text-[11px] text-muted-foreground">
                  {entries.length} balanced transactions
                </p>
              </div>
            </div>

            {/* Step 3 */}
            <div className="p-3 rounded-lg border border-border/60 bg-muted/20 flex items-start gap-3">
              {isSealed || hasControllerSigned || isPendingAuditor ? (
                <CheckCircle2 className="size-5 text-emerald-500 shrink-0 mt-0.5" />
              ) : (
                <CircleDot className="size-5 text-muted-foreground shrink-0 mt-0.5" />
              )}
              <div className="space-y-0.5">
                <p className="text-xs font-semibold text-foreground">
                  3. Controller Close
                </p>
                <p className="text-[11px] text-muted-foreground">
                  {isSealed || hasControllerSigned || isPendingAuditor
                    ? "Approver 1 signed"
                    : "Pending proposal"}
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
                <p className="text-xs font-semibold text-foreground">
                  4. Hedera Seal
                </p>
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
                    className="p-3.5 hover:bg-muted/30 transition-colors flex items-center justify-between gap-4"
                  >
                    <div className="space-y-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="font-mono text-xs font-bold text-foreground">
                          {entry.id}
                        </span>
                        <span className="text-[11px] text-muted-foreground font-mono">
                          {formatDate(entry.date)}
                        </span>
                      </div>
                      <p className="text-xs text-muted-foreground truncate">
                        {entry.description}
                      </p>
                    </div>

                    <div className="text-right shrink-0">
                      <span className="font-mono text-xs font-semibold text-foreground">
                        {formatCurrency(totalDebitMinor)}
                      </span>
                      <p className="text-[10px] text-muted-foreground font-mono">
                        {entry.lines.length} lines
                      </p>
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>

        {/* Right Col: Accounting Period Summary Card */}
        <div className="space-y-3">
          <h3 className="text-sm font-semibold text-foreground flex items-center gap-2">
            <FileSpreadsheet className="size-4 text-primary" />
            <span>Period Information</span>
          </h3>

          <Card className="border-border bg-card shadow-xs">
            <CardContent className="p-4 space-y-3 text-xs">
              <div className="flex items-center justify-between pb-2 border-b border-border">
                <span className="text-muted-foreground">Reporting Entity</span>
                <span className="font-semibold text-foreground text-right">
                  {currentPeriod.entity}
                </span>
              </div>
              <div className="flex items-center justify-between pb-2 border-b border-border">
                <span className="text-muted-foreground">Start Date</span>
                <span className="font-mono text-foreground">
                  {currentPeriod.start_date}
                </span>
              </div>
              <div className="flex items-center justify-between pb-2 border-b border-border">
                <span className="text-muted-foreground">End Date</span>
                <span className="font-mono text-foreground">
                  {currentPeriod.end_date}
                </span>
              </div>
              <div className="flex items-center justify-between pb-2 border-b border-border">
                <span className="text-muted-foreground">Period Status</span>
                <Badge
                  variant={isSealed ? "sealed" : "default"}
                  className="font-mono text-[10px]"
                >
                  {currentPeriod.status.toUpperCase()}
                </Badge>
              </div>
              {seal?.statement_hash && (
                <div className="pt-1 space-y-1">
                  <span className="text-muted-foreground text-[11px]">
                    Statement Pre-Hash
                  </span>
                  <div className="p-2 rounded bg-muted/50 border border-border/50 font-mono text-[10px] break-all text-foreground">
                    {seal.statement_hash}
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
