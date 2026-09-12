import { useState, type FormEvent } from "react";
import { Building2, Plus, Loader2, AlertCircle } from "lucide-react";
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
import type { ApiClient } from "../../types";

export interface CreateOrgModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  api: ApiClient;
  onOrgCreated?: (orgId: string) => Promise<void> | void;
}

export function CreateOrgModal({
  open,
  onOpenChange,
  api,
  onOrgCreated,
}: CreateOrgModalProps) {
  const [name, setName] = useState("");
  const [currency, setCurrency] = useState("USD");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    const cleanName = name.trim();
    if (!cleanName) return;

    setLoading(true);
    setError(null);
    try {
      const org = await api.createOrganization(cleanName, currency);
      setName("");
      onOpenChange(false);
      if (onOrgCreated) {
        await onOrgCreated(org.id);
      }
    } catch (err: any) {
      setError(err.message || "Failed to create organization");
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md bg-card border-border">
        <DialogHeader>
          <div className="flex items-center gap-3">
            <div className="size-9 rounded-lg bg-primary/10 text-primary flex items-center justify-center border border-primary/20">
              <Building2 className="size-5" />
            </div>
            <div>
              <DialogTitle className="text-lg">Create New Organization</DialogTitle>
              <DialogDescription className="text-xs text-muted-foreground">
                Set up a new reporting entity or client vault with its own immutable ledger.
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4 pt-2">
          {error && (
            <div className="p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-xs text-destructive flex items-start gap-2">
              <AlertCircle className="size-4 shrink-0 mt-0.5" />
              <span>{error}</span>
            </div>
          )}

          <div className="space-y-1.5">
            <Label className="text-xs font-medium text-foreground">
              Organization Name
            </Label>
            <Input
              type="text"
              placeholder="e.g. Acme Capital Holdings LLC"
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
              autoFocus
              className="h-9 text-xs bg-background"
            />
          </div>

          <div className="space-y-1.5">
            <Label className="text-xs font-medium text-foreground">
              Base Reporting Currency
            </Label>
            <Select
              value={currency}
              onChange={(e) => setCurrency(e.target.value)}
              className="h-9 text-xs bg-background"
            >
              <option value="USD">USD - United States Dollar ($)</option>
              <option value="EUR">EUR - Euro (€)</option>
              <option value="GBP">GBP - British Pound (£)</option>
              <option value="INR">INR - Indian Rupee (₹)</option>
              <option value="JPY">JPY - Japanese Yen (¥)</option>
              <option value="SGD">SGD - Singapore Dollar (S$)</option>
            </Select>
          </div>

          <div className="flex items-center justify-end gap-2 pt-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => onOpenChange(false)}
              className="h-9 text-xs"
            >
              Cancel
            </Button>
            <Button
              type="submit"
              size="sm"
              disabled={loading || !name.trim()}
              className="h-9 gap-1.5 text-xs font-medium"
            >
              {loading ? (
                <Loader2 className="size-3.5 animate-spin" />
              ) : (
                <Plus className="size-3.5" />
              )}
              <span>Create Organization</span>
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
