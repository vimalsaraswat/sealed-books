import * as React from "react";
import { format, parseISO, isValid } from "date-fns";
import { Calendar as CalendarIcon } from "lucide-react";
import { cn } from "../../lib/utils";
import { Button } from "./button";
import { Calendar } from "./calendar";
import { Popover, PopoverContent, PopoverTrigger } from "./popover";

export interface DatePickerProps {
  date?: string; // YYYY-MM-DD
  onDateChange?: (dateStr: string) => void;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
}

export function DatePicker({
  date,
  onDateChange,
  placeholder = "Pick a date",
  disabled = false,
  className,
}: DatePickerProps) {
  const [open, setOpen] = React.useState(false);

  const parsedDate = React.useMemo(() => {
    if (!date) return undefined;
    const d = parseISO(date);
    return isValid(d) ? d : undefined;
  }, [date]);

  const handleSelect = (selectedDate: Date | undefined) => {
    if (selectedDate && onDateChange) {
      onDateChange(format(selectedDate, "yyyy-MM-dd"));
      setOpen(false);
    }
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          disabled={disabled}
          className={cn(
            "w-full justify-start text-left font-mono text-xs h-9 px-3 bg-background border-input hover:bg-muted/50",
            !date && "text-muted-foreground",
            className,
          )}
        >
          <CalendarIcon className="mr-2 size-3.5 text-muted-foreground" />
          {parsedDate ? format(parsedDate, "yyyy-MM-dd") : <span>{placeholder}</span>}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-auto p-0 bg-popover border-border" align="start">
        <Calendar
          mode="single"
          selected={parsedDate}
          onSelect={handleSelect}
          initialFocus
        />
      </PopoverContent>
    </Popover>
  );
}
