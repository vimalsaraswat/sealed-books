import { useState } from "react";
import {
  Award,
  CheckCircle2,
  Copy,
  ExternalLink,
  ShieldCheck,
} from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "../ui/dialog";
import { Button } from "../ui/button";
import { Badge } from "../ui/badge";
import { formatCurrency, truncateHash, getHashscanUrl } from "../../lib/utils";
import type { Period, VerificationReport } from "../../types";

export interface AuditCertificateModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  period: Period;
  report: VerificationReport | null;
}

export function AuditCertificateModal({
  open,
  onOpenChange,
  period,
  report,
}: AuditCertificateModalProps) {
  const [copied, setCopied] = useState(false);

  if (!report) return null;

  const hashscanUrl = getHashscanUrl({
    consensusTimestamp: report.consensus_timestamp,
    topicId: report.topic_id,
    rawUrl: report.hashscan_url,
  });

  const handleCopyJSON = () => {
    navigator.clipboard.writeText(JSON.stringify(report, null, 2));
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handlePrint = () => {
    window.print();
  };

  const ledgerRoot =
    report.on_chain_statement?.ledger_root ||
    report.ledger_merkle_root ||
    report.consensus_merkle_root ||
    "0x...";
  const statementHash = report.on_chain_statement?.statement_hash || "0x...";
  const entryCount =
    report.on_chain_statement?.entry_count ||
    report.database_summary?.entry_count ||
    0;
  const debits =
    report.on_chain_statement?.total_debits_minor ||
    report.database_summary?.total_debits_minor ||
    0;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl max-h-[90vh] overflow-y-auto print:p-0 print:border-none bg-card border-border">
        <DialogHeader className="border-b border-border pb-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2.5">
              <div className="size-10 rounded-full bg-emerald-500/15 flex items-center justify-center text-emerald-600 dark:text-emerald-400">
                <ShieldCheck className="size-6" />
              </div>
              <div>
                <DialogTitle className="text-xl tracking-tight text-foreground">
                  Cryptographic Audit Certificate
                </DialogTitle>
                <DialogDescription className="text-xs text-muted-foreground">
                  Decentralized Ledger Integrity & Multi-Party Consensus Seal
                </DialogDescription>
              </div>
            </div>
            <Badge variant="success" className="gap-1 px-3 py-1 text-xs">
              <CheckCircle2 className="size-3.5" />
              <span>Consensus Verified</span>
            </Badge>
          </div>
        </DialogHeader>

        {/* Certificate Body */}
        <div className="space-y-4 py-3 text-sm">
          <div className="rounded-lg bg-muted/40 border border-border p-4 space-y-3">
            <div className="grid grid-cols-2 gap-3 text-xs">
              <div>
                <div className="text-muted-foreground font-medium">
                  Reporting Entity
                </div>
                <div className="font-semibold text-foreground text-sm mt-0.5">
                  {period.entity}
                </div>
              </div>
              <div>
                <div className="text-muted-foreground font-medium">
                  Accounting Period
                </div>
                <div className="font-mono font-semibold text-foreground text-sm mt-0.5">
                  {period.start_date} → {period.end_date}
                </div>
              </div>
              <div>
                <div className="text-muted-foreground font-medium">
                  Total Entries Sealed
                </div>
                <div className="font-semibold text-foreground mt-0.5">
                  {entryCount} Balanced Transactions
                </div>
              </div>
              <div>
                <div className="text-muted-foreground font-medium">
                  Closed Ledger Turnover
                </div>
                <div className="font-semibold text-foreground mt-0.5">
                  {formatCurrency(debits)}
                </div>
              </div>
            </div>
          </div>

          {/* Decentralized Proof Metadata */}
          <div className="border border-border rounded-lg p-4 space-y-2.5 bg-card">
            <div className="font-semibold text-xs text-foreground uppercase tracking-wider flex items-center gap-1.5">
              <Award className="size-4 text-primary" />
              <span>Hedera Consensus Service Attestation</span>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-2 text-xs font-mono">
              <div className="bg-muted/40 p-2 rounded border border-border">
                <div className="text-muted-foreground text-[10px] uppercase">
                  Topic ID
                </div>
                <div className="font-semibold text-foreground mt-0.5">
                  {report.topic_id}
                </div>
              </div>
              <div className="bg-muted/40 p-2 rounded border border-border">
                <div className="text-muted-foreground text-[10px] uppercase">
                  Sequence #
                </div>
                <div className="font-semibold text-foreground mt-0.5">
                  {report.sequence_number}
                </div>
              </div>
              <div className="bg-muted/40 p-2 rounded border border-border">
                <div className="text-muted-foreground text-[10px] uppercase">
                  Consensus Time
                </div>
                <div
                  className="font-semibold text-foreground mt-0.5 truncate"
                  title={report.consensus_timestamp}
                >
                  {report.consensus_timestamp}
                </div>
              </div>
            </div>

            <div className="space-y-1.5 pt-1 text-xs">
              <div className="flex items-center justify-between py-1 border-b border-border">
                <span className="text-muted-foreground font-medium">
                  Ledger Merkle Root:
                </span>
                <span
                  className="font-mono text-[11px] text-foreground"
                  title={ledgerRoot}
                >
                  {truncateHash(ledgerRoot, 14, 12)}
                </span>
              </div>
              <div className="flex items-center justify-between py-1 border-b border-border">
                <span className="text-muted-foreground font-medium">
                  Canonical Statement Hash:
                </span>
                <span
                  className="font-mono text-[11px] text-foreground"
                  title={statementHash}
                >
                  {truncateHash(statementHash, 14, 12)}
                </span>
              </div>
            </div>
          </div>

          {/* Multi-Sig Approvers */}
          <div className="border border-border rounded-lg p-4 space-y-2 bg-card">
            <div className="font-semibold text-xs text-foreground uppercase tracking-wider">
              Cryptographic Multi-Signature Quorum (2-of-2)
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs">
              <div className="bg-emerald-500/10 border border-emerald-500/20 rounded p-2.5">
                <div className="flex items-center justify-between">
                  <span className="font-medium text-emerald-900 dark:text-emerald-300">
                    Approver 1 (Controller)
                  </span>
                  <Badge variant="success" className="text-[10px] px-1.5 py-0">
                    Valid
                  </Badge>
                </div>
                <div className="font-mono text-[11px] text-emerald-700 dark:text-emerald-400 mt-1 truncate">
                  {report.approver_1?.eth_address || "0x..."}
                </div>
              </div>

              <div className="bg-emerald-500/10 border border-emerald-500/20 rounded p-2.5">
                <div className="flex items-center justify-between">
                  <span className="font-medium text-emerald-900 dark:text-emerald-300">
                    Approver 2 (Auditor)
                  </span>
                  <Badge variant="success" className="text-[10px] px-1.5 py-0">
                    Valid
                  </Badge>
                </div>
                <div className="font-mono text-[11px] text-emerald-700 dark:text-emerald-400 mt-1 truncate">
                  {report.approver_2?.eth_address || "0x..."}
                </div>
              </div>
            </div>
          </div>
        </div>

        <DialogFooter className="flex-col sm:flex-row gap-2 border-t border-border pt-3">
          {hashscanUrl && (
            <Button
              type="button"
              variant="outline"
              size="sm"
              asChild
              className="gap-1.5 text-xs mr-auto"
            >
              <a href={hashscanUrl} target="_blank" rel="noopener noreferrer">
                <span>View on HashScan</span>
                <ExternalLink className="size-3.5" />
              </a>
            </Button>
          )}
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={handleCopyJSON}
            className="gap-1.5 text-xs"
          >
            <Copy className="size-3.5" />
            <span>{copied ? "Copied JSON!" : "Copy Proof JSON"}</span>
          </Button>
          <Button
            type="button"
            size="sm"
            onClick={handlePrint}
            className="gap-1.5 text-xs"
          >
            <span>Print Certificate</span>
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
