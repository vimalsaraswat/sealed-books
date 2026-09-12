import { useMemo, Fragment } from "react";
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
import type { Entry, Account } from "../../types";
import { AlertTriangle, Hash } from "lucide-react";

export interface LedgerTableProps {
  entries: Entry[];
  accounts: Account[];
  highlightedEntryId?: string | null;
}

export function LedgerTable({
  entries,
  accounts,
  highlightedEntryId,
}: LedgerTableProps) {
  const accountMap = useMemo(() => {
    const map = new Map<string, Account>();
    for (const acc of accounts) {
      map.set(acc.id, acc);
    }
    return map;
  }, [accounts]);

  return (
    <div className="rounded-lg border border-border bg-card overflow-hidden shadow-xs">
      <Table>
        <TableHeader>
          <TableRow className="bg-muted/50">
            <TableHead className="w-[120px] text-xs">Date</TableHead>
            <TableHead className="w-[180px] text-xs">Entry ID</TableHead>
            <TableHead className="text-xs">Description / Account</TableHead>
            <TableHead className="text-right w-[140px] text-xs">
              Debit
            </TableHead>
            <TableHead className="text-right w-[140px] text-xs">
              Credit
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {entries.length === 0 ? (
            <TableRow>
              <TableCell
                colSpan={5}
                className="text-center py-8 text-xs text-muted-foreground"
              >
                No accounting entries found for this period.
              </TableCell>
            </TableRow>
          ) : (
            entries.map((entry) => {
              const isTampered = highlightedEntryId === entry.id;
              return (
                <Fragment key={entry.id}>
                  {/* Entry Header Row */}
                  <TableRow
                    className={
                      isTampered
                        ? "bg-destructive/15 border-l-4 border-l-destructive font-medium"
                        : "bg-muted/30 hover:bg-muted/60"
                    }
                  >
                    <TableCell className="font-mono text-xs text-foreground font-semibold align-top py-2.5">
                      {entry.date}
                    </TableCell>
                    <TableCell className="font-mono text-xs text-muted-foreground align-top py-2.5">
                      <div className="flex items-center gap-1.5">
                        <Hash className="size-3 text-muted-foreground" />
                        <span>{entry.id}</span>
                        {isTampered && (
                          <Badge
                            variant="destructive"
                            className="ml-1 text-[10px] py-0 px-1.5"
                          >
                            <AlertTriangle className="size-2.5 mr-0.5" />
                            Tampered
                          </Badge>
                        )}
                      </div>
                    </TableCell>
                    <TableCell
                      colSpan={3}
                      className="text-sm font-semibold text-foreground py-2.5"
                    >
                      {entry.description}
                    </TableCell>
                  </TableRow>

                  {/* Transaction Lines Rows */}
                  {entry.lines.map((line) => {
                    const account = accountMap.get(line.account_id);
                    const isDebit = line.direction === "debit";
                    return (
                      <TableRow
                        key={line.id}
                        className={
                          isTampered
                            ? "bg-destructive/10 border-l-4 border-l-destructive/60 hover:bg-destructive/20"
                            : "hover:bg-muted/40"
                        }
                      >
                        <TableCell className="py-2"></TableCell>
                        <TableCell className="py-2"></TableCell>
                        <TableCell className="py-2 text-xs">
                          <div className="flex items-center gap-2 pl-4">
                            <span className="font-mono font-medium text-muted-foreground">
                              {account?.code || "---"}
                            </span>
                            <span className="text-foreground">
                              {account?.name ||
                                line.description ||
                                "Line detail"}
                            </span>
                          </div>
                        </TableCell>
                        <TableCell className="py-2 text-right font-mono text-xs text-emerald-600 dark:text-emerald-400 font-medium">
                          {isDebit ? formatCurrency(line.amount_minor) : ""}
                        </TableCell>
                        <TableCell className="py-2 text-right font-mono text-xs text-primary font-medium">
                          {!isDebit ? formatCurrency(line.amount_minor) : ""}
                        </TableCell>
                      </TableRow>
                    );
                  })}
                </Fragment>
              );
            })
          )}
        </TableBody>
      </Table>
    </div>
  );
}
