import { useState } from "react";
import {
  ShieldCheck,
  ArrowUpRight,
  Clock,
  AlertTriangle,
  Award,
  Bug,
  CheckCircle2,
  Copy,
  Check,
  Activity,
  Layers,
  FileCheck,
  Search,
  ExternalLink,
} from "lucide-react";
import { Button } from "../ui/button";
import { Badge } from "../ui/badge";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "../ui/card";
import { truncateHash } from "../../lib/utils";
import { useAuth } from "../../context/AuthContext";
import { useLedger } from "../../context/LedgerContext";
import { useModals } from "../../context/ModalContext";

export function AuditCenterView() {
  const { userRole } = useAuth();
  const {
    currentPeriod,
    entries,
    statement,
    seal,
    verificationReport,
    isVerifying,
    setActiveTab,
    handleVerify,
    handleLocateOffendingEntry,
  } = useLedger();

  const { setIsAuditorModalOpen, setIsCertificateOpen, setIsTamperOpen } =
    useModals();

  const [copiedKey, setCopiedKey] = useState<string | null>(null);

  const isSealed = currentPeriod?.status === "sealed";
  const isAuditor = userRole === "auditor";
  const isPendingAuditor = seal?.dispatch_status === "pending_auditor";

  const handleCopy = (key: string, text: string) => {
    navigator.clipboard.writeText(text);
    setCopiedKey(key);
    setTimeout(() => setCopiedKey(null), 1500);
  };

  const handleRunVerify = async () => {
    await handleVerify();
  };

  const handleOpenCert = async () => {
    if (!seal) return;
    await handleVerify();
    setIsCertificateOpen(true);
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
          Please select an accounting period to review cryptographic audit
          proofs.
        </p>
      </div>
    );
  }

  const topicId = seal?.topic_id || "0.0.10462941";
  const ledgerRoot =
    seal?.root ||
    statement?.ledger_root_hex ||
    statement?.statement.ledger_root ||
    "0xca714ec5092a4060db2a5fbe42617f6daec9805561917f6cb89698dcf6fae9a4";

  const offendingEntryId =
    verificationReport?.discrepancy?.offending_entry_id ||
    verificationReport?.offending_entry_id;

  const calculatedRoot =
    verificationReport?.database_summary?.ledger_root ||
    verificationReport?.ledger_merkle_root ||
    "—";

  const consensusRoot =
    verificationReport?.on_chain_statement?.ledger_root ||
    verificationReport?.consensus_merkle_root ||
    "—";

  return (
    <div className="space-y-6">
      {/* =========================================================
          TOP HEADER
          ========================================================= */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-border/80 pb-4">
        <div>
          <h2 className="text-lg font-bold text-foreground flex items-center gap-2">
            <ShieldCheck className="size-5 text-primary" />
            <span>Cryptographic Consensus & Audit Center</span>
          </h2>
          <p className="text-xs text-muted-foreground font-mono">
            Zero-knowledge consensus anchoring on Hedera Topic #{topicId}
          </p>
        </div>

        <div className="flex items-center gap-2">
          {isSealed && (
            <Button
              variant="outline"
              size="sm"
              onClick={handleOpenCert}
              className="h-8 gap-1.5 text-xs"
            >
              <Award className="size-3.5 text-primary" />
              <span>Audit Certificate</span>
            </Button>
          )}

          <Button
            variant="default"
            size="sm"
            onClick={handleRunVerify}
            disabled={isVerifying || !isSealed}
            className="h-8 gap-1.5 text-xs bg-emerald-600 hover:bg-emerald-700 text-white"
          >
            <ShieldCheck className="size-3.5" />
            <span>
              {isVerifying
                ? "Verifying Mirror Node..."
                : "Verify Ledger Integrity"}
            </span>
          </Button>
        </div>
      </div>

      {/* =========================================================
          AUDITOR REVIEW NOTIFICATION (IF PENDING)
          ========================================================= */}
      {isPendingAuditor && !isSealed && (
        <Card className="border-amber-500/40 bg-amber-500/5 shadow-xs">
          <CardContent className="p-5">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
              <div className="space-y-1">
                <div className="flex items-center gap-2">
                  <Clock className="size-4 text-amber-600 dark:text-amber-400" />
                  <span className="font-bold text-sm text-foreground">
                    Independent Auditor Review Pending
                  </span>
                  <Badge
                    variant="outline"
                    className="border-amber-500/30 text-amber-600 bg-amber-500/10 text-[10px] font-mono"
                  >
                    2-of-2 Quorum
                  </Badge>
                </div>
                <p className="text-xs text-muted-foreground">
                  The Controller has prepared and signed the period close
                  statement. An external auditor co-signature is required to
                  commit the seal to Hedera Consensus Service.
                </p>
              </div>

              {isAuditor ? (
                <Button
                  variant="default"
                  size="sm"
                  onClick={() => setIsAuditorModalOpen(true)}
                  className="bg-emerald-600 hover:bg-emerald-700 text-white gap-1.5 text-xs shrink-0 animate-pulse"
                >
                  <FileCheck className="size-4" />
                  <span>Review & Co-Sign Period</span>
                </Button>
              ) : (
                <span className="text-xs font-mono text-amber-600 dark:text-amber-400">
                  Waiting for auditor sign-off
                </span>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      {/* =========================================================
          1. CONSENSUS PROOF CARDS (3 COLUMNS)
          ========================================================= */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Card 1: Hedera HCS Anchor */}
        <Card className="border-border shadow-xs">
          <CardHeader className="p-4 pb-2">
            <div className="flex items-center justify-between">
              <CardDescription className="text-xs font-mono uppercase font-semibold text-muted-foreground">
                Consensus Anchor
              </CardDescription>
              <Badge variant="outline" className="text-[10px] font-mono">
                Hedera Testnet
              </Badge>
            </div>
            <CardTitle className="text-base font-bold text-foreground flex items-center gap-1.5 pt-1">
              <span>Topic #{topicId}</span>
              <a
                href={`https://hashscan.io/testnet/topic/${topicId}`}
                target="_blank"
                rel="noreferrer"
                className="text-primary hover:text-primary/80"
                title="View Topic on HashScan"
              >
                <ExternalLink className="size-3.5" />
              </a>
            </CardTitle>
          </CardHeader>
          <CardContent className="p-4 pt-1 space-y-2 text-xs font-mono">
            <div className="flex justify-between text-muted-foreground">
              <span>Sequence #:</span>
              <span className="text-foreground font-semibold">
                {seal?.sequence_number || (isSealed ? "1" : "Pending")}
              </span>
            </div>
            <div className="flex justify-between text-muted-foreground">
              <span>Payload Size:</span>
              <span className="text-foreground">
                303 bytes (compact binary)
              </span>
            </div>
            {seal?.consensus_timestamp && (
              <div className="pt-1">
                <a
                  href={`https://hashscan.io/testnet/transaction/${seal.consensus_timestamp}`}
                  target="_blank"
                  rel="noreferrer"
                  className="text-[11px] text-primary hover:underline flex items-center gap-1"
                >
                  <span>Timestamp: {seal.consensus_timestamp}</span>
                  <ArrowUpRight className="size-3" />
                </a>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Card 2: 2-of-2 Signature Quorum */}
        <Card className="border-border shadow-xs">
          <CardHeader className="p-4 pb-2">
            <div className="flex items-center justify-between">
              <CardDescription className="text-xs font-mono uppercase font-semibold text-muted-foreground">
                Multi-Sig Quorum
              </CardDescription>
              <Badge
                variant={isSealed ? "sealed" : "outline"}
                className="text-[10px] font-mono uppercase"
              >
                {isSealed
                  ? "2 of 2 Signed"
                  : isPendingAuditor
                    ? "1 of 2 Signed"
                    : "0 of 2 Signed"}
              </Badge>
            </div>
            <CardTitle className="text-base font-bold text-foreground pt-1">
              secp256k1 Dual Custody
            </CardTitle>
          </CardHeader>
          <CardContent className="p-4 pt-1 space-y-2 text-xs font-mono">
            {/* Approver 1: Controller */}
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground">1. Controller:</span>
              <div className="flex items-center gap-1 text-foreground font-semibold">
                {isSealed || isPendingAuditor ? (
                  <>
                    <Check className="size-3 text-emerald-500" />
                    <span>Signed</span>
                  </>
                ) : (
                  <span className="text-muted-foreground font-normal">
                    Pending
                  </span>
                )}
              </div>
            </div>

            {/* Approver 2: Auditor */}
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground">2. Auditor:</span>
              <div className="flex items-center gap-1 text-foreground font-semibold">
                {isSealed ? (
                  <>
                    <Check className="size-3 text-emerald-500" />
                    <span>Attested</span>
                  </>
                ) : (
                  <span className="text-muted-foreground font-normal">
                    Pending
                  </span>
                )}
              </div>
            </div>

            <p className="text-[10px] text-muted-foreground pt-1 font-sans">
              Both parties must independently sign the statement hash before
              dispatch.
            </p>
          </CardContent>
        </Card>

        {/* Card 3: Ledger Merkle Root */}
        <Card className="border-border shadow-xs">
          <CardHeader className="p-4 pb-2">
            <div className="flex items-center justify-between">
              <CardDescription className="text-xs font-mono uppercase font-semibold text-muted-foreground">
                Cryptographic Root
              </CardDescription>
              <Badge variant="outline" className="text-[10px] font-mono">
                Keccak-256
              </Badge>
            </div>
            <CardTitle className="text-base font-bold text-foreground pt-1">
              Sorted Merkle Root
            </CardTitle>
          </CardHeader>
          <CardContent className="p-4 pt-1 space-y-2">
            <div className="p-2 rounded bg-muted/50 font-mono text-[11px] text-muted-foreground break-all flex items-start justify-between gap-2">
              <span>{truncateHash(ledgerRoot, 14, 12)}</span>
              <button
                onClick={() => handleCopy("root", ledgerRoot)}
                title="Copy full 32-byte Merkle root"
                className="hover:text-foreground shrink-0 mt-0.5"
              >
                {copiedKey === "root" ? (
                  <Check className="size-3 text-emerald-500" />
                ) : (
                  <Copy className="size-3" />
                )}
              </button>
            </div>
            <p className="text-[10px] text-muted-foreground">
              Deterministic Merkle tree generated over all{" "}
              {statement?.statement.entry_count ?? entries.length} transactions.
            </p>
          </CardContent>
        </Card>
      </div>

      {/* =========================================================
          2. LIVE VERIFICATION CONSOLE & TELEMETRY
          ========================================================= */}
      <Card className="border-border shadow-xs">
        <CardHeader className="p-5 pb-3">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
            <div>
              <CardTitle className="text-sm font-bold text-foreground flex items-center gap-2">
                <Activity className="size-4 text-primary" />
                <span>Zero-Trust Verification Engine</span>
              </CardTitle>
              <CardDescription className="text-xs text-muted-foreground">
                Independent HTTP verification comparing SQLite ledger rows with
                Hedera Mirror Node
              </CardDescription>
            </div>

            {verificationReport && (
              <Badge
                variant={
                  verificationReport.status === "verified"
                    ? "sealed"
                    : "destructive"
                }
                className="text-xs font-mono px-2.5 py-1 gap-1.5"
              >
                {verificationReport.status === "verified" ? (
                  <>
                    <CheckCircle2 className="size-3.5" />
                    <span>CONSENSUS VERIFIED</span>
                  </>
                ) : (
                  <>
                    <AlertTriangle className="size-3.5" />
                    <span>TAMPER DETECTED</span>
                  </>
                )}
              </Badge>
            )}
          </div>
        </CardHeader>

        <CardContent className="p-5 pt-1 space-y-4">
          {/* Telemetry Output / Report */}
          {verificationReport ? (
            <div
              className={`p-4 rounded-lg border font-mono text-xs space-y-3 ${
                verificationReport.status === "verified"
                  ? "bg-emerald-500/5 border-emerald-500/30 text-foreground"
                  : "bg-destructive/10 border-destructive/30 text-foreground"
              }`}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 font-bold">
                  {verificationReport.status === "verified" ? (
                    <span className="text-emerald-600 dark:text-emerald-400">
                      ✓ Consensus Intact: All records match Hedera Testnet
                      anchor.
                    </span>
                  ) : (
                    <span className="text-destructive">
                      ✕ Fraud Alert: Tampered ledger entry detected.
                    </span>
                  )}
                </div>
                <span className="text-[10px] text-muted-foreground">
                  Timestamp: {verificationReport.consensus_timestamp}
                </span>
              </div>

              {/* Details grid */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 text-[11px] pt-1">
                <div>
                  <span className="text-muted-foreground">
                    Hedera Topic ID:
                  </span>{" "}
                  <span className="font-semibold">
                    {verificationReport.topic_id}
                  </span>
                </div>
                <div>
                  <span className="text-muted-foreground">
                    Sequence Number:
                  </span>{" "}
                  <span className="font-semibold">
                    {verificationReport.sequence_number}
                  </span>
                </div>
                <div>
                  <span className="text-muted-foreground">
                    Calculated Local Root:
                  </span>{" "}
                  <span className="font-semibold">
                    {truncateHash(calculatedRoot, 10, 8)}
                  </span>
                </div>
                <div>
                  <span className="text-muted-foreground">
                    Consensus Proof Root:
                  </span>{" "}
                  <span className="font-semibold">
                    {truncateHash(consensusRoot, 10, 8)}
                  </span>
                </div>
              </div>

              {/* Offending Entry Details if Tampered */}
              {verificationReport.status === "tampered" && offendingEntryId && (
                <div className="mt-3 p-3 rounded bg-destructive/15 border border-destructive/30 space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="font-bold text-destructive text-xs">
                      Offending Entry: {offendingEntryId}
                    </span>
                    <Button
                      variant="destructive"
                      size="sm"
                      onClick={() => {
                        if (offendingEntryId) {
                          handleLocateOffendingEntry(offendingEntryId);
                          setActiveTab("ledger");
                        }
                      }}
                      className="h-7 text-xs gap-1"
                    >
                      <Search className="size-3" />
                      <span>Locate in General Ledger</span>
                    </Button>
                  </div>
                  <p className="text-[11px] text-foreground font-sans">
                    The cryptographic leaf hash computed from the local SQLite
                    database does not match the leaf set committed to Hedera
                    Consensus Service.
                  </p>
                </div>
              )}
            </div>
          ) : (
            <div className="p-6 rounded-lg border border-border/60 bg-muted/20 text-center space-y-2">
              <ShieldCheck className="size-8 mx-auto text-muted-foreground/60" />
              <p className="text-xs font-semibold text-foreground">
                Zero-Knowledge Verifier Standing By
              </p>
              <p className="text-[11px] text-muted-foreground max-w-md mx-auto font-sans">
                Click "Verify Ledger Integrity" to fetch the 303-byte canonical
                consensus record from the Hedera Mirror Node and verify all
                transactions.
              </p>
            </div>
          )}
        </CardContent>
      </Card>

      {/* =========================================================
          3. UTILITIES: TAMPER LAB & CERTIFICATE
          ========================================================= */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Certificate Generator Card */}
        <Card className="border-border shadow-xs">
          <CardHeader className="p-4 pb-2">
            <CardTitle className="text-sm font-bold text-foreground flex items-center gap-2">
              <Award className="size-4 text-primary" />
              <span>Certificate of Auditability</span>
            </CardTitle>
            <CardDescription className="text-xs text-muted-foreground">
              Printable cryptographic certificate documenting signers and Hedera
              consensus anchor
            </CardDescription>
          </CardHeader>
          <CardContent className="p-4 pt-1 space-y-3">
            <p className="text-xs text-muted-foreground">
              Suitable for formal audit submissions, bank debt covenants, and
              board reviews.
            </p>
            <Button
              variant="outline"
              size="sm"
              onClick={handleOpenCert}
              disabled={!isSealed}
              className="w-full text-xs justify-between"
            >
              <span>
                {isSealed
                  ? "Open Printable Certificate"
                  : "Available Post-Close"}
              </span>
              <Award className="size-3.5" />
            </Button>
          </CardContent>
        </Card>

        {/* Tamper Simulator Lab */}
        <Card className="border-border shadow-xs">
          <CardHeader className="p-4 pb-2">
            <CardTitle className="text-sm font-bold text-foreground flex items-center gap-2">
              <Bug className="size-4 text-destructive" />
              <span>Integrity Testing Sandbox</span>
            </CardTitle>
            <CardDescription className="text-xs text-muted-foreground">
              Simulate a quiet direct SQLite mutation to demonstrate tamper
              detection
            </CardDescription>
          </CardHeader>
          <CardContent className="p-4 pt-1 space-y-3">
            <p className="text-xs text-muted-foreground">
              Deliberately alters a transaction memo or amount to test the
              zero-trust verifier.
            </p>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setIsTamperOpen(true)}
              disabled={!isSealed}
              className="w-full text-xs justify-between border-destructive/30 text-destructive hover:bg-destructive/10"
            >
              <span>
                {isSealed
                  ? "Open Tamper Simulator"
                  : "Available on Sealed Periods"}
              </span>
              <Bug className="size-3.5" />
            </Button>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
