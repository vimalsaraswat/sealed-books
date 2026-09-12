import { useState } from "react";
import {
  Menu,
  Calendar,
  ChevronDown,
  Check,
  Plus,
  RotateCw,
  ExternalLink,
} from "lucide-react";
import { Badge } from "../ui/badge";
import { Button } from "../ui/button";
import { ModeToggle } from "../ui/mode-toggle";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "../ui/dropdown-menu";
import { useAuth } from "../../context/AuthContext";
import { useLedger } from "../../context/LedgerContext";
import { useModals } from "../../context/ModalContext";

export interface HeaderProps {
  onMobileMenuToggle?: () => void;
}

export function Header({ onMobileMenuToggle }: HeaderProps) {
  const { userRole } = useAuth();
  const {
    periods,
    currentPeriod,
    selectedPeriodId,
    seal,
    handlePeriodChange,
    loadPeriodData,
  } = useLedger();

  const { setIsNewPeriodOpen } = useModals();

  const [isRefreshing, setIsRefreshing] = useState(false);

  const handleRefresh = async () => {
    if (!selectedPeriodId) return;
    setIsRefreshing(true);
    try {
      await loadPeriodData(selectedPeriodId);
    } finally {
      setTimeout(() => setIsRefreshing(false), 500);
    }
  };

  const isCurrentSealed = currentPeriod?.status === "sealed";
  const topicId = seal?.topic_id || "0.0.10462941";

  return (
    <header className="sticky top-0 z-30 w-full h-14 border-b border-border bg-background/95 backdrop-blur-md px-4 sm:px-6 flex items-center justify-between gap-4">
      {/* =========================================================
          LEFT ZONE: Hamburger & Period Selector Context
          ========================================================= */}
      <div className="flex items-center gap-3 min-w-0">
        {/* Mobile Hamburger Menu Toggle */}
        <button
          onClick={onMobileMenuToggle}
          className="md:hidden p-1.5 rounded-md text-muted-foreground hover:text-foreground hover:bg-muted"
        >
          <Menu className="size-5" />
        </button>

        {/* Accounting Period Selector */}
        {currentPeriod && (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="outline"
                size="sm"
                className="h-8 gap-2 px-3 text-xs font-mono bg-background hover:bg-muted/60"
              >
                <Calendar className="size-3.5 text-muted-foreground shrink-0" />
                <span className="font-semibold text-foreground">
                  {currentPeriod.start_date} → {currentPeriod.end_date}
                </span>

                {isCurrentSealed ? (
                  <span className="size-2 rounded-full bg-emerald-500 shrink-0 ring-2 ring-emerald-500/20" />
                ) : (
                  <span className="size-2 rounded-full bg-amber-500 shrink-0 ring-2 ring-amber-500/20" />
                )}

                <ChevronDown className="size-3 text-muted-foreground shrink-0" />
              </Button>
            </DropdownMenuTrigger>

            <DropdownMenuContent align="start" className="w-72 font-mono">
              <DropdownMenuLabel className="text-[10px] uppercase tracking-wider font-sans">
                Accounting Periods
              </DropdownMenuLabel>
              {periods.map((p) => {
                const isSelected = p.id === currentPeriod.id;
                const isSealed = p.status === "sealed";
                return (
                  <DropdownMenuItem
                    key={p.id}
                    onClick={() => handlePeriodChange(p.id)}
                    className="justify-between text-xs cursor-pointer py-2"
                  >
                    <div className="flex flex-col">
                      <span className="font-semibold text-foreground">
                        {p.start_date} → {p.end_date}
                      </span>
                      <span className="text-[10px] text-muted-foreground">ID: {p.id}</span>
                    </div>
                    <div className="flex items-center gap-1.5">
                      <Badge
                        variant={isSealed ? "sealed" : "default"}
                        className="text-[9px] px-1.5 py-0 uppercase"
                      >
                        {p.status}
                      </Badge>
                      {isSelected && <Check className="size-3 text-primary" />}
                    </div>
                  </DropdownMenuItem>
                );
              })}
              {userRole !== "auditor" && (
                <>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    onClick={() => setIsNewPeriodOpen(true)}
                    className="gap-2 text-xs font-sans cursor-pointer text-primary focus:text-primary py-2"
                  >
                    <Plus className="size-3.5" />
                    <span>Open New Accounting Period</span>
                  </DropdownMenuItem>
                </>
              )}
            </DropdownMenuContent>
          </DropdownMenu>
        )}

        {/* Status Badge */}
        {currentPeriod && (
          <Badge
            variant={isCurrentSealed ? "sealed" : "default"}
            className="hidden sm:inline-flex text-[10px] uppercase font-mono py-0.5"
          >
            {currentPeriod.status}
          </Badge>
        )}
      </div>

      {/* =========================================================
          RIGHT ZONE: Live Mirror Node Status & Controls
          ========================================================= */}
      <div className="flex items-center gap-2 shrink-0">
        {/* Hedera Consensus Service Topic Pill */}
        <a
          href={`https://hashscan.io/testnet/topic/${topicId}`}
          target="_blank"
          rel="noreferrer"
          className="hidden md:inline-flex items-center gap-1.5 text-[11px] font-mono px-2.5 py-1 rounded-md bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground border border-border/60 transition-colors"
          title="View HCS Consensus Topic on HashScan"
        >
          <span className="size-1.5 rounded-full bg-emerald-500 animate-pulse" />
          <span>HCS #{topicId}</span>
          <ExternalLink className="size-3 opacity-60" />
        </a>

        {/* Live Refresh */}
        <Button
          variant="ghost"
          size="sm"
          onClick={handleRefresh}
          disabled={isRefreshing}
          title="Refresh ledger state"
          className="h-8 w-8 p-0 text-muted-foreground hover:text-foreground"
        >
          <RotateCw
            className={`size-3.5 ${isRefreshing ? "animate-spin text-primary" : ""}`}
          />
        </Button>

        {/* Dark / Light Mode Toggle */}
        <ModeToggle />
      </div>
    </header>
  );
}
