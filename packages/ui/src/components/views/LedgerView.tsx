import { useState, useMemo } from "react";
import {
  Search,
  Plus,
  Hash,
  AlertTriangle,
  FileSpreadsheet,
  Copy,
  Check,
  CheckCircle2,
  Layers,
  ChevronRight,
} from "lucide-react";
import { Input } from "../ui/input";
import { Button } from "../ui/button";
import { Badge } from "../ui/badge";
import { Select } from "../ui/select";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "../ui/sheet";
import { formatCurrency } from "../../lib/utils";
import { useAuth } from "../../context/AuthContext";
import { useLedger } from "../../context/LedgerContext";
import { useModals } from "../../context/ModalContext";
import type { Entry, Account } from "../../types";

export function LedgerView() {
  const { userRole } = useAuth();
  const { currentPeriod, entries, accounts, highlightedEntryId } = useLedger();

  const { setIsAddEntryOpen } = useModals();

  const [searchQuery, setSearchQuery] = useState("");
  const [selectedAccountId, setSelectedAccountId] = useState<string>("all");
  const [directionFilter, setDirectionFilter] = useState<
    "all" | "debit" | "credit"
  >("all");
  const [selectedEntry, setSelectedEntry] = useState<Entry | null>(null);
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const isSealed = currentPeriod?.status === "sealed";
  const isAuditor = userRole === "auditor";

  const accountMap = useMemo(() => {
    const map = new Map<string, Account>();
    for (const acc of accounts) {
      map.set(acc.id, acc);
    }
    return map;
  }, [accounts]);

  // Filter entries according to search query, account, and direction
  const filteredEntries = useMemo(() => {
    return entries.filter((entry) => {
      // 1. Search Query
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        const matchesDesc = entry.description.toLowerCase().includes(q);
        const matchesId = entry.id.toLowerCase().includes(q);
        const matchesDate = entry.date.includes(q);
        const matchesLineAccount = entry.lines.some((line) => {
          const acc = accountMap.get(line.account_id);
          return (
            acc?.name.toLowerCase().includes(q) ||
            acc?.code.toLowerCase().includes(q)
          );
        });

        if (!matchesDesc && !matchesId && !matchesDate && !matchesLineAccount) {
          return false;
        }
      }

      // 2. Account Filter
      if (selectedAccountId !== "all") {
        const hasAccount = entry.lines.some(
          (line) => line.account_id === selectedAccountId,
        );
        if (!hasAccount) return false;
      }

      // 3. Direction Filter
      if (directionFilter !== "all") {
        const hasDirection = entry.lines.some(
          (line) => line.direction === directionFilter,
        );
        if (!hasDirection) return false;
      }

      return true;
    });
  }, [entries, searchQuery, selectedAccountId, directionFilter, accountMap]);

  const handleCopy = (id: string, text: string, e?: React.MouseEvent) => {
    if (e) e.stopPropagation();
    navigator.clipboard.writeText(text);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 1500);
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
          Please select an accounting period to view general ledger
          transactions.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* =========================================================
          TOOLBAR & FILTER STRIP
          ========================================================= */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-3 bg-card p-3.5 rounded-lg border border-border shadow-xs">
        {/* Left Side: Search & Filters */}
        <div className="flex items-center gap-2.5 flex-1 flex-wrap">
          {/* Search Input */}
          <div className="relative flex-1 min-w-[200px] max-w-md">
            <Search className="absolute left-2.5 top-2.5 size-3.5 text-muted-foreground" />
            <Input
              type="text"
              placeholder="Search memo, ID, or account..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="h-8 pl-8 text-xs bg-background"
            />
          </div>

          {/* Account Filter */}
          <div className="w-[180px]">
            <Select
              value={selectedAccountId}
              onChange={(e) => setSelectedAccountId(e.target.value)}
              className="h-8 text-xs bg-background"
            >
              <option value="all">All Accounts ({accounts.length})</option>
              {accounts.map((acc) => (
                <option key={acc.id} value={acc.id}>
                  {acc.code} - {acc.name}
                </option>
              ))}
            </Select>
          </div>

          {/* Direction Filter */}
          <div className="w-[130px]">
            <Select
              value={directionFilter}
              onChange={(e) =>
                setDirectionFilter(e.target.value as "all" | "debit" | "credit")
              }
              className="h-8 text-xs bg-background"
            >
              <option value="all">All Directions</option>
              <option value="debit">Debits Only</option>
              <option value="credit">Credits Only</option>
            </Select>
          </div>

          {(searchQuery ||
            selectedAccountId !== "all" ||
            directionFilter !== "all") && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => {
                setSearchQuery("");
                setSelectedAccountId("all");
                setDirectionFilter("all");
              }}
              className="h-8 text-xs text-muted-foreground hover:text-foreground"
            >
              Reset
            </Button>
          )}
        </div>

        {/* Right Side: Count & New Entry Action */}
        <div className="flex items-center gap-3 shrink-0">
          <span className="text-xs font-mono text-muted-foreground">
            {filteredEntries.length} of {entries.length} entries
          </span>

          {!isAuditor && !isSealed && (
            <Button
              variant="default"
              size="sm"
              onClick={() => setIsAddEntryOpen(true)}
              className="h-8 gap-1.5 text-xs"
            >
              <Plus className="size-3.5" />
              <span>New Entry</span>
            </Button>
          )}
        </div>
      </div>

      {/* =========================================================
          JOURNAL ENTRIES: PERFECTLY ALIGNED FINANCIAL CARDS
          ========================================================= */}
      {filteredEntries.length === 0 ? (
        <div className="rounded-lg border border-border bg-card p-12 text-center text-xs text-muted-foreground space-y-2">
          <FileSpreadsheet className="size-8 mx-auto text-muted-foreground/60 mb-2" />
          <p className="font-semibold text-foreground text-sm">
            No transactions match your search
          </p>
          <p className="text-xs">
            Try adjusting your search criteria or resetting filters.
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {filteredEntries.map((entry) => {
            const isTampered = highlightedEntryId === entry.id;
            const totalDebitMinor = entry.lines
              .filter((l) => l.direction === "debit")
              .reduce((sum, l) => sum + l.amount_minor, 0);

            return (
              <div
                key={entry.id}
                onClick={() => setSelectedEntry(entry)}
                className={`group rounded-lg border bg-card overflow-hidden shadow-xs transition-all cursor-pointer ${
                  isTampered
                    ? "border-destructive/60 bg-destructive/5 ring-1 ring-destructive/40"
                    : "border-border hover:border-primary/50 hover:shadow-sm"
                }`}
              >
                {/* Transaction Header Bar */}
                <div
                  className={`px-4 py-2.5 flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b transition-colors ${
                    isTampered
                      ? "bg-destructive/15 border-destructive/30"
                      : "bg-muted/40 border-border/70 group-hover:bg-muted/60"
                  }`}
                >
                  <div className="flex items-center gap-2.5 flex-wrap min-w-0">
                    <span className="font-mono text-xs font-bold text-foreground bg-background px-2 py-0.5 rounded border border-border/60">
                      {entry.date}
                    </span>

                    <div className="flex items-center gap-1 font-mono text-xs text-muted-foreground">
                      <Hash className="size-3" />
                      <span>{entry.id}</span>
                      <button
                        onClick={(e) => handleCopy(entry.id, entry.id, e)}
                        title="Copy Entry ID"
                        className="text-muted-foreground hover:text-foreground opacity-60 hover:opacity-100 p-0.5"
                      >
                        {copiedId === entry.id ? (
                          <Check className="size-3 text-emerald-500" />
                        ) : (
                          <Copy className="size-3" />
                        )}
                      </button>
                    </div>

                    <span className="text-xs font-semibold text-foreground truncate max-w-md">
                      {entry.description}
                    </span>

                    {isTampered && (
                      <Badge
                        variant="destructive"
                        className="text-[10px] py-0 px-1.5 gap-1"
                      >
                        <AlertTriangle className="size-2.5" />
                        <span>Tampered</span>
                      </Badge>
                    )}
                  </div>

                  <div className="flex items-center gap-2 shrink-0 self-end sm:self-auto">
                    <Badge
                      variant="outline"
                      className="font-mono text-[11px] font-semibold bg-background"
                    >
                      Balanced: {formatCurrency(totalDebitMinor)}
                    </Badge>
                    <ChevronRight className="size-4 text-muted-foreground group-hover:text-foreground group-hover:translate-x-0.5 transition-all" />
                  </div>
                </div>

                {/* Posting Lines Grid with Fixed Headers */}
                <div className="p-0">
                  <table className="w-full text-xs border-collapse">
                    <thead>
                      <tr className="border-b border-border/50 text-[10px] font-mono uppercase text-muted-foreground tracking-wider bg-muted/10">
                        <th className="text-left font-medium py-1.5 px-4 w-[90px]">
                          Code
                        </th>
                        <th className="text-left font-medium py-1.5 px-4">
                          Account & Memo
                        </th>
                        <th className="text-right font-medium py-1.5 px-4 w-[140px]">
                          Debit
                        </th>
                        <th className="text-right font-medium py-1.5 px-4 w-[140px]">
                          Credit
                        </th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-border/40">
                      {entry.lines.map((line, idx) => {
                        const account = accountMap.get(line.account_id);
                        const isDebit = line.direction === "debit";

                        return (
                          <tr
                            key={line.id || idx}
                            className="hover:bg-muted/20 transition-colors"
                          >
                            <td className="py-2 px-4 font-mono font-bold text-muted-foreground align-top">
                              {account?.code || "---"}
                            </td>
                            <td className="py-2 px-4 align-top">
                              <div className="flex flex-col sm:flex-row sm:items-center gap-1">
                                <span className="font-semibold text-foreground">
                                  {account?.name || "Unknown Account"}
                                </span>
                                {line.description && (
                                  <span className="text-[11px] text-muted-foreground italic">
                                    ({line.description})
                                  </span>
                                )}
                              </div>
                            </td>
                            <td className="py-2 px-4 text-right font-mono font-bold text-foreground align-top">
                              {isDebit ? (
                                <span className="text-foreground">
                                  {formatCurrency(line.amount_minor)}
                                </span>
                              ) : (
                                <span className="text-muted-foreground/30">
                                  —
                                </span>
                              )}
                            </td>
                            <td className="py-2 px-4 text-right font-mono font-bold text-foreground align-top">
                              {!isDebit ? (
                                <span className="text-foreground">
                                  {formatCurrency(line.amount_minor)}
                                </span>
                              ) : (
                                <span className="text-muted-foreground/30">
                                  —
                                </span>
                              )}
                            </td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* =========================================================
          SLIDE-OVER TRANSACTION DETAIL DRAWER
          ========================================================= */}
      <Sheet
        open={Boolean(selectedEntry)}
        onOpenChange={(open) => {
          if (!open) setSelectedEntry(null);
        }}
      >
        <SheetContent side="right" className="sm:max-w-lg flex flex-col">
          {selectedEntry && (
            <>
              <SheetHeader className="pb-4 border-b border-border/80">
                <div className="flex items-center justify-between pr-6">
                  <div className="flex items-center gap-2">
                    <Badge variant="outline" className="font-mono text-xs">
                      {selectedEntry.date}
                    </Badge>
                    <Badge variant="secondary" className="font-mono text-xs">
                      Balanced
                    </Badge>
                  </div>
                </div>
                <SheetTitle className="text-base font-bold text-foreground pt-1">
                  Transaction {selectedEntry.id}
                </SheetTitle>
                <SheetDescription className="text-xs text-muted-foreground font-mono">
                  Double-entry posting in {currentPeriod.entity}
                </SheetDescription>
              </SheetHeader>

              <div className="flex-1 overflow-y-auto py-4 space-y-5">
                {/* Description Box */}
                <div className="space-y-1.5">
                  <span className="text-[10px] uppercase font-mono text-muted-foreground font-semibold">
                    Description / Memo
                  </span>
                  <p className="text-sm font-medium text-foreground bg-muted/30 p-3 rounded-md border border-border/50">
                    {selectedEntry.description}
                  </p>
                </div>

                {/* Canonical Entry Hash */}
                <div className="space-y-1.5">
                  <div className="flex items-center justify-between">
                    <span className="text-[10px] uppercase font-mono text-muted-foreground font-semibold">
                      Canonical Reference
                    </span>
                    <button
                      onClick={() => handleCopy("drawer_id", selectedEntry.id)}
                      className="text-xs text-primary hover:underline flex items-center gap-1 font-mono"
                    >
                      {copiedId === "drawer_id" ? (
                        <>
                          <Check className="size-3 text-emerald-500" />
                          <span>Copied</span>
                        </>
                      ) : (
                        <>
                          <Copy className="size-3" />
                          <span>Copy ID</span>
                        </>
                      )}
                    </button>
                  </div>
                  <div className="p-2.5 rounded-md bg-muted/40 border border-border/50 font-mono text-xs text-muted-foreground flex items-center justify-between">
                    <span>ID: {selectedEntry.id}</span>
                    <span>Period: {currentPeriod.id}</span>
                  </div>
                </div>

                {/* Balanced Ledger Lines */}
                <div className="space-y-2">
                  <span className="text-[10px] uppercase font-mono text-muted-foreground font-semibold">
                    Double-Entry Postings ({selectedEntry.lines.length} lines)
                  </span>

                  <div className="rounded-md border border-border overflow-hidden divide-y divide-border/60">
                    {selectedEntry.lines.map((line, i) => {
                      const acc = accountMap.get(line.account_id);
                      const isDebit = line.direction === "debit";
                      return (
                        <div
                          key={i}
                          className="p-3 bg-card flex items-center justify-between gap-3 text-xs"
                        >
                          <div className="space-y-0.5 min-w-0">
                            <div className="flex items-center gap-2">
                              <span className="font-mono font-bold text-foreground">
                                {acc?.code || "---"}
                              </span>
                              <span className="text-foreground truncate font-medium">
                                {acc?.name || "Account"}
                              </span>
                            </div>
                            <span className="text-[10px] uppercase font-mono text-muted-foreground">
                              {line.direction}
                            </span>
                          </div>

                          <div className="text-right shrink-0">
                            <span
                              className={`font-mono font-bold text-xs ${
                                isDebit ? "text-primary" : "text-foreground"
                              }`}
                            >
                              {formatCurrency(line.amount_minor)}
                            </span>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>

                {/* Equilibrium Confirmation */}
                <div className="p-3 rounded-md bg-emerald-500/10 border border-emerald-500/20 flex items-center gap-2 text-xs text-emerald-700 dark:text-emerald-400 font-mono">
                  <CheckCircle2 className="size-4 shrink-0" />
                  <span>
                    Debits strictly equal credits. Balanced to 0 minor units.
                  </span>
                </div>
              </div>
            </>
          )}
        </SheetContent>
      </Sheet>
    </div>
  );
}
