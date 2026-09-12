import { useState, useEffect } from "react";
import { AlertTriangle, Bug, ShieldAlert } from "lucide-react";
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
import type { Entry, Period } from "../../types";

export interface TamperSimulatorDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  period: Period;
  entries: Entry[];
  onTamper: (entryId: string, newDescription: string) => Promise<void>;
  onTriggerVerify: () => void;
}

export function TamperSimulatorDialog({
  open,
  onOpenChange,
  period,
  entries,
  onTamper,
  onTriggerVerify,
}: TamperSimulatorDialogProps) {
  const defaultEntryId = entries[4]?.id || entries[0]?.id || "";
  const [selectedEntryId, setSelectedEntryId] = useState(defaultEntryId);
  const [newDescription, setNewDescription] = useState(
    "Fraudulent invoice kickback modification",
  );
  const [tampering, setTampering] = useState(false);
  const [tamperedSuccess, setTamperedSuccess] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setSelectedEntryId(entries[4]?.id || entries[0]?.id || "");
      setNewDescription("Fraudulent invoice kickback modification");
      setTamperedSuccess(false);
      setError(null);
    }
  }, [open, entries]);

  const selectedEntry = entries.find((e) => e.id === selectedEntryId);

  const handleTamper = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedEntryId) {
      setError("Please select an entry to tamper with.");
      return;
    }

    setError(null);
    setTampering(true);
    try {
      await onTamper(selectedEntryId, newDescription.trim());
      setTamperedSuccess(true);
    } catch (err: unknown) {
      const msg =
        err instanceof Error ? err.message : "Failed to inject tamper record.";
      setError(msg);
    } finally {
      setTampering(false);
    }
  };

  const handleVerifyNow = () => {
    onOpenChange(false);
    onTriggerVerify();
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg bg-card border-border">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-destructive text-lg">
            <Bug className="size-5" />
            <span>Tamper Simulation Lab</span>
          </DialogTitle>
          <DialogDescription className="text-xs text-muted-foreground">
            Simulate an out-of-band direct database modification for{" "}
            <span className="font-semibold text-foreground">
              {period.entity}
            </span>{" "}
            to test Hedera cryptographic verification.
          </DialogDescription>
        </DialogHeader>

        {tamperedSuccess ? (
          <div className="space-y-4 py-3">
            <div className="rounded-lg bg-destructive/10 border border-destructive/20 p-4 space-y-2 text-sm text-destructive">
              <div className="flex items-center gap-2 font-semibold">
                <AlertTriangle className="size-5" />
                <span>Tamper Successfully Injected into Database!</span>
              </div>
              <p className="text-xs leading-relaxed text-muted-foreground">
                Entry{" "}
                <code className="font-mono font-bold text-destructive">
                  {selectedEntryId}
                </code>{" "}
                has been altered directly in storage, bypassing application
                safeguards.
              </p>
              <p className="text-xs text-muted-foreground">
                The on-chain cryptographic seal on Hedera Consensus Service
                remains immutable. Click below to run an independent
                verification audit.
              </p>
            </div>

            <DialogFooter className="pt-2 border-t border-border">
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
                className="text-xs h-9"
              >
                Close
              </Button>
              <Button
                type="button"
                variant="destructive"
                onClick={handleVerifyNow}
                className="gap-2 text-xs h-9 font-medium"
              >
                <ShieldAlert className="size-4" />
                <span>Verify Now (Catch Tampering)</span>
              </Button>
            </DialogFooter>
          </div>
        ) : (
          <form onSubmit={handleTamper} className="space-y-4 py-2">
            {error && (
              <div className="rounded-md bg-destructive/10 border border-destructive/20 p-2.5 text-xs text-destructive">
                {error}
              </div>
            )}

            <div className="rounded-lg bg-muted/40 border border-border p-3 text-xs text-muted-foreground space-y-1">
              <div className="font-medium text-foreground">How this works:</div>
              <div>
                1. We execute an unauthorized{" "}
                <code className="font-mono bg-muted px-1 py-0.5 rounded text-foreground">
                  UPDATE entries
                </code>{" "}
                directly on the database.
              </div>
              <div>
                2. The local ledger state diverges from the seal published to
                Hedera Consensus Service.
              </div>
              <div>
                3. The verification engine recalculates each leaf hash and
                pinpoints the altered transaction.
              </div>
            </div>

            <div>
              <Label
                htmlFor="tamper-entry"
                className="text-xs font-medium text-foreground"
              >
                Target Entry to Alter
              </Label>
              <Select
                id="tamper-entry"
                value={selectedEntryId}
                onChange={(e) => setSelectedEntryId(e.target.value)}
                className="mt-1.5 text-xs font-mono bg-background"
              >
                {entries.map((entry) => (
                  <option key={entry.id} value={entry.id}>
                    {entry.id} — {entry.date} — {entry.description}
                  </option>
                ))}
              </Select>
            </div>

            {selectedEntry && (
              <div className="p-2.5 rounded-md bg-muted/40 border border-border text-xs">
                <div className="text-muted-foreground font-medium">
                  Original Description:
                </div>
                <div className="text-foreground font-semibold mt-0.5">
                  &ldquo;{selectedEntry.description}&rdquo;
                </div>
              </div>
            )}

            <div>
              <Label
                htmlFor="tamper-desc"
                className="text-xs font-medium text-foreground"
              >
                Fraudulent Altered Description
              </Label>
              <Input
                id="tamper-desc"
                value={newDescription}
                onChange={(e) => setNewDescription(e.target.value)}
                required
                className="mt-1.5 text-xs h-9 bg-background"
              />
            </div>

            <DialogFooter className="pt-2 border-t border-border">
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
                disabled={tampering}
                className="text-xs h-9"
              >
                Cancel
              </Button>
              <Button
                type="submit"
                variant="destructive"
                disabled={
                  tampering || !selectedEntryId || !newDescription.trim()
                }
                className="gap-1.5 text-xs h-9 font-medium"
              >
                <Bug className="size-4" />
                <span>{tampering ? "Tampering..." : "Inject Tamper"}</span>
              </Button>
            </DialogFooter>
          </form>
        )}
      </DialogContent>
    </Dialog>
  );
}
