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
import { Badge } from "../ui/badge";
import { Card, CardContent } from "../ui/card";
import { formatCurrency, truncateHash } from "../../lib/utils";

import type { ProposeResponse, SealRecord, OrgMember } from "../../types";
import {
  Lock,
  CheckCircle2,
  Send,
  UserCheck,
  ShieldAlert,
  Loader2,
  AlertTriangle,
  Clock,
} from "lucide-react";

export interface SealProposalDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  proposal: ProposeResponse | null;
  seal?: SealRecord | null;
  onApprove: (data?: {
    approver_pubkey?: string;
    signature?: string;
    wallet_id?: string;
  }) => Promise<void>;
  onDispatch?: (auditorId: string) => Promise<void>;
  onPublish: () => Promise<SealRecord>;
  onSuccessClose?: () => void;
}

export function SealProposalDialog({
  open,
  onOpenChange,
  proposal,
  seal,
  onApprove,
  onDispatch,
  onPublish: _onPublish,
  onSuccessClose: _onSuccessClose,
}: SealProposalDialogProps) {
  const [approvals, setApprovals] = useState<
    Map<string, { signature: string; address: string }>
  >(new Map());
  const [signingId, setSigningId] = useState<string | null>(null);
  const [isDispatching, setIsDispatching] = useState(false);
  const [isDispatched, setIsDispatched] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const { currentUser, activeOrg, api } = useAuth();
  const controllerId = currentUser?.id || "controller";
  const [auditors, setAuditors] = useState<OrgMember[]>([]);
  const [selectedAuditorId, setSelectedAuditorId] = useState<string>("");


  // Reset state and load auditors only when dialog is first opened or activeOrg changes
  useEffect(() => {
    if (open) {
      setSigningId(null);
      setIsDispatching(false);
      setIsDispatched(seal?.dispatch_status === "pending_auditor");
      setErrorMsg(null);

      if (activeOrg && api) {
        api.getOrgMembers(activeOrg.id)
          .then((members: OrgMember[]) => {
            const auditList = members.filter((m: OrgMember) => m.role === "auditor");
            setAuditors(auditList);
            if (auditList.length > 0) {
              setSelectedAuditorId(auditList[0].user_id);
            }
          })
          .catch(console.error);
      }
    } else {
      setApprovals(new Map());
    }
  }, [open, activeOrg, api]);

  if (!proposal) return null;

  const { statement, statement_hash: statementHash } = proposal;

  const hasControllerSigned =
    approvals.has(controllerId) ||
    Boolean(seal?.approver_1_pubkey && seal?.approver_1_sig);

  const handleControllerSign = async () => {
    try {
      setErrorMsg(null);
      setSigningId(controllerId);

      const hashToSign = statementHash || seal?.statement_hash;
      if (!hashToSign) {
        throw new Error(
          "Missing statement hash to sign. Please re-propose the period close.",
        );
      }

      await onApprove();

      setApprovals((prev) => {
        const next = new Map(prev);
        next.set(controllerId, {
          signature: "Server Wallet Signed",
          address: currentUser?.eth_address || (seal?.approver_1_pubkey ? truncateHash(seal.approver_1_pubkey, 8, 6) : "Configured Server Wallet"),
        });
        return next;
      });
    } catch (err: unknown) {
      const msg =
        err instanceof Error ? err.message : "Failed to sign as Controller";
      setErrorMsg(msg);
    } finally {
      setSigningId(null);
    }
  };

  const handleDispatchToAuditor = async () => {
    if (!onDispatch) return;
    try {
      setErrorMsg(null);
      setIsDispatching(true);
      await onDispatch(selectedAuditorId || "");
      setIsDispatched(true);
    } catch (err: unknown) {
      const msg =
        err instanceof Error
          ? err.message
          : "Failed to dispatch seal proposal to auditor";
      setErrorMsg(msg);
    } finally {
      setIsDispatching(false);
    }
  };

  const isRejected = seal?.dispatch_status === "rejected";

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto bg-card border-border">
        <DialogHeader>
          <div className="flex items-center gap-2">
            <div className="p-2 rounded-lg bg-primary/10 text-primary border border-primary/20">
              <Lock className="size-6" />
            </div>
            <div>
              <DialogTitle className="text-xl text-foreground">
                Period Close & Cryptographic Seal Proposal
              </DialogTitle>
              <DialogDescription className="text-muted-foreground text-xs mt-1">
                Canonical multi-party dual-custody verification workflow (Step 1
                of 2).
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        <div className="space-y-4 py-2">
          {errorMsg && (
            <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-3 text-destructive text-xs flex items-center gap-2">
              <ShieldAlert className="size-4 shrink-0" />
              <span>{errorMsg}</span>
            </div>
          )}

          {isRejected && (
            <div className="p-3.5 rounded-lg border border-amber-500/30 bg-amber-500/10 space-y-2">
              <div className="flex items-center gap-2 text-amber-800 dark:text-amber-200 font-semibold text-xs">
                <AlertTriangle className="size-4 text-amber-500 shrink-0" />
                <span>Previous Proposal Rejected by External Auditor</span>
              </div>
              {seal?.auditor_notes && (
                <div className="text-xs font-mono text-muted-foreground bg-background/60 p-2.5 rounded border border-border/50">
                  <span className="font-sans font-semibold text-foreground">
                    Auditor Notes:{" "}
                  </span>
                  &ldquo;{seal.auditor_notes}&rdquo;
                </div>
              )}
              <p className="text-[11px] text-muted-foreground">
                Please review ledger entries, adjust as needed, and sign again
                to re-dispatch.
              </p>
            </div>
          )}

          {/* Statement Summary Card */}
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
                    Total Balanced Entries
                  </div>
                  <div className="text-base font-bold text-foreground mt-0.5">
                    {statement.entry_count}
                  </div>
                </div>
                <div>
                  <div className="text-muted-foreground">
                    Period Turnover (Debits)
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
                    className="text-foreground"
                    title={statement.ledger_root}
                  >
                    {truncateHash(statement.ledger_root, 12, 10)}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">
                    Canonical Statement Hash:
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

          {/* Dual-Custody Signature Matrix */}
          <div className="rounded-lg border border-border p-4 space-y-3 bg-card">
            <div className="flex items-center justify-between">
              <div className="font-semibold text-xs text-foreground uppercase tracking-wider">
                Multi-Party Signature Consensus Quorum
              </div>
              <Badge variant="outline" className="text-[10px] font-mono">
                2-of-2 Required
              </Badge>
            </div>

            <div className="space-y-2 text-xs">
              {/* Approver 1: Controller */}
              <div className="flex items-center justify-between p-3 rounded-lg border border-border bg-muted/40">
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="font-semibold text-foreground">
                      Approver 1: Controller
                    </span>
                    <Badge variant="secondary" className="text-[10px]">
                      You
                    </Badge>
                  </div>
                  <div className="text-[11px] font-mono text-muted-foreground">
                    Address: {currentUser?.eth_address ? truncateHash(currentUser.eth_address, 8, 6) : (seal?.approver_1_pubkey ? truncateHash(seal.approver_1_pubkey, 8, 6) : "Not connected")} (Server Wallet)
                  </div>
                </div>

                <div>
                  {hasControllerSigned ? (
                    <div className="flex items-center gap-1.5 text-emerald-600 dark:text-emerald-400 font-mono text-xs font-semibold">
                      <CheckCircle2 className="size-4" />
                      <span>Signed</span>
                    </div>
                  ) : (
                    <Button
                      size="sm"
                      onClick={handleControllerSign}
                      disabled={signingId === controllerId}
                      className="gap-1.5 h-8 text-xs"
                    >
                      {signingId === controllerId ? (
                        <>
                          <Loader2 className="size-3.5 animate-spin" />
                          <span>Signing...</span>
                        </>
                      ) : (
                        <>
                          <UserCheck className="size-3.5" />
                          <span>Sign Statement</span>
                        </>
                      )}
                    </Button>
                  )}
                </div>
              </div>

              {/* Approver 2: Auditor */}
              <div className="flex items-center justify-between p-3 rounded-lg border border-border bg-muted/40">
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="font-semibold text-foreground">
                      Approver 2: Independent Auditor
                    </span>
                    <Badge variant="outline" className="text-[10px]">
                      External
                    </Badge>
                  </div>
                  <div className="text-[11px] font-mono text-muted-foreground">
                    Role: Independent Statutory Auditor
                  </div>
                </div>

                <div>
                  {seal?.approver_2_sig ? (
                    <div className="flex items-center gap-1.5 text-emerald-600 dark:text-emerald-400 font-mono text-xs font-semibold">
                      <CheckCircle2 className="size-4" />
                      <span>Co-Signed</span>
                    </div>
                  ) : isDispatched ? (
                    <div className="flex items-center gap-1.5 text-amber-600 dark:text-amber-400 font-mono text-xs">
                      <Clock className="size-4 animate-pulse" />
                      <span>Awaiting Auditor Review</span>
                    </div>
                  ) : (
                    <span className="text-[11px] text-muted-foreground font-mono">
                      Awaiting Dispatch
                    </span>
                  )}
                </div>
              </div>
            </div>
          </div>

          {/* Controller Dispatch Step */}
          {hasControllerSigned && !isDispatched && !seal?.approver_2_sig && (
            <div className="rounded-lg border border-primary/20 bg-primary/5 p-4 space-y-3">
              <div className="flex items-start gap-3">
                <div className="p-1.5 rounded-full bg-primary/10 text-primary mt-0.5">
                  <Send className="size-4" />
                </div>
                <div className="space-y-1 flex-1">
                  <div className="text-xs font-semibold text-foreground">
                    Dispatch to External Auditor
                  </div>
                  <p className="text-[11px] text-muted-foreground">
                    Your Controller signature is recorded. Dispatch this seal
                    proposal to an independent statutory auditor for formal
                    dual-custody sign-off.
                  </p>
                </div>
              </div>

              {auditors.length > 0 ? (
                <div className="space-y-1.5 pt-1">
                  <label className="text-[11px] font-medium text-foreground">
                    Select Statutory Auditor:
                  </label>
                  <select
                    value={selectedAuditorId}
                    onChange={(e) => setSelectedAuditorId(e.target.value)}
                    className="w-full text-xs rounded-md border border-input bg-background px-3 py-1.5 text-foreground shadow-sm focus:outline-none focus:ring-1 focus:ring-ring"
                  >
                    {auditors.map((auditor) => (
                      <option key={auditor.user_id} value={auditor.user_id}>
                        {auditor.name} ({auditor.email})
                      </option>
                    ))}
                  </select>
                </div>
              ) : null}

              <div className="flex justify-end">
                <Button
                  size="sm"
                  onClick={handleDispatchToAuditor}
                  disabled={isDispatching}
                  className="gap-1.5 h-8 text-xs"
                >
                  {isDispatching ? (
                    <>
                      <Loader2 className="size-3.5 animate-spin" />
                      <span>Dispatching...</span>
                    </>
                  ) : (
                    <>
                      <Send className="size-3.5" />
                      <span>Dispatch to Auditor</span>
                    </>
                  )}
                </Button>
              </div>
            </div>
          )}

          {isDispatched && !seal?.approver_2_sig && (
            <div className="rounded-lg border border-border p-3.5 bg-muted/30 text-xs text-muted-foreground flex items-center gap-2">
              <Clock className="size-4 text-amber-500 shrink-0" />
              <span>
                Proposal dispatched to statutory auditor. Waiting for auditor to
                login, inspect trial balance, and co-sign.
              </span>
            </div>
          )}
        </div>

        <DialogFooter className="pt-2 border-t border-border">
          <Button
            variant="outline"
            size="sm"
            onClick={() => onOpenChange(false)}
            className="text-xs"
          >
            Close
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
