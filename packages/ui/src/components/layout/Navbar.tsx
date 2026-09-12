import { useState } from "react";
import {
  Building2,
  Calendar,
  RotateCw,
  ChevronDown,
  Check,
  Plus,
  Users,
  LogOut,
  Copy,
  LayoutDashboard,
  BookOpen,
  Scale,
  ShieldCheck,
} from "lucide-react";
import { Badge } from "../ui/badge";
import { Button } from "../ui/button";
import { BrandLogo } from "../ui/BrandLogo";
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
import { truncateAddress } from "../../lib/utils";
import type { NavTab } from "../../types";

export function Navbar() {
  const {
    currentUser,
    activeOrg,
    userRole,
    availableOrgs,
    pendingAudits,
    switchOrg,
    logout,
  } = useAuth();

  const {
    periods,
    currentPeriod,
    selectedPeriodId,
    handlePeriodChange,
    loadPeriodData,
    activeTab,
    setActiveTab,
  } = useLedger();

  const { setIsCreateOrgOpen, setIsNewPeriodOpen } = useModals();

  const [isRefreshing, setIsRefreshing] = useState(false);
  const [copiedAddr, setCopiedAddr] = useState(false);

  const handleRefresh = async () => {
    if (!selectedPeriodId) return;
    setIsRefreshing(true);
    try {
      await loadPeriodData(selectedPeriodId);
    } finally {
      setTimeout(() => setIsRefreshing(false), 500);
    }
  };

  const handleCopyAddress = () => {
    if (!currentUser?.eth_address) return;
    navigator.clipboard.writeText(currentUser.eth_address);
    setCopiedAddr(true);
    setTimeout(() => setCopiedAddr(false), 1500);
  };

  const isCurrentSealed = currentPeriod?.status === "sealed";
  const userInitials =
    currentUser?.name
      ?.split(" ")
      .map((n) => n[0])
      .slice(0, 2)
      .join("")
      .toUpperCase() || "U";

  const navTabs: {
    id: NavTab;
    label: string;
    icon: React.ElementType;
    badge?: number;
  }[] = [
    { id: "overview", label: "Overview", icon: LayoutDashboard },
    { id: "ledger", label: "General Ledger", icon: BookOpen },
    { id: "financials", label: "Financials", icon: Scale },
    {
      id: "audit",
      label: "Audit & Consensus",
      icon: ShieldCheck,
      badge:
        userRole === "auditor" && pendingAudits.length > 0
          ? pendingAudits.length
          : undefined,
    },
    { id: "team", label: "Team", icon: Users },
  ];

  return (
    <header className="sticky top-0 z-40 w-full border-b border-border/80 bg-background/95 backdrop-blur-md">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16 gap-4">
          {/* =========================================================
              LEFT ZONE: Brand & Workspace Context Selectors
              ========================================================= */}
          <div className="flex items-center gap-3 min-w-0">
            {/* Brand Logo & Name */}
            <div className="flex items-center gap-2.5 shrink-0">
              <BrandLogo size="sm" />
              <div className="hidden sm:block">
                <div className="flex items-center gap-1.5">
                  <span className="font-bold text-sm tracking-tight text-foreground">
                    Sealed Books
                  </span>
                  <span className="text-[10px] font-mono px-1.5 py-0.5 rounded-sm bg-muted text-muted-foreground border border-border/60">
                    HCS
                  </span>
                </div>
              </div>
            </div>

            <div className="h-5 w-px bg-border/60 hidden sm:block" />

            {/* Organization Selector */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg border border-border/70 hover:bg-accent/50 text-xs font-semibold text-foreground transition-colors cursor-pointer group">
                  <div className="size-4 rounded-xs bg-primary/10 text-primary flex items-center justify-center">
                    <Building2 className="size-3" />
                  </div>
                  <span className="max-w-[130px] truncate">
                    {activeOrg?.name || "Select Org"}
                  </span>
                  <ChevronDown className="size-3 text-muted-foreground group-hover:text-foreground transition-transform" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start" className="w-56">
                <DropdownMenuLabel className="text-xs text-muted-foreground">
                  Switch Organization
                </DropdownMenuLabel>
                <DropdownMenuSeparator />
                {availableOrgs.map((org) => {
                  const isCurrent = org.id === activeOrg?.id;
                  return (
                    <DropdownMenuItem
                      key={org.id}
                      onClick={() => switchOrg(org.id)}
                      className="flex items-center justify-between cursor-pointer py-1.5"
                    >
                      <div className="flex items-center gap-2 min-w-0">
                        <Building2 className="size-3.5 text-muted-foreground shrink-0" />
                        <span className="text-xs truncate">{org.name}</span>
                      </div>
                      {isCurrent && (
                        <Check className="size-3.5 text-primary shrink-0" />
                      )}
                    </DropdownMenuItem>
                  );
                })}
                <DropdownMenuSeparator />
                <DropdownMenuItem
                  onClick={() => setIsCreateOrgOpen(true)}
                  className="flex items-center gap-2 text-primary cursor-pointer text-xs"
                >
                  <Plus className="size-3.5" />
                  <span>Create Organization</span>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>

            {/* Accounting Period Selector */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg border border-border/70 hover:bg-accent/50 text-xs font-medium text-foreground transition-colors cursor-pointer group">
                  <Calendar className="size-3.5 text-muted-foreground" />
                  <span className="max-w-[140px] truncate font-mono text-[11px]">
                    {currentPeriod
                      ? `${currentPeriod.entity} (${currentPeriod.start_date.slice(0, 7)})`
                      : "Select Period"}
                  </span>
                  {isCurrentSealed ? (
                    <Badge
                      variant="secondary"
                      className="text-[9px] px-1 py-0 h-4 bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/20"
                    >
                      SEALED
                    </Badge>
                  ) : (
                    <Badge
                      variant="outline"
                      className="text-[9px] px-1 py-0 h-4 bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20"
                    >
                      OPEN
                    </Badge>
                  )}
                  <ChevronDown className="size-3 text-muted-foreground group-hover:text-foreground transition-transform" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start" className="w-64">
                <DropdownMenuLabel className="text-xs text-muted-foreground">
                  Select Financial Period
                </DropdownMenuLabel>
                <DropdownMenuSeparator />
                {periods.map((p) => {
                  const isCurrent = p.id === currentPeriod?.id;
                  const isSealed = p.status === "sealed";
                  return (
                    <DropdownMenuItem
                      key={p.id}
                      onClick={() => handlePeriodChange(p.id)}
                      className="flex items-center justify-between cursor-pointer py-2"
                    >
                      <div className="flex flex-col min-w-0 pr-2">
                        <span className="text-xs font-medium truncate">
                          {p.entity}
                        </span>
                        <span className="text-[10px] text-muted-foreground font-mono">
                          {p.start_date} → {p.end_date}
                        </span>
                      </div>
                      <div className="flex items-center gap-1.5 shrink-0">
                        <Badge
                          variant={isSealed ? "secondary" : "outline"}
                          className={`text-[9px] px-1 py-0 h-4 ${
                            isSealed
                              ? "bg-emerald-500/10 text-emerald-600 border-emerald-500/20"
                              : "bg-amber-500/10 text-amber-600 border-amber-500/20"
                          }`}
                        >
                          {p.status.toUpperCase()}
                        </Badge>
                        {isCurrent && (
                          <Check className="size-3.5 text-primary" />
                        )}
                      </div>
                    </DropdownMenuItem>
                  );
                })}
                {userRole !== "auditor" && (
                  <>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem
                      onClick={() => setIsNewPeriodOpen(true)}
                      className="flex items-center gap-2 text-primary cursor-pointer text-xs"
                    >
                      <Plus className="size-3.5" />
                      <span>Open New Financial Period</span>
                    </DropdownMenuItem>
                  </>
                )}
              </DropdownMenuContent>
            </DropdownMenu>

            {/* Quick Refresh Button */}
            <Button
              variant="ghost"
              size="icon"
              onClick={handleRefresh}
              disabled={isRefreshing}
              title="Refresh ledger state"
              className="size-7 text-muted-foreground hover:text-foreground cursor-pointer"
            >
              <RotateCw
                className={`size-3.5 ${isRefreshing ? "animate-spin text-primary" : ""}`}
              />
            </Button>
          </div>

          {/* =========================================================
              CENTER ZONE: Navigation Tabs (Desktop)
              ========================================================= */}
          <nav className="hidden lg:flex items-center gap-1 bg-muted/50 p-1 rounded-lg border border-border/50">
            {navTabs.map((tab) => {
              const Icon = tab.icon;
              const isActive = activeTab === tab.id;
              return (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={`flex items-center gap-2 px-3 py-1.5 rounded-md text-xs font-medium transition-colors cursor-pointer ${
                    isActive
                      ? "bg-background text-foreground shadow-xs"
                      : "text-muted-foreground hover:text-foreground hover:bg-background/40"
                  }`}
                >
                  <Icon className="size-3.5" />
                  <span>{tab.label}</span>
                  {tab.badge !== undefined && (
                    <Badge
                      variant={isActive ? "default" : "secondary"}
                      className="text-[10px] px-1 py-0 h-4 rounded-full"
                    >
                      {tab.badge}
                    </Badge>
                  )}
                </button>
              );
            })}
          </nav>

          {/* =========================================================
              RIGHT ZONE: User Identity, Signer & Controls
              ========================================================= */}
          <div className="flex items-center gap-2">
            <ModeToggle />

            {/* User Profile & Key Signer Badge */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button className="flex items-center gap-2 pl-2 pr-1.5 py-1 rounded-lg border border-border/70 hover:bg-accent/50 transition-colors cursor-pointer group">
                  <div className="flex flex-col items-end text-right hidden sm:block">
                    <span className="text-xs font-semibold text-foreground leading-tight">
                      {currentUser?.name || "Accounting Officer"}
                    </span>
                    <span className="text-[10px] font-mono text-muted-foreground uppercase">
                      {userRole || "viewer"}
                    </span>
                  </div>
                  <div className="size-7 rounded-full bg-primary/10 text-primary border border-primary/20 flex items-center justify-center font-bold text-xs shrink-0 group-hover:ring-2 group-hover:ring-primary/20 transition-all">
                    {userInitials}
                  </div>
                  <ChevronDown className="size-3 text-muted-foreground group-hover:text-foreground transition-transform" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-60">
                <DropdownMenuLabel className="space-y-1">
                  <div className="text-xs font-semibold text-foreground">
                    {currentUser?.name || "Accounting Officer"}
                  </div>
                  <div className="text-[11px] text-muted-foreground font-normal truncate">
                    {currentUser?.email}
                  </div>
                </DropdownMenuLabel>
                <DropdownMenuSeparator />

                {/* Signing Key Address */}
                {currentUser?.eth_address && (
                  <div className="px-2 py-1.5 text-xs">
                    <div className="text-[10px] text-muted-foreground uppercase font-mono mb-1">
                      Web3 Signer Address
                    </div>
                    <button
                      onClick={handleCopyAddress}
                      className="w-full flex items-center justify-between gap-1 p-1.5 rounded-md bg-muted/60 hover:bg-muted font-mono text-[11px] text-foreground transition-colors cursor-pointer"
                    >
                      <span>{truncateAddress(currentUser.eth_address)}</span>
                      <div className="flex items-center gap-1 text-[10px] text-muted-foreground">
                        {copiedAddr ? (
                          <span className="text-emerald-500 font-sans">
                            Copied!
                          </span>
                        ) : (
                          <Copy className="size-3" />
                        )}
                      </div>
                    </button>
                  </div>
                )}

                <DropdownMenuSeparator />
                <DropdownMenuItem
                  onClick={logout}
                  className="flex items-center gap-2 text-destructive cursor-pointer text-xs"
                >
                  <LogOut className="size-3.5" />
                  <span>Sign Out</span>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </div>
    </header>
  );
}
