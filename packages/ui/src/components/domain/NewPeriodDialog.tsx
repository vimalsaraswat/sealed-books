import { useState, useEffect, type FormEvent } from "react";
import { Calendar as CalendarIcon, Plus } from "lucide-react";
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
import { DatePicker } from "../ui/date-picker";
import type { CreatePeriodInput } from "../../types";

export interface NewPeriodDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  defaultEntity?: string;
  onCreate: (period: CreatePeriodInput) => Promise<void>;
}

export function NewPeriodDialog({
  open,
  onOpenChange,
  defaultEntity = "Acme Trading Pvt Ltd",
  onCreate,
}: NewPeriodDialogProps) {
  const [entity, setEntity] = useState(defaultEntity);
  const [startDate, setStartDate] = useState("2026-09-01");
  const [endDate, setEndDate] = useState("2026-09-30");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setEntity(defaultEntity);
      setError(null);
    }
  }, [open, defaultEntity]);

  const setPreset = (start: string, end: string) => {
    setStartDate(start);
    setEndDate(end);
  };

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!entity.trim()) {
      setError("Please provide an entity name.");
      return;
    }
    if (startDate > endDate) {
      setError("Start date cannot be after end date.");
      return;
    }

    setError(null);
    setSaving(true);
    try {
      await onCreate({
        entity: entity.trim(),
        start_date: startDate,
        end_date: endDate,
      });
      onOpenChange(false);
    } catch (err: unknown) {
      const msg =
        err instanceof Error
          ? err.message
          : "Failed to create accounting period.";
      setError(msg);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg bg-card border-border">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-foreground text-lg">
            <CalendarIcon className="size-5 text-primary" />
            <span>Create Accounting Period</span>
          </DialogTitle>
          <DialogDescription className="text-xs text-muted-foreground">
            Open a new ledger cycle. Periods can be any arbitrary timeframe
            (monthly, quarterly, or custom).
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4 py-2">
          {error && (
            <div className="rounded-lg bg-destructive/10 border border-destructive/20 p-3 text-xs text-destructive">
              {error}
            </div>
          )}

          <div>
            <Label
              htmlFor="entity-name"
              className="text-xs font-medium text-foreground"
            >
              Legal Entity Name
            </Label>
            <Input
              id="entity-name"
              value={entity}
              onChange={(e) => setEntity(e.target.value)}
              placeholder="e.g. Acme Trading Pvt Ltd"
              required
              className="mt-1.5 h-9 text-xs bg-background"
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label className="text-xs font-medium text-foreground">
                Start Date
              </Label>
              <div className="mt-1.5">
                <DatePicker
                  date={startDate}
                  onDateChange={setStartDate}
                  placeholder="Select start date"
                />
              </div>
            </div>
            <div>
              <Label className="text-xs font-medium text-foreground">
                End Date
              </Label>
              <div className="mt-1.5">
                <DatePicker
                  date={endDate}
                  onDateChange={setEndDate}
                  placeholder="Select end date"
                />
              </div>
            </div>
          </div>

          <div className="space-y-1.5 pt-1">
            <Label className="text-[10px] text-muted-foreground">
              Quick Presets
            </Label>
            <div className="flex flex-wrap gap-1.5">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setPreset("2026-09-01", "2026-09-30")}
                className="text-xs h-7 px-2"
              >
                Sept 2026
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setPreset("2026-10-01", "2026-10-31")}
                className="text-xs h-7 px-2"
              >
                Oct 2026
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setPreset("2026-07-01", "2026-09-30")}
                className="text-xs h-7 px-2"
              >
                Q3 2026
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setPreset("2026-01-01", "2026-12-31")}
                className="text-xs h-7 px-2"
              >
                FY 2026
              </Button>
            </div>
          </div>

          <DialogFooter className="pt-3 border-t border-border">
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
              disabled={saving}
              className="text-xs h-9 font-medium gap-1.5"
            >
              <Plus className="size-3.5" />
              <span>{saving ? "Opening..." : "Create Period"}</span>
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
