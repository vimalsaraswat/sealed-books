import { useState, useEffect } from "react";
import { useAuth } from "../../context/AuthContext";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "../ui/dialog";
import { Button } from "../ui/button";
import { Card, CardContent } from "../ui/card";
import { formatCurrency, truncateHash, getHashscanUrl } from "../../lib/utils";
import type { StatementResponse, SealRecord } from "../../types";
import {
  ShieldCheck,
  CheckCircle2,
  XCircle,
  Lock,
  ArrowUpRight,
  Sparkles,
  AlertTriangle,
  Loader2,
} from "lucide-react";

export interface AuditorReviewModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  statementResponse: StatementResponse | null;
  seal: SealRecord | null;
  onApproveAndPublish: (data?: {
    approver_pubkey?: string;
    signature?: string;
    wallet_id?: string;
  }) => Promise<SealRecord>;
  onReject: (notes: string) => Promise<void>;
  onSuccessDone?: () => void;
}

export function AuditorReviewModal({
  open,
  onOpenChange,
  statementResponse,
  seal,
  onApproveAndPublish,
  onReject,
  onSuccessDone,
}: AuditorReviewModalProps) {
  const [isRejecting, setIsRejecting] = useState(false);
  const [rejectNotes, setRejectNotes] = useState("");
  const [isSubmittingReject, setIsSubmittingReject] = useState(false);
  const { currentUser } = useAuth();
  const [isSigning, setIsSigning] = useState(false);
  const [publishedSeal, setPublishedSeal] = useState<SealRecord | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);



  useEffect(() => {
    if (open) {
      setIsRejecting(false);
      setRejectNotes("");
      setIsSubmittingReject(false);
      setIsSigning(false);
      setPublishedSeal(null);
      setErrorMsg(null);
    }
  }, [open]);

  if (!statementResponse) return null;

  const { statement, statement_hash_hex } = statementResponse;
  const statementHash = statement_hash_hex || seal?.statement_hash || "";

  const handleAuditorSignAndPublish = async () => {
    try {
      setErrorMsg(null);
      setIsSigning(true);

      if (!statementHash) {
        throw new Error("Missing statement hash to sign.");
      }

      // Automatically signed via Auditor Privy Server Wallet Policy and published to Hedera HCS
      const resultSeal = await onApproveAndPublish();

      setPublishedSeal(resultSeal);
      if (onSuccessDone) {
        onSuccessDone();
      }
    } catch (err: unknown) {
      const msg =
        err instanceof Error ? err.message : "Failed to sign and publish seal.";
      setErrorMsg(msg);
    } finally {
      setIsSigning(false);
    }
  };

  const handleConfirmReject = async () => {
    if (!rejectNotes.trim()) {
      setErrorMsg(
        "Please provide rejection notes explaining the audit deficiency.",
      );
      return;
    }

    try {
      setErrorMsg(null);
      setIsSubmittingReject(true);
      await onReject(rejectNotes.trim());
      onOpenChange(false);
    } catch (err: unknown) {
      const msg =
        err instanceof Error ? err.message : "Failed to reject seal proposal.";
      setErrorMsg(msg);
    } finally {
      setIsSubmittingReject(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto bg-card border-border">
        <DialogHeader>
          <div className="flex items-center gap-2">
            <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20">
              <ShieldCheck className="size-6" />
            </div>
            <div>
              <DialogTitle className="text-xl text-foreground">
                Auditor Statutory Review & Hedera Consensus Anchor
              </DialogTitle>
              <DialogDescription className="text-xs text-muted-foreground">
                Independent examination of Controller close proposal (Step 2 of
                2).
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        {publishedSeal ? (
          <div className="space-y-4 py-4 text-center">
            <div className="size-14 rounded-full bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 flex items-center justify-center mx-auto border border-emerald-500/30">
              <Sparkles className="size-8" />
            </div>

            <div className="space-y-1">
              <h3 className="text-lg font-bold text-foreground">
                Period Cryptographically Sealed & Anchored to Hedera!
              </h3>
              <p className="text-xs text-muted-foreground max-w-md mx-auto">
                Consensus quorum (2-of-2 dual custody) achieved. 303-byte
                canonical seal dispatched and accepted by Hedera Consensus
                Service.
              </p>
            </div>

            <Card className="bg-muted/40 border-border text-left font-mono text-xs">
              <CardContent className="p-4 space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">HCS Topic ID:</span>
                  <span className="font-semibold text-foreground">
                    {publishedSeal.topic_id}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">
                    Consensus Timestamp:
                  </span>
                  <span className="font-semibold text-foreground">
                    {publishedSeal.consensus_timestamp}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">
                    Sequence Number:
                  </span>
                  <span className="font-semibold text-foreground">
                    #{publishedSeal.sequence_number}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">Merkle Root:</span>
                  <span className="text-foreground">
                    {truncateHash(publishedSeal.root, 10, 8)}
                  </span>
                </div>
              </CardContent>
            </Card>

            <DialogFooter className="flex items-center justify-between gap-2 pt-2 border-t border-border">
              {publishedSeal.topic_id && (
                <a
                  href={
                    getHashscanUrl({
                      topicId: publishedSeal.topic_id,
                      consensusTimestamp: publishedSeal.consensus_timestamp,
                    }) || "#"
                  }
                  target="_blank"
                  rel="noreferrer"
                  className="text-xs font-mono text-primary flex items-center gap-1 hover:underline"
                >
                  <span>Verify on HashScan Explorer</span>
                  <ArrowUpRight className="size-3.5" />
                </a>
              )}
              <Button
                type="button"
                onClick={() => onOpenChange(false)}
                className="text-xs h-9"
              >
                Close Portal
              </Button>
            </DialogFooter>
          </div>
        ) : (
          <div className="space-y-4 py-2">
            {errorMsg && (
              <div className="p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-xs text-destructive flex items-center gap-2">
                <AlertTriangle className="size-4 shrink-0" />
                <span>{errorMsg}</span>
              </div>
            )}

            {/* Financial Summary Card */}
            <Card className="bg-muted/40 border-border">
              <CardContent className="p-4 space-y-3">
                <div className="flex items-center justify-between text-xs">
                  <span className="text-muted-foreground font-medium">
                    Accounting Period
                  </span>
                  <span className="font-mono font-semibold text-foreground">
                    {statement.period_start} → {statement.period_end}
                  </span>
                </div>

                <div className="grid grid-cols-2 gap-3 pt-1 border-t border-border text-xs">
                  <div>
                    <div className="text-muted-foreground">
                      Total Journal Entries
                    </div>
                    <div className="text-base font-bold text-foreground mt-0.5">
                      {statement.entry_count}
                    </div>
                  </div>
                  <div>
                    <div className="text-muted-foreground">
                      Turnover (Gross Debits)
                    </div>
                    <div className="text-base font-bold text-foreground mt-0.5">
                      {formatCurrency(statement.total_debits_minor)}
                    </div>
                  </div>
                </div>

                <div className="space-y-1.5 pt-2 border-t border-border text-xs font-mono">
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground">
                      Ledger Merkle Root:
                    </span>
                    <span
                      className="text-foreground font-semibold"
                      title={statement.ledger_root}
                    >
                      {truncateHash(statement.ledger_root, 12, 10)}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground">
                      Statement Hash:
                    </span>
                    <span
                      className="text-foreground font-semibold"
                      title={statementHash}
                    >
                      {truncateHash(statementHash, 12, 10)}
                    </span>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Approver 1 Status Confirmation */}
            <div className="p-3 bg-muted/30 border border-border rounded-lg space-y-1.5 text-xs">
              <div className="flex items-center justify-between">
                <span className="font-semibold text-foreground">
                  Dual-Custody Status
                </span>
                <span className="font-mono text-emerald-600 dark:text-emerald-400 font-semibold flex items-center gap-1">
                  <CheckCircle2 className="size-3.5" />
                  Controller Signed
                </span>
              </div>
              <p className="text-[11px] text-muted-foreground">
                The Financial Controller has signed this canonical statement. Your
                statutory co-signature as {currentUser?.name || "Independent Auditor"} will complete the 2-of-2 multi-sig quorum and
                anchor the seal to Hedera Consensus Service.
              </p>
            </div>

            {/* Rejection form input */}
            {isRejecting ? (
              <div className="space-y-2 p-3 rounded-lg border border-destructive/30 bg-destructive/5 text-xs">
                <label className="font-semibold text-destructive block">
                  Auditor Rejection Notes:
                </label>
                <textarea
                  value={rejectNotes}
                  onChange={(e) => setRejectNotes(e.target.value)}
                  placeholder="Explain reason for rejecting the seal proposal (e.g. unverified invoice, balance discrepancy)..."
                  className="w-full h-20 p-2 text-xs bg-background border border-border rounded-md font-mono focus:outline-none focus:ring-1 focus:ring-destructive"
                />
                <div className="flex justify-end gap-2 pt-1">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setIsRejecting(false)}
                    disabled={isSubmittingReject}
                    className="h-7 text-xs"
                  >
                    Cancel
                  </Button>
                  <Button
                    variant="destructive"
                    size="sm"
                    onClick={handleConfirmReject}
                    disabled={isSubmittingReject}
                    className="h-7 text-xs gap-1"
                  >
                    {isSubmittingReject ? (
                      <>
                        <Loader2 className="size-3 animate-spin" />
                        <span>Rejecting...</span>
                      </>
                    ) : (
                      <>
                        <XCircle className="size-3" />
                        <span>Confirm Rejection</span>
                      </>
                    )}
                  </Button>
                </div>
              </div>
            ) : (
              <div className="flex items-center justify-between pt-2 border-t border-border">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setIsRejecting(true)}
                  disabled={isSigning}
                  className="text-destructive hover:bg-destructive/10 border-destructive/30 text-xs h-8 gap-1"
                >
                  <XCircle className="size-3.5" />
                  <span>Reject & Request Changes</span>
                </Button>

                <Button
                  size="sm"
                  onClick={handleAuditorSignAndPublish}
                  disabled={isSigning}
                  className="gap-1.5 h-8 text-xs bg-emerald-600 hover:bg-emerald-700 text-white font-semibold"
                >
                  {isSigning ? (
                    <>
                      <Loader2 className="size-3.5 animate-spin" />
                      <span>Sealing on Hedera...</span>
                    </>
                  ) : (
                    <>
                      <Lock className="size-3.5" />
                      <span>Co-Sign & Anchor to Hedera</span>
                    </>
                  )}
                </Button>
              </div>
            )}
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
