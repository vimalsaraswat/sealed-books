import { useState, useEffect } from "react";
import { AlertCircle, CheckCircle2, Plus, Trash2 } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "../ui/dialog";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { Select } from "../ui/select";
import { DatePicker } from "../ui/date-picker";
import { formatCurrency } from "../../lib/utils";
import type { Account, CreateEntryInput, Period } from "../../types";

interface LineDraft {
  id: string;
  account_id: string;
  direction: "debit" | "credit";
  amount_str: string;
  description: string;
}

export interface AddEntryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  period: Period;
  accounts: Account[];
  onSave: (entry: CreateEntryInput) => Promise<void>;
}

export function AddEntryDialog({
  open,
  onOpenChange,
  period,
  accounts,
  onSave,
}: AddEntryDialogProps) {
  const defaultAccount1 = accounts[0]?.id || "";
  const defaultAccount2 = accounts[1]?.id || accounts[0]?.id || "";

  const [date, setDate] = useState(period.start_date);
  const [description, setDescription] = useState("");
  const [lines, setLines] = useState<LineDraft[]>([
    {
      id: "1",
      account_id: defaultAccount1,
      direction: "debit",
      amount_str: "",
      description: "",
    },
    {
      id: "2",
      account_id: defaultAccount2,
      direction: "credit",
      amount_str: "",
      description: "",
    },
  ]);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Reset form when dialog opens
  useEffect(() => {
    if (open) {
      setDate(period.start_date);
      setDescription("");
      setLines([
        {
          id: "1",
          account_id: accounts[0]?.id || "",
          direction: "debit",
          amount_str: "",
          description: "",
        },
        {
          id: "2",
          account_id: accounts[1]?.id || accounts[0]?.id || "",
          direction: "credit",
          amount_str: "",
          description: "",
        },
      ]);
      setError(null);
    }
  }, [open, period, accounts]);

  const parseAmountMinor = (val: string): number => {
    const clean = val.replace(/[^0-9.]/g, "");
    const num = parseFloat(clean);
    if (isNaN(num) || num <= 0) return 0;
    return Math.round(num * 100);
  };

  const totalDebits = lines
    .filter((l) => l.direction === "debit")
    .reduce((acc, l) => acc + parseAmountMinor(l.amount_str), 0);

  const totalCredits = lines
    .filter((l) => l.direction === "credit")
    .reduce((acc, l) => acc + parseAmountMinor(l.amount_str), 0);

  const isBalanced = totalDebits > 0 && totalDebits === totalCredits;

  const handleAddLine = () => {
    const newId = String(Date.now());
    const fallbackAcc = accounts[0]?.id || "";
    setLines([
      ...lines,
      {
        id: newId,
        account_id: fallbackAcc,
        direction: "credit",
        amount_str: "",
        description: "",
      },
    ]);
  };

  const handleRemoveLine = (idx: number) => {
    if (lines.length <= 2) return;
    setLines(lines.filter((_, i) => i !== idx));
  };

  const handleUpdateLine = (idx: number, patch: Partial<LineDraft>) => {
    setLines(
      lines.map((line, i) => (i === idx ? { ...line, ...patch } : line)),
    );
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!description.trim()) {
      setError("Please provide an entry description.");
      return;
    }

    if (!isBalanced) {
      setError(
        `Debits (${formatCurrency(totalDebits)}) must equal Credits (${formatCurrency(totalCredits)}) and be greater than 0.`,
      );
      return;
    }

    for (const line of lines) {
      if (!line.account_id) {
        setError("Every line must have an account assigned.");
        return;
      }
      const amt = parseAmountMinor(line.amount_str);
      if (amt <= 0) {
        setError("All amounts must be greater than zero.");
        return;
      }
    }

    setError(null);
    setSaving(true);
    try {
      await onSave({
        date,
        description: description.trim(),
        lines: lines.map((l) => ({
          account_id: l.account_id,
          direction: l.direction,
          amount_minor: parseAmountMinor(l.amount_str),
          description: l.description.trim() || undefined,
        })),
      });
      onOpenChange(false);
    } catch (err: unknown) {
      const msg =
        err instanceof Error
          ? err.message
          : "Failed to record transaction entry.";
      setError(msg);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl max-h-[90vh] overflow-y-auto bg-card border-border">
        <DialogHeader>
          <DialogTitle className="text-lg text-foreground">
            Post Journal Entry
          </DialogTitle>
          <DialogDescription className="text-xs text-muted-foreground">
            Record a balanced double-entry transaction into the current
            accounting period for{" "}
            <span className="font-semibold text-foreground">
              {period.entity}
            </span>
            .
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4 py-2">
          {error && (
            <div className="flex items-start gap-2 rounded-lg bg-destructive/10 border border-destructive/20 p-3 text-xs text-destructive">
              <AlertCircle className="size-4 shrink-0 mt-0.5" />
              <span>{error}</span>
            </div>
          )}

          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            <div>
              <Label className="text-xs font-medium text-foreground">
                Date
              </Label>
              <div className="mt-1.5">
                <DatePicker
                  date={date}
                  onDateChange={setDate}
                  placeholder="Select entry date"
                />
              </div>
            </div>
            <div className="md:col-span-2">
              <Label
                htmlFor="entry-desc"
                className="text-xs font-medium text-foreground"
              >
                Description / Reference
              </Label>
              <Input
                id="entry-desc"
                placeholder="e.g. Vendor payment, Client invoice payment..."
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                required
                className="mt-1.5 h-9 text-xs bg-background"
              />
            </div>
          </div>

          <div className="space-y-2 pt-2">
            <div className="flex items-center justify-between">
              <Label className="text-xs font-medium text-foreground">
                Accounting Lines (Min 2)
              </Label>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={handleAddLine}
                className="h-7 text-xs gap-1"
              >
                <Plus className="size-3.5" />
                <span>Add Line</span>
              </Button>
            </div>

            <div className="space-y-2 border border-border rounded-lg p-3 bg-muted/30">
              {lines.map((line, idx) => (
                <div
                  key={line.id}
                  className="grid grid-cols-12 gap-2 items-center"
                >
                  <div className="col-span-5">
                    <Select
                      value={line.account_id}
                      onChange={(e) =>
                        handleUpdateLine(idx, { account_id: e.target.value })
                      }
                      className="text-xs h-8 bg-background"
                    >
                      {accounts.map((acc) => (
                        <option key={acc.id} value={acc.id}>
                          {acc.code} - {acc.name} ({acc.account_type})
                        </option>
                      ))}
                    </Select>
                  </div>

                  <div className="col-span-3">
                    <Select
                      value={line.direction}
                      onChange={(e) =>
                        handleUpdateLine(idx, {
                          direction: e.target.value as "debit" | "credit",
                        })
                      }
                      className="text-xs h-8 font-medium bg-background"
                    >
                      <option value="debit">Debit (+)</option>
                      <option value="credit">Credit (-)</option>
                    </Select>
                  </div>

                  <div className="col-span-3">
                    <Input
                      type="number"
                      step="0.01"
                      min="0.01"
                      placeholder="0.00"
                      value={line.amount_str}
                      onChange={(e) =>
                        handleUpdateLine(idx, { amount_str: e.target.value })
                      }
                      className="text-xs h-8 font-mono text-right bg-background"
                      required
                    />
                  </div>

                  <div className="col-span-1 flex justify-center">
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={() => handleRemoveLine(idx)}
                      disabled={lines.length <= 2}
                      className="h-8 w-8 p-0 text-muted-foreground hover:text-destructive"
                    >
                      <Trash2 className="size-3.5" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Balance Status Footer Indicator */}
          <div
            className={`flex items-center justify-between p-3 rounded-lg border text-xs font-mono transition-colors ${
              isBalanced
                ? "bg-emerald-500/10 border-emerald-500/20 text-emerald-700 dark:text-emerald-400"
                : "bg-amber-500/10 border-amber-500/20 text-amber-700 dark:text-amber-400"
            }`}
          >
            <div className="flex items-center gap-1.5 font-sans font-medium">
              {isBalanced ? (
                <>
                  <CheckCircle2 className="size-4 text-emerald-600 dark:text-emerald-400" />
                  <span>Entry is Balanced</span>
                </>
              ) : (
                <>
                  <AlertCircle className="size-4 text-amber-600 dark:text-amber-400" />
                  <span>Unbalanced Entry</span>
                </>
              )}
            </div>
            <div className="flex items-center gap-3 text-xs">
              <span>Debits: {formatCurrency(totalDebits)}</span>
              <span>•</span>
              <span>Credits: {formatCurrency(totalCredits)}</span>
            </div>
          </div>

          <DialogFooter className="pt-2 border-t border-border">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => onOpenChange(false)}
              disabled={saving}
              className="text-xs h-9"
            >
              Cancel
            </Button>
            <Button
              type="submit"
              size="sm"
              disabled={saving || !isBalanced}
              className="text-xs h-9 font-medium"
            >
              {saving ? "Recording..." : "Post Transaction"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
