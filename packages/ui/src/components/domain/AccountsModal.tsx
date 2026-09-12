import { useState } from "react";
import { BookOpen, Plus } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "../ui/dialog";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { Select } from "../ui/select";
import { Badge } from "../ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../ui/table";
import type { Account, AccountType, CreateAccountInput } from "../../types";

export interface AccountsModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  accounts: Account[];
  onCreateAccount: (account: CreateAccountInput) => Promise<void>;
}

export function AccountsModal({
  open,
  onOpenChange,
  accounts,
  onCreateAccount,
}: AccountsModalProps) {
  const [showAddForm, setShowAddForm] = useState(false);
  const [code, setCode] = useState("");
  const [name, setName] = useState("");
  const [accountType, setAccountType] = useState<AccountType>("expense");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const getTypeBadgeVariant = (type: AccountType) => {
    switch (type) {
      case "asset":
        return "default";
      case "liability":
        return "destructive";
      case "equity":
        return "secondary";
      case "revenue":
        return "success";
      case "expense":
        return "warning";
    }
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!code.trim() || !name.trim()) {
      setError("Please fill in both code and account name.");
      return;
    }

    if (accounts.some((a) => a.code === code.trim())) {
      setError(`Account with code ${code.trim()} already exists.`);
      return;
    }

    setError(null);
    setSaving(true);
    try {
      await onCreateAccount({
        code: code.trim(),
        name: name.trim(),
        account_type: accountType,
      });
      setCode("");
      setName("");
      setShowAddForm(false);
    } catch (err: unknown) {
      const msg =
        err instanceof Error ? err.message : "Failed to create account.";
      setError(msg);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl max-h-[85vh] overflow-y-auto bg-card border-border">
        <DialogHeader>
          <div className="flex items-center justify-between pr-6">
            <DialogTitle className="flex items-center gap-2 text-foreground text-lg">
              <BookOpen className="size-5 text-primary" />
              <span>Chart of Accounts</span>
            </DialogTitle>
            <Button
              size="sm"
              variant={showAddForm ? "secondary" : "default"}
              onClick={() => setShowAddForm(!showAddForm)}
              className="h-8 gap-1 text-xs"
            >
              <Plus className="size-3.5" />
              <span>{showAddForm ? "Close Form" : "New Account"}</span>
            </Button>
          </div>
          <DialogDescription className="text-xs text-muted-foreground">
            General ledger account definitions categorized by accounting
            classifications.
          </DialogDescription>
        </DialogHeader>

        {showAddForm && (
          <form
            onSubmit={handleCreate}
            className="p-4 rounded-lg border border-primary/20 bg-primary/5 space-y-3"
          >
            <div className="font-semibold text-xs text-foreground uppercase tracking-wider">
              Create New Account
            </div>

            {error && (
              <div className="rounded-md bg-destructive/10 border border-destructive/20 p-2.5 text-xs text-destructive">
                {error}
              </div>
            )}

            <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
              <div>
                <Label
                  htmlFor="acc-code"
                  className="text-xs font-medium text-foreground"
                >
                  Code
                </Label>
                <Input
                  id="acc-code"
                  placeholder="e.g. 6010"
                  value={code}
                  onChange={(e) => setCode(e.target.value)}
                  required
                  className="mt-1 font-mono text-xs bg-background h-8"
                />
              </div>
              <div>
                <Label
                  htmlFor="acc-name"
                  className="text-xs font-medium text-foreground"
                >
                  Account Name
                </Label>
                <Input
                  id="acc-name"
                  placeholder="e.g. Cloud Subscriptions"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  required
                  className="mt-1 bg-background h-8 text-xs"
                />
              </div>
              <div>
                <Label
                  htmlFor="acc-type"
                  className="text-xs font-medium text-foreground"
                >
                  Account Type
                </Label>
                <Select
                  id="acc-type"
                  value={accountType}
                  onChange={(e) =>
                    setAccountType(e.target.value as AccountType)
                  }
                  className="mt-1 text-xs bg-background capitalize h-8"
                >
                  <option value="asset">Asset (1000s)</option>
                  <option value="liability">Liability (2000s)</option>
                  <option value="equity">Equity (3000s)</option>
                  <option value="revenue">Revenue (4000s)</option>
                  <option value="expense">Expense (5000s)</option>
                </Select>
              </div>
            </div>

            <div className="flex justify-end gap-2 pt-1">
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={() => setShowAddForm(false)}
                className="text-xs h-8"
              >
                Cancel
              </Button>
              <Button
                type="submit"
                size="sm"
                disabled={saving}
                className="text-xs h-8"
              >
                {saving ? "Saving..." : "Save Account"}
              </Button>
            </div>
          </form>
        )}

        <div className="rounded-lg border border-border overflow-hidden bg-card">
          <Table>
            <TableHeader className="bg-muted/50">
              <TableRow>
                <TableHead className="w-[100px] text-xs">Code</TableHead>
                <TableHead className="text-xs">Account Name</TableHead>
                <TableHead className="w-[120px] text-right text-xs">
                  Classification
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {accounts.map((acc) => (
                <TableRow key={acc.id} className="hover:bg-muted/40">
                  <TableCell className="font-mono font-semibold text-xs text-foreground py-2.5">
                    {acc.code}
                  </TableCell>
                  <TableCell className="text-xs text-foreground font-medium py-2.5">
                    {acc.name}
                  </TableCell>
                  <TableCell className="text-right py-2.5">
                    <Badge
                      variant={getTypeBadgeVariant(acc.account_type)}
                      className="capitalize text-[10px]"
                    >
                      {acc.account_type}
                    </Badge>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </DialogContent>
    </Dialog>
  );
}
