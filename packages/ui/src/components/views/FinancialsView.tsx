import { useState, useMemo } from "react";
import {
  Scale,
  BookOpen,
  Download,
  Plus,
  CheckCircle2,
  AlertTriangle,
  Layers,
} from "lucide-react";
import { Button } from "../ui/button";
import { Badge } from "../ui/badge";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "../ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../ui/table";
import { formatCurrency } from "../../lib/utils";
import { useAuth } from "../../context/AuthContext";
import { useLedger } from "../../context/LedgerContext";
import { useModals } from "../../context/ModalContext";
import type { Account, AccountType } from "../../types";

export function FinancialsView() {
  const { userRole } = useAuth();
  const { currentPeriod, accounts, entries } = useLedger();
  const { setIsAccountsOpen } = useModals();

  const [activeSubTab, setActiveSubTab] = useState<"trial_balance" | "chart_of_accounts">("trial_balance");

  const isAuditor = userRole === "auditor";

  // Aggregate debits and credits per account for the Trial Balance
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

    return accounts.map((acc) => {
      const totals = map.get(acc.id) || { debits: 0, credits: 0 };
      const isDebitNormal = acc.account_type === "asset" || acc.account_type === "expense";
      const netBalance = isDebitNormal
        ? totals.debits - totals.credits
        : totals.credits - totals.debits;

      return {
        account: acc,
        debits: totals.debits,
        credits: totals.credits,
        netBalance,
        hasActivity: totals.debits > 0 || totals.credits > 0,
      };
    });
  }, [accounts, entries]);

  const totalDebits = useMemo(
    () => accountTotals.reduce((sum, item) => sum + item.debits, 0),
    [accountTotals]
  );
  const totalCredits = useMemo(
    () => accountTotals.reduce((sum, item) => sum + item.credits, 0),
    [accountTotals]
  );
  const isBalanced = totalDebits === totalCredits && totalDebits > 0;

  // Group accounts by financial category for Chart of Accounts
  const accountsByCategory = useMemo(() => {
    const categories: Record<AccountType, Account[]> = {
      asset: [],
      liability: [],
      equity: [],
      revenue: [],
      expense: [],
    };
    for (const acc of accounts) {
      if (categories[acc.account_type]) {
        categories[acc.account_type].push(acc);
      }
    }
    return categories;
  }, [accounts]);

  // Export Trial Balance to CSV
  const handleExportCSV = () => {
    if (!currentPeriod) return;
    const headers = ["Account Code", "Account Name", "Type", "Debit Total ($)", "Credit Total ($)", "Net Balance ($)"];
    const rows = accountTotals.map((item) => [
      item.account.code,
      `"${item.account.name.replace(/"/g, '""')}"`,
      item.account.account_type,
      (item.debits / 100).toFixed(2),
      (item.credits / 100).toFixed(2),
      (item.netBalance / 100).toFixed(2),
    ]);

    rows.push([
      "TOTAL",
      "All Accounts Balanced",
      "Summary",
      (totalDebits / 100).toFixed(2),
      (totalCredits / 100).toFixed(2),
      "0.00",
    ]);

    const csvContent = "data:text/csv;charset=utf-8," + [headers.join(","), ...rows.map((e) => e.join(","))].join("\n");
    const encodedUri = encodeURI(csvContent);
    const link = document.createElement("a");
    link.setAttribute("href", encodedUri);
    link.setAttribute("download", `trial_balance_${currentPeriod.id}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
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
          Please select an accounting period to review financial statements.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* =========================================================
          SUB-NAV SWITCHER & ACTION STRIP
          ========================================================= */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border/80 pb-4">
        {/* Toggle Pills */}
        <div className="inline-flex h-9 items-center rounded-lg bg-muted/60 p-1 text-muted-foreground border border-border/50">
          <button
            onClick={() => setActiveSubTab("trial_balance")}
            className={`flex items-center gap-1.5 px-3 py-1 text-xs font-medium rounded-md transition-all ${
              activeSubTab === "trial_balance"
                ? "bg-background text-foreground shadow-xs"
                : "hover:text-foreground"
            }`}
          >
            <Scale className="size-3.5" />
            <span>Trial Balance Sheet</span>
          </button>
          <button
            onClick={() => setActiveSubTab("chart_of_accounts")}
            className={`flex items-center gap-1.5 px-3 py-1 text-xs font-medium rounded-md transition-all ${
              activeSubTab === "chart_of_accounts"
                ? "bg-background text-foreground shadow-xs"
                : "hover:text-foreground"
            }`}
          >
            <BookOpen className="size-3.5" />
            <span>Chart of Accounts</span>
          </button>
        </div>

        {/* Action Buttons */}
        <div className="flex items-center gap-2">
          {activeSubTab === "trial_balance" ? (
            <Button
              variant="outline"
              size="sm"
              onClick={handleExportCSV}
              className="h-8 gap-1.5 text-xs"
            >
              <Download className="size-3.5 text-muted-foreground" />
              <span>Export CSV</span>
            </Button>
          ) : (
            !isAuditor && (
              <Button
                variant="default"
                size="sm"
                onClick={() => setIsAccountsOpen(true)}
                className="h-8 gap-1.5 text-xs"
              >
                <Plus className="size-3.5" />
                <span>Add Account</span>
              </Button>
            )
          )}
        </div>
      </div>

      {/* =========================================================
          TAB 1: TRIAL BALANCE SHEET
          ========================================================= */}
      {activeSubTab === "trial_balance" && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-base font-bold text-foreground">
                Trial Balance for {currentPeriod.entity}
              </h3>
              <p className="text-xs text-muted-foreground font-mono">
                Accounting Period: {currentPeriod.start_date} to {currentPeriod.end_date}
              </p>
            </div>
            <div className="flex items-center gap-2">
              {isBalanced ? (
                <Badge
                  variant="outline"
                  className="bg-emerald-500/10 border-emerald-500/30 text-emerald-600 dark:text-emerald-400 gap-1.5 font-mono text-xs py-1"
                >
                  <CheckCircle2 className="size-3.5" />
                  <span>EQUILIBRIUM VERIFIED</span>
                </Badge>
              ) : (
                <Badge
                  variant="destructive"
                  className="gap-1.5 font-mono text-xs py-1"
                >
                  <AlertTriangle className="size-3.5" />
                  <span>IMBALANCE DETECTED</span>
                </Badge>
              )}
            </div>
          </div>

          <div className="rounded-lg border border-border bg-card overflow-hidden shadow-xs">
            <Table>
              <TableHeader className="bg-muted/50">
                <TableRow>
                  <TableHead className="w-[100px] text-xs">Code</TableHead>
                  <TableHead className="text-xs">Account Name</TableHead>
                  <TableHead className="w-[120px] text-xs">Type</TableHead>
                  <TableHead className="w-[140px] text-right text-xs">Debit Total</TableHead>
                  <TableHead className="w-[140px] text-right text-xs">Credit Total</TableHead>
                  <TableHead className="w-[140px] text-right text-xs">Net Balance</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {accountTotals.map((item) => {
                  return (
                    <TableRow key={item.account.id} className="hover:bg-muted/40 text-xs">
                      <TableCell className="font-mono font-bold text-foreground">
                        {item.account.code}
                      </TableCell>
                      <TableCell className="font-medium text-foreground">
                        {item.account.name}
                      </TableCell>
                      <TableCell>
                        <Badge
                          variant="outline"
                          className="capitalize text-[10px] font-mono py-0 text-muted-foreground"
                        >
                          {item.account.account_type}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-right font-mono text-foreground font-semibold">
                        {item.debits > 0 ? formatCurrency(item.debits) : "—"}
                      </TableCell>
                      <TableCell className="text-right font-mono text-foreground font-semibold">
                        {item.credits > 0 ? formatCurrency(item.credits) : "—"}
                      </TableCell>
                      <TableCell className="text-right font-mono text-foreground font-bold">
                        {formatCurrency(item.netBalance)}
                      </TableCell>
                    </TableRow>
                  );
                })}

                {/* Sticky Summary Row */}
                <TableRow className="bg-muted/60 font-bold border-t-2 border-border text-xs">
                  <TableCell colSpan={3} className="text-foreground uppercase font-mono tracking-wider">
                    Total Equilibrium Summary
                  </TableCell>
                  <TableCell className="text-right font-mono text-primary font-bold text-sm">
                    {formatCurrency(totalDebits)}
                  </TableCell>
                  <TableCell className="text-right font-mono text-primary font-bold text-sm">
                    {formatCurrency(totalCredits)}
                  </TableCell>
                  <TableCell className="text-right font-mono text-emerald-600 dark:text-emerald-400 font-bold">
                    $0.00
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </div>
        </div>
      )}

      {/* =========================================================
          TAB 2: CHART OF ACCOUNTS EXPLORER
          ========================================================= */}
      {activeSubTab === "chart_of_accounts" && (
        <div className="space-y-6">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-base font-bold text-foreground">
                Chart of Accounts
              </h3>
              <p className="text-xs text-muted-foreground">
                General ledger structure and account classifications for {currentPeriod.entity}
              </p>
            </div>
            <span className="text-xs font-mono text-muted-foreground">
              {accounts.length} Total Accounts
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {/* Category Cards */}
            {(
              [
                {
                  type: "asset",
                  title: "Assets",
                  range: "1000 - 1999",
                  desc: "Cash, accounts receivable, prepayments, and equipment",
                  accounts: accountsByCategory.asset,
                },
                {
                  type: "liability",
                  title: "Liabilities",
                  range: "2000 - 2999",
                  desc: "Accounts payable, accrued liabilities, and debts",
                  accounts: accountsByCategory.liability,
                },
                {
                  type: "equity",
                  title: "Equity",
                  range: "3000 - 3999",
                  desc: "Share capital, retained earnings, and founder contributions",
                  accounts: accountsByCategory.equity,
                },
                {
                  type: "revenue",
                  title: "Revenue",
                  range: "4000 - 4999",
                  desc: "Sales income, subscription fees, and consulting earnings",
                  accounts: accountsByCategory.revenue,
                },
                {
                  type: "expense",
                  title: "Expenses",
                  range: "5000 - 5999",
                  desc: "Office lease, payroll, cloud servers, and operations",
                  accounts: accountsByCategory.expense,
                },
              ] as const
            ).map((cat) => (
              <Card key={cat.type} className="border-border shadow-xs">
                <CardHeader className="p-4 pb-2">
                  <div className="flex items-center justify-between">
                    <CardTitle className="text-sm font-bold text-foreground flex items-center gap-2">
                      <span>{cat.title}</span>
                      <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
                        {cat.range}
                      </span>
                    </CardTitle>
                    <span className="text-xs font-mono text-muted-foreground">
                      {cat.accounts.length} accounts
                    </span>
                  </div>
                  <CardDescription className="text-[11px] text-muted-foreground">
                    {cat.desc}
                  </CardDescription>
                </CardHeader>
                <CardContent className="p-4 pt-2">
                  <div className="rounded-md border border-border/60 overflow-hidden divide-y divide-border/50 text-xs">
                    {cat.accounts.length === 0 ? (
                      <div className="p-3 text-center text-muted-foreground text-[11px]">
                        No {cat.title.toLowerCase()} accounts defined yet.
                      </div>
                    ) : (
                      cat.accounts.map((acc) => (
                        <div
                          key={acc.id}
                          className="p-2.5 flex items-center justify-between hover:bg-muted/30 transition-colors"
                        >
                          <div className="flex items-center gap-2">
                            <span className="font-mono font-bold text-foreground">
                              {acc.code}
                            </span>
                            <span className="text-foreground font-medium">
                              {acc.name}
                            </span>
                          </div>
                          <span className="text-[10px] uppercase font-mono text-muted-foreground">
                            {acc.account_type}
                          </span>
                        </div>
                      ))
                    )}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
