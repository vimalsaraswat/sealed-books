import { Badge } from "../ui/badge";
import { Button } from "../ui/button";
import { Card, CardContent } from "../ui/card";
import { Select } from "../ui/select";
import type { Period, PeriodStatement, SealRecord } from "../../types";
import {
  Lock,
  ShieldCheck,
  ArrowUpRight,
  FileSpreadsheet,
  Plus,
  BookOpen,
  Scale,
  Award,
  Bug,
  Calendar,
  Clock,
  AlertTriangle,
} from "lucide-react";

export interface PeriodHeaderProps {
  period: Period;
  allPeriods?: Period[];
  statement?: PeriodStatement | null;
  seal?: SealRecord | null;
  userRole?: string;
  onSelectPeriod?: (periodId: string) => void;
  onOpenNewEntryDialog?: () => void;
  onOpenNewPeriodDialog?: () => void;
  onOpenAccountsDialog?: () => void;
  onOpenTrialBalanceDialog?: () => void;
  onOpenCertificateDialog?: () => void;
  onOpenTamperDialog?: () => void;
  onOpenCloseDialog?: () => void;
  onOpenAuditorReviewDialog?: () => void;
  onOpenVerifyDialog?: () => void;
  isVerifying?: boolean;
}

export function PeriodHeader({
  period,
  allPeriods = [],
  statement: _statement,
  seal,
  userRole = "controller",
  onSelectPeriod,
  onOpenNewEntryDialog,
  onOpenNewPeriodDialog,
  onOpenAccountsDialog,
  onOpenTrialBalanceDialog,
  onOpenCertificateDialog,
  onOpenTamperDialog,
  onOpenCloseDialog,
  onOpenAuditorReviewDialog,
  onOpenVerifyDialog,
  isVerifying = false,
}: PeriodHeaderProps) {
  const isSealed = period.status === "sealed";
  const isAuditor = userRole === "auditor";
  const isStaff = userRole === "staff";
  const isPendingAuditor = seal?.dispatch_status === "pending_auditor";
  const isRejected = seal?.dispatch_status === "rejected";

  return (
    <Card className="border-border bg-card shadow-xs">
      <CardContent className="p-6">
        {/* Top Bar: Title, Period Selector, Actions */}
        <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
          <div className="space-y-1.5">
            <div className="flex flex-wrap items-center gap-3">
              <h1 className="text-2xl font-bold tracking-tight text-foreground">
                {period.entity}
              </h1>
              <Badge variant={isSealed ? "sealed" : "default"}>
                {isSealed ? (
                  <span className="flex items-center gap-1.5 font-mono">
                    <Lock className="size-3" /> SEALED
                  </span>
                ) : (
                  <span className="flex items-center gap-1.5 font-mono">
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

              {allPeriods.length > 1 && onSelectPeriod && (
                <div className="w-auto inline-block">
                  <Select
                    value={period.id}
                    onChange={(e) => onSelectPeriod(e.target.value)}
                    className="h-8 text-xs font-mono bg-background"
                  >
                    {allPeriods.map((p) => (
                      <option key={p.id} value={p.id}>
                        {p.start_date} → {p.end_date} ({p.status})
                      </option>
                    ))}
                  </Select>
                </div>
              )}
            </div>

            <p className="text-xs text-muted-foreground font-mono flex items-center gap-2">
              <span>
                Period: {period.start_date} to {period.end_date}
              </span>
              <span>•</span>
              <span>ID: {period.id}</span>
            </p>
          </div>

          {/* Action Buttons */}
          <div className="flex flex-wrap items-center gap-2">
            {!isAuditor && onOpenNewPeriodDialog && (
              <Button
                variant="outline"
                size="sm"
                onClick={onOpenNewPeriodDialog}
                className="text-xs gap-1.5 h-9"
              >
                <Calendar className="size-3.5" />
                <span>+ New Period</span>
              </Button>
            )}

            {isSealed ? (
              <>
                {seal?.topic_id && (
                  <a
                    href={
                      seal.consensus_timestamp
                        ? `https://hashscan.io/testnet/transaction/${seal.consensus_timestamp}`
                        : `https://hashscan.io/testnet/topic/${seal.topic_id}`
                    }
                    target="_blank"
                    rel="noreferrer"
                    className="inline-flex items-center gap-1 text-xs text-primary hover:underline bg-primary/10 border border-primary/20 px-2.5 py-1.5 rounded-md font-mono h-9 transition-colors"
                  >
                    <span>HCS #{seal.sequence_number || 1}</span>
                    <ArrowUpRight className="size-3" />
                  </a>
                )}

                {onOpenCertificateDialog && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={onOpenCertificateDialog}
                    className="text-xs gap-1.5 h-9"
                  >
                    <Award className="size-3.5 text-primary" />
                    <span>Audit Certificate</span>
                  </Button>
                )}

                {onOpenTamperDialog && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={onOpenTamperDialog}
                    className="text-xs gap-1.5 h-9 border-destructive/30 text-destructive hover:bg-destructive/10"
                  >
                    <Bug className="size-3.5 text-destructive" />
                    <span>Simulate Tamper</span>
                  </Button>
                )}

                <Button
                  variant="default"
                  onClick={onOpenVerifyDialog}
                  disabled={isVerifying}
                  className="font-medium h-9 text-xs gap-1.5 bg-emerald-600 hover:bg-emerald-700 text-white dark:bg-emerald-600 dark:hover:bg-emerald-500"
                >
                  <ShieldCheck className="size-4" />
                  <span>
                    {isVerifying ? "Verifying..." : "Verify on Hedera"}
                  </span>
                </Button>
              </>
            ) : (
              <>
                {/* Non-sealed actions */}
                {!isAuditor && onOpenNewEntryDialog && (
                  <Button
                    variant="outline"
                    onClick={onOpenNewEntryDialog}
                    className="font-medium h-9 text-xs gap-1.5"
                  >
                    <Plus className="size-4" />
                    <span>New Journal Entry</span>
                  </Button>
                )}

                {/* Auditor Review Action */}
                {isAuditor && isPendingAuditor && onOpenAuditorReviewDialog && (
                  <Button
                    variant="default"
                    onClick={onOpenAuditorReviewDialog}
                    className="bg-emerald-600 hover:bg-emerald-700 text-white font-medium h-9 text-xs gap-1.5 animate-pulse"
                  >
                    <ShieldCheck className="size-4" />
                    <span>Review Audit & Anchor Seal</span>
                  </Button>
                )}

                {/* Controller Close / Dispatch / Findings Action */}
                {!isAuditor && onOpenCloseDialog && (
                  <Button
                    variant="default"
                    onClick={onOpenCloseDialog}
                    disabled={isStaff}
                    title={
                      isStaff
                        ? "Closing restricted to Controllers and Admins"
                        : undefined
                    }
                    className="font-medium h-9 text-xs gap-1.5"
                  >
                    <Lock className="size-3.5" />
                    <span>
                      {isRejected
                        ? "Review Auditor Findings"
                        : isPendingAuditor
                          ? "View Audit Dispatch"
                          : "Close Period"}
                    </span>
                  </Button>
                )}
              </>
            )}
          </div>
        </div>

        {/* Middle Tool Bar: Quick Links to Accounts & Trial Balance */}
        <div className="mt-4 pt-3 border-t border-border flex flex-wrap items-center justify-between gap-3 text-xs">
          <div className="flex items-center gap-2">
            {onOpenAccountsDialog && (
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={onOpenAccountsDialog}
                className="text-xs h-7 gap-1.5 text-muted-foreground hover:text-foreground"
              >
                <BookOpen className="size-3.5" />
                <span>Chart of Accounts</span>
              </Button>
            )}
            {onOpenTrialBalanceDialog && (
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={onOpenTrialBalanceDialog}
                className="text-xs h-7 gap-1.5 text-muted-foreground hover:text-foreground"
              >
                <Scale className="size-3.5" />
                <span>Trial Balance</span>
              </Button>
            )}
          </div>

          <div className="text-[11px] text-muted-foreground font-mono">
            Immutable Double-Entry Ledger Engine • Hedera Consensus Seal Quorum
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
