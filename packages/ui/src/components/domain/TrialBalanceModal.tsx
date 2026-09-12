import { useMemo } from "react";
import { CheckCircle2, Scale } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "../ui/dialog";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../ui/table";
import { Badge } from "../ui/badge";
import { formatCurrency } from "../../lib/utils";
import type { Account, Entry, Period } from "../../types";

export interface TrialBalanceModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  period: Period;
  accounts: Account[];
  entries: Entry[];
}

export function TrialBalanceModal({
  open,
  onOpenChange,
  period,
  accounts,
  entries,
}: TrialBalanceModalProps) {
  // Aggregate debits and credits per account
  const accountTotals = useMemo(() => {
    const map = new Map<string, { debits: number; credits: number }>();

    for (const entry of entries) {
      for (const line of entry.lines) {
        const current = map.get(line.account_id) || { debits: 0, credits: 0 };
        if (line.direction === "debit") {
          current.debits += line.amount_minor;
        } else {
          current.credits += line.amount_minor;
        }
        map.set(line.account_id, current);
      }
    }

    return accounts
      .map((acc) => {
        const totals = map.get(acc.id) || { debits: 0, credits: 0 };
        return {
          account: acc,
          debits: totals.debits,
          credits: totals.credits,
          hasActivity: totals.debits > 0 || totals.credits > 0,
        };
      })
      .filter((item) => item.hasActivity);
  }, [accounts, entries]);

  const totalDebits = accountTotals.reduce((sum, item) => sum + item.debits, 0);
  const totalCredits = accountTotals.reduce(
    (sum, item) => sum + item.credits,
    0,
  );
  const isBalanced = totalDebits === totalCredits && totalDebits > 0;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-3xl max-h-[85vh] overflow-y-auto bg-card border-border">
        <DialogHeader>
          <div className="flex items-center justify-between pr-6">
            <DialogTitle className="flex items-center gap-2 text-foreground text-lg">
              <Scale className="size-5 text-primary" />
              <span>Trial Balance Sheet</span>
            </DialogTitle>
            <Badge variant="outline" className="font-mono text-xs">
              {period.start_date} → {period.end_date}
            </Badge>
          </div>
          <DialogDescription className="text-xs text-muted-foreground">
            Account activity breakdown confirming total debits equal total
            credits for{" "}
            <span className="font-semibold text-foreground">
              {period.entity}
            </span>
            .
          </DialogDescription>
        </DialogHeader>

        <div className="rounded-lg border border-border overflow-hidden bg-card">
          <Table>
            <TableHeader className="bg-muted/50">
              <TableRow>
                <TableHead className="w-[80px] text-xs">Code</TableHead>
                <TableHead className="text-xs">Account Name</TableHead>
                <TableHead className="w-[100px] text-xs">Class</TableHead>
                <TableHead className="w-[140px] text-right text-xs">
                  Debit Total
                </TableHead>
                <TableHead className="w-[140px] text-right text-xs">
                  Credit Total
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {accountTotals.length === 0 ? (
                <TableRow>
                  <TableCell
                    colSpan={5}
                    className="text-center py-8 text-xs text-muted-foreground"
                  >
                    No transactions recorded for this period yet.
                  </TableCell>
                </TableRow>
              ) : (
                accountTotals.map((row) => (
                  <TableRow
                    key={row.account.id}
                    className="hover:bg-muted/40 text-xs"
                  >
                    <TableCell className="font-mono font-semibold text-foreground py-2.5">
                      {row.account.code}
                    </TableCell>
                    <TableCell className="font-medium text-foreground py-2.5">
                      {row.account.name}
                    </TableCell>
                    <TableCell className="py-2.5">
                      <span className="text-[10px] uppercase tracking-wider text-muted-foreground font-semibold">
                        {row.account.account_type}
                      </span>
                    </TableCell>
                    <TableCell className="font-mono text-right font-medium text-foreground py-2.5">
                      {row.debits > 0 ? formatCurrency(row.debits) : "—"}
                    </TableCell>
                    <TableCell className="font-mono text-right font-medium text-foreground py-2.5">
                      {row.credits > 0 ? formatCurrency(row.credits) : "—"}
                    </TableCell>
                  </TableRow>
                ))
              )}
              {/* Grand Totals */}
              <TableRow className="bg-muted/60 font-semibold border-t-2 border-border">
                <TableCell
                  colSpan={3}
                  className="text-right text-xs uppercase tracking-wider text-foreground"
                >
                  Grand Totals
                </TableCell>
                <TableCell className="font-mono text-right text-xs text-foreground font-bold">
                  {formatCurrency(totalDebits)}
                </TableCell>
                <TableCell className="font-mono text-right text-xs text-foreground font-bold">
                  {formatCurrency(totalCredits)}
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>

        {/* Audit Equilibrium Confirmation */}
        <div
          className={`flex items-center justify-between rounded-lg border p-3 text-xs ${
            isBalanced
              ? "bg-emerald-500/10 border-emerald-500/20 text-emerald-700 dark:text-emerald-400"
              : "bg-amber-500/10 border-amber-500/20 text-amber-700 dark:text-amber-400"
          }`}
        >
          <div className="flex items-center gap-2">
            <CheckCircle2 className="size-4" />
            <span className="font-medium">
              {isBalanced
                ? "Ledger Equilibrium Maintained (Σ Debits = Σ Credits)"
                : "Ledger Out of Balance or Zero Turnover"}
            </span>
          </div>
          <span className="font-mono text-[11px]">
            Δ = {formatCurrency(Math.abs(totalDebits - totalCredits))}
          </span>
        </div>
      </DialogContent>
    </Dialog>
  );
}
