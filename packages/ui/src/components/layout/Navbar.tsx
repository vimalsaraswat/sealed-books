import { useState } from "react";
import {
  Lock,
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
              <div className="size-8 rounded-lg bg-primary text-primary-foreground flex items-center justify-center shadow-xs">
                <Lock className="size-4.5" />
              </div>
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
            {activeOrg && (
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 gap-1.5 px-2 text-xs font-semibold hover:bg-muted/80"
                  >
                    <Building2 className="size-3.5 text-muted-foreground shrink-0" />
                    <span className="truncate max-w-[120px] sm:max-w-[160px]">
                      {activeOrg.name}
                    </span>
                    <ChevronDown className="size-3 text-muted-foreground shrink-0" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" className="w-56">
                  <DropdownMenuLabel className="text-[10px] uppercase font-mono tracking-wider">
                    Workspaces
                  </DropdownMenuLabel>
                  {availableOrgs.map((org) => {
                    const isSelected = org.id === activeOrg.id;
                    return (
                      <DropdownMenuItem
                        key={org.id}
                        onClick={() => switchOrg(org.id)}
                        className="justify-between text-xs cursor-pointer"
                      >
                        <span className="truncate font-medium">{org.name}</span>
                        <div className="flex items-center gap-1.5">
                          <span className="text-[10px] font-mono text-muted-foreground uppercase">
                            {org.role}
                          </span>
                          {isSelected && (
                            <Check className="size-3 text-primary" />
                          )}
                        </div>
                      </DropdownMenuItem>
                    );
                  })}
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    onClick={() => setIsCreateOrgOpen(true)}
                    className="gap-2 text-xs cursor-pointer text-primary focus:text-primary"
                  >
                    <Plus className="size-3.5" />
                    <span>Create Organization</span>
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            )}

            {/* Period Selector */}
            {currentPeriod && (
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-8 gap-1.5 px-2.5 text-xs font-mono bg-background/60 hover:bg-muted/60"
                  >
                    <Calendar className="size-3.5 text-muted-foreground shrink-0" />
                    <span className="hidden md:inline">
                      {currentPeriod.start_date.slice(5)} →{" "}
                      {currentPeriod.end_date.slice(5)}
                    </span>
                    <span className="md:hidden truncate max-w-[80px]">
                      {currentPeriod.id.replace("per_", "")}
                    </span>
                    {isCurrentSealed ? (
                      <span className="size-2 rounded-full bg-emerald-500 shrink-0 ring-2 ring-emerald-500/20" />
                    ) : (
                      <span className="size-2 rounded-full bg-amber-500 shrink-0 ring-2 ring-amber-500/20" />
                    )}
                    <ChevronDown className="size-3 text-muted-foreground shrink-0" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" className="w-64 font-mono">
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
                        className="justify-between text-xs cursor-pointer"
                      >
                        <div className="flex flex-col">
                          <span className="font-semibold">
                            {p.start_date} → {p.end_date}
                          </span>
                          <span className="text-[10px] text-muted-foreground">
                            ID: {p.id}
                          </span>
                        </div>
                        <div className="flex items-center gap-1.5">
                          <Badge
                            variant={isSealed ? "sealed" : "default"}
                            className="text-[9px] px-1.5 py-0 uppercase"
                          >
                            {p.status}
                          </Badge>
                          {isSelected && (
                            <Check className="size-3 text-primary" />
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
                        className="gap-2 text-xs font-sans cursor-pointer text-primary focus:text-primary"
                      >
                        <Plus className="size-3.5" />
                        <span>Open New Accounting Period</span>
                      </DropdownMenuItem>
                    </>
                  )}
                </DropdownMenuContent>
              </DropdownMenu>
            )}
          </div>

          {/* =========================================================
              CENTER ZONE: Primary Navigation Tabs (Desktop)
              ========================================================= */}
          <nav className="hidden lg:flex items-center gap-1 bg-muted/50 p-1 rounded-lg border border-border/50">
            {navTabs.map((tab) => {
              const Icon = tab.icon;
              const isActive = activeTab === tab.id;
              return (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={`relative flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md transition-all duration-150 ${
                    isActive
                      ? "bg-background text-foreground shadow-xs"
                      : "text-muted-foreground hover:text-foreground hover:bg-background/40"
                  }`}
                >
                  <Icon className="size-3.5 shrink-0" />
                  <span>{tab.label}</span>
                  {tab.badge !== undefined && (
                    <span className="ml-1 px-1.5 py-0.2 rounded-full bg-amber-500/20 text-amber-700 dark:text-amber-300 font-mono font-bold text-[10px] animate-pulse">
                      {tab.badge}
                    </span>
                  )}
                </button>
              );
            })}
          </nav>

          {/* =========================================================
              RIGHT ZONE: Utilities & User Profile
              ========================================================= */}
          <div className="flex items-center gap-2 shrink-0">
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

            {/* Theme Toggle */}
            <ModeToggle />

            {/* User Profile Dropdown */}
            {currentUser && (
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 gap-2 px-2 hover:bg-muted/80"
                  >
                    <div className="size-6 rounded-full bg-primary/15 text-primary border border-primary/20 flex items-center justify-center font-bold text-[10px]">
                      {userInitials}
                    </div>
                    <div className="hidden sm:flex flex-col items-start text-left">
                      <span className="text-xs font-semibold text-foreground leading-tight truncate max-w-[110px]">
                        {currentUser.name}
                      </span>
                      <span className="text-[10px] font-mono uppercase text-muted-foreground leading-none">
                        {userRole}
                      </span>
                    </div>
                    <ChevronDown className="size-3 text-muted-foreground shrink-0" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="w-64">
                  <div className="px-2 py-2">
                    <p className="text-xs font-semibold text-foreground">
                      {currentUser.name}
                    </p>
                    <p className="text-[11px] text-muted-foreground truncate">
                      {currentUser.email}
                    </p>
                    <div className="mt-2 flex items-center justify-between p-1.5 rounded-md bg-muted/60 border border-border/50">
                      <span className="text-[10px] font-mono text-muted-foreground">
                        Role:
                      </span>
                      <Badge
                        variant="outline"
                        className="text-[9px] font-mono uppercase py-0"
                      >
                        {userRole}
                      </Badge>
                    </div>
                    {currentUser.eth_address && (
                      <div className="mt-1.5 flex items-center justify-between p-1.5 rounded-md bg-muted/40 text-[10px] font-mono text-muted-foreground">
                        <span>{truncateAddress(currentUser.eth_address)}</span>
                        <button
                          onClick={handleCopyAddress}
                          className="hover:text-foreground inline-flex items-center gap-1"
                          title="Copy public address"
                        >
                          {copiedAddr ? (
                            <Check className="size-3 text-emerald-500" />
                          ) : (
                            <Copy className="size-3" />
                          )}
                        </button>
                      </div>
                    )}
                  </div>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    onClick={() => setActiveTab("team")}
                    className="gap-2 text-xs cursor-pointer"
                  >
                    <Users className="size-3.5 text-muted-foreground" />
                    <span>Team & Governance</span>
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    onClick={() => logout()}
                    className="gap-2 text-xs cursor-pointer text-destructive focus:text-destructive focus:bg-destructive/10"
                  >
                    <LogOut className="size-3.5" />
                    <span>Sign Out</span>
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            )}
          </div>
        </div>

        {/* =========================================================
            MOBILE/TABLET NAVIGATION BAR (Under 1024px)
            ========================================================= */}
        <div className="flex lg:hidden items-center justify-between overflow-x-auto py-2 border-t border-border/60 gap-1 scrollbar-none">
          {navTabs.map((tab) => {
            const Icon = tab.icon;
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`flex items-center gap-1.5 px-3 py-1 text-xs font-medium rounded-md whitespace-nowrap transition-colors ${
                  isActive
                    ? "bg-primary/10 text-primary font-semibold"
                    : "text-muted-foreground hover:text-foreground"
                }`}
              >
                <Icon className="size-3.5 shrink-0" />
                <span>{tab.label}</span>
                {tab.badge !== undefined && (
                  <span className="px-1.5 py-0.2 rounded-full bg-amber-500/20 text-amber-700 dark:text-amber-300 font-mono font-bold text-[9px]">
                    {tab.badge}
                  </span>
                )}
              </button>
            );
          })}
        </div>
      </div>
    </header>
  );
}
