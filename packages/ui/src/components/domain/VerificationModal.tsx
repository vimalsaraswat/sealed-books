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
import { Card, CardContent } from "../ui/card";
import { truncateHash, getHashscanUrl } from "../../lib/utils";
import type { VerificationReport } from "../../types";
import {
  CheckCircle2,
  XCircle,
  ShieldCheck,
  ShieldAlert,
  ArrowUpRight,
  AlertTriangle,
} from "lucide-react";

export interface VerificationModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  report: VerificationReport | null;
  isLoading?: boolean;
  onLocateOffendingEntry?: (entryId: string) => void;
}

export function VerificationModal({
  open,
  onOpenChange,
  report,
  isLoading = false,
  onLocateOffendingEntry,
}: VerificationModalProps) {
  if (!report && !isLoading) return null;

  const isIntact = report?.is_intact === true;
  const topicId = report?.topic_id || "0.0.10462941";
  const seqNum = report?.sequence_number || 1;
  const consensusRoot =
    report?.consensus_merkle_root ||
    report?.on_chain_statement?.ledger_root ||
    "";
  const databaseRoot =
    report?.ledger_merkle_root || report?.database_summary?.ledger_root || "";

  const approvers = report?.approvers || [
    ...(report?.approver_1
      ? [
          {
            index: 1,
            address: report.approver_1.eth_address,
            pubkey: report.approver_1.compressed_pubkey,
            signature: "",
            is_valid: report.approver_1.signature_valid,
          },
        ]
      : []),
    ...(report?.approver_2
      ? [
          {
            index: 2,
            address: report.approver_2.eth_address,
            pubkey: report.approver_2.compressed_pubkey,
            signature: "",
            is_valid: report.approver_2.signature_valid,
          },
        ]
      : []),
  ];

  const offendingEntryId =
    report?.discrepancy?.offending_entry_id || report?.offending_entry_id;
  const discrepancyMessage = report?.discrepancy?.message || report?.reason;

  const hashscanUrl = report
    ? getHashscanUrl({
        consensusTimestamp: report.consensus_timestamp,
        topicId: report.topic_id,
        rawUrl: report.hashscan_url,
      })
    : null;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto bg-card border-border">
        <DialogHeader>
          <div className="flex items-center gap-3">
            <div
              className={`p-2.5 rounded-lg border ${
                isIntact
                  ? "bg-emerald-500/10 border-emerald-500/20 text-emerald-600 dark:text-emerald-400"
                  : "bg-destructive/10 border-destructive/20 text-destructive"
              }`}
            >
              {isIntact ? (
                <ShieldCheck className="size-6" />
              ) : (
                <ShieldAlert className="size-6" />
              )}
            </div>
            <div>
              <DialogTitle className="text-xl text-foreground">
                {isLoading
                  ? "Verifying Hedera Consensus..."
                  : isIntact
                    ? "Ledger Integrity Verified"
                    : "Tamper Detected in Ledger"}
              </DialogTitle>
              <DialogDescription className="text-xs text-muted-foreground">
                {isLoading
                  ? "Fetching cryptographic proof from Hedera mirror node..."
                  : isIntact
                    ? "Every journal entry matches the consensus Merkle root on Hedera."
                    : "Database has been altered after the period was cryptographically sealed."}
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        {isLoading ? (
          <div className="py-12 flex flex-col items-center justify-center gap-3 text-muted-foreground">
            <div className="size-8 border-3 border-primary border-t-transparent rounded-full animate-spin" />
            <p className="text-xs font-medium">
              Querying Hedera Mirror Node Topic {topicId}...
            </p>
          </div>
        ) : (
          <div className="space-y-4 py-2">
            {/* Status Summary Banner */}
            <div
              className={`p-4 rounded-lg border flex items-center justify-between ${
                isIntact
                  ? "bg-emerald-500/10 border-emerald-500/20 text-emerald-900 dark:text-emerald-300"
                  : "bg-destructive/10 border-destructive/20 text-destructive"
              }`}
            >
              <div className="flex items-center gap-2 font-semibold text-sm">
                {isIntact ? (
                  <>
                    <CheckCircle2 className="size-5 text-emerald-600 dark:text-emerald-400" />
                    <span>Cryptographic Verification Passed</span>
                  </>
                ) : (
                  <>
                    <XCircle className="size-5 text-destructive" />
                    <span>Merkle Root Mismatch</span>
                  </>
                )}
              </div>
              <Badge
                variant={isIntact ? "success" : "destructive"}
                className="text-xs"
              >
                {isIntact ? "100% INTACT" : "TAMPER DETECTED"}
              </Badge>
            </div>

            {/* If Tamper / Discrepancy Found */}
            {!isIntact && offendingEntryId && (
              <Card className="border-destructive/30 bg-destructive/5">
                <CardContent className="p-4 space-y-2">
                  <div className="flex items-center gap-2 text-destructive font-semibold text-xs uppercase tracking-wider">
                    <AlertTriangle className="size-4" />
                    <span>Identified Offending Entry</span>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    The leaf hash for entry{" "}
                    <code className="font-mono font-bold text-destructive">
                      {offendingEntryId}
                    </code>{" "}
                    does not match the sealed Merkle tree.
                  </p>
                  {discrepancyMessage && (
                    <div className="p-2 rounded bg-background border border-border font-mono text-[11px] text-muted-foreground">
                      {discrepancyMessage}
                    </div>
                  )}
                  {onLocateOffendingEntry && (
                    <Button
                      size="sm"
                      variant="destructive"
                      onClick={() => {
                        onOpenChange(false);
                        onLocateOffendingEntry(offendingEntryId);
                      }}
                      className="text-xs gap-1.5 mt-2 h-8"
                    >
                      <span>Highlight in Ledger Table</span>
                    </Button>
                  )}
                </CardContent>
              </Card>
            )}

            {/* Hedera Consensus Service Verification Details */}
            <div className="rounded-lg border border-border p-4 space-y-3 bg-card text-xs">
              <div className="font-semibold text-xs text-foreground uppercase tracking-wider">
                Consensus Service Proof (HCS)
              </div>

              <div className="grid grid-cols-2 gap-2 font-mono">
                <div className="p-2.5 rounded bg-muted/40 border border-border">
                  <span className="text-[10px] text-muted-foreground uppercase block">
                    Topic ID
                  </span>
                  <span className="text-foreground font-semibold">
                    {topicId}
                  </span>
                </div>
                <div className="p-2.5 rounded bg-muted/40 border border-border">
                  <span className="text-[10px] text-muted-foreground uppercase block">
                    Sequence #
                  </span>
                  <span className="text-foreground font-semibold">
                    {seqNum}
                  </span>
                </div>
              </div>

              <div className="space-y-2 pt-1 font-mono text-[11px]">
                <div className="p-2.5 rounded bg-muted/40 border border-border">
                  <span className="text-[10px] text-muted-foreground uppercase block">
                    Hedera Consensus Merkle Root (Anchor)
                  </span>
                  <span className="text-foreground font-semibold break-all">
                    {consensusRoot || "N/A"}
                  </span>
                </div>
                <div className="p-2.5 rounded bg-muted/40 border border-border">
                  <span className="text-[10px] text-muted-foreground uppercase block">
                    Recalculated Database Merkle Root
                  </span>
                  <span
                    className={`font-semibold break-all ${
                      isIntact
                        ? "text-emerald-600 dark:text-emerald-400"
                        : "text-destructive"
                    }`}
                  >
                    {databaseRoot || "N/A"}
                  </span>
                </div>
              </div>
            </div>

            {/* Dual Custody Signers */}
            <div className="rounded-lg border border-border p-4 space-y-2 bg-card text-xs">
              <div className="font-semibold text-xs text-foreground uppercase tracking-wider">
                Multi-Party Signature Verification
              </div>
              <div className="space-y-2">
                {approvers.map((app) => (
                  <div
                    key={app.index}
                    className="flex items-center justify-between p-2 rounded bg-muted/40 border border-border"
                  >
                    <div className="space-y-0.5">
                      <div className="font-medium text-foreground">
                        {app.index === 1
                          ? "Approver 1 (Controller)"
                          : "Approver 2 (Auditor)"}
                      </div>
                      <div className="font-mono text-[11px] text-muted-foreground">
                        {truncateHash(app.address || "", 12, 10)}
                      </div>
                    </div>
                    <Badge
                      variant={app.is_valid ? "success" : "destructive"}
                      className="text-[10px]"
                    >
                      {app.is_valid ? "Signature Valid" : "Invalid Signature"}
                    </Badge>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        <DialogFooter className="pt-2 border-t border-border flex items-center justify-between">
          {hashscanUrl && (
            <Button
              type="button"
              variant="outline"
              size="sm"
              asChild
              className="gap-1.5 text-xs mr-auto h-9"
            >
              <a href={hashscanUrl} target="_blank" rel="noopener noreferrer">
                <span>View on HashScan</span>
                <ArrowUpRight className="size-3.5" />
              </a>
            </Button>
          )}
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => onOpenChange(false)}
            className="text-xs h-9"
          >
            Close
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
