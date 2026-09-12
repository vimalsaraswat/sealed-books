import { useState } from "react";
import {
  Building2,
  ChevronDown,
  Check,
  Plus,
  LayoutDashboard,
  BookOpen,
  Scale,
  ShieldCheck,
  Users,
  LogOut,
  Copy,
  X,
} from "lucide-react";
import { Badge } from "../ui/badge";
import { Button } from "../ui/button";
import { BrandLogo } from "../ui/BrandLogo";
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

export interface SidebarProps {
  mobileOpen?: boolean;
  onMobileClose?: () => void;
}

export function Sidebar({ mobileOpen = false, onMobileClose }: SidebarProps) {
  const {
    currentUser,
    activeOrg,
    userRole,
    availableOrgs,
    pendingAudits,
    switchOrg,
    logout,
  } = useAuth();

  const { activeTab, setActiveTab } = useLedger();
  const { setIsCreateOrgOpen } = useModals();

  const [copiedAddr, setCopiedAddr] = useState(false);

  const handleCopyAddress = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!currentUser?.eth_address) return;
    navigator.clipboard.writeText(currentUser.eth_address);
    setCopiedAddr(true);
    setTimeout(() => setCopiedAddr(false), 1500);
  };

  const navItems: {
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
    { id: "team", label: "Team & Governance", icon: Users },
  ];

  const userInitials =
    currentUser?.name
      ?.split(" ")
      .map((n) => n[0])
      .slice(0, 2)
      .join("")
      .toUpperCase() || "U";

  const handleSelectTab = (tabId: NavTab) => {
    setActiveTab(tabId);
    if (onMobileClose) {
      onMobileClose();
    }
  };

  return (
    <>
      {/* Mobile Backdrop Overlay */}
      {mobileOpen && (
        <div
          className="fixed inset-0 z-40 bg-black/60 backdrop-blur-xs md:hidden"
          onClick={onMobileClose}
        />
      )}

      {/* Sidebar Container */}
      <aside
        className={`fixed top-0 bottom-0 left-0 z-50 w-64 border-r border-border bg-card flex flex-col justify-between transition-transform duration-200 ease-in-out md:translate-x-0 ${
          mobileOpen ? "translate-x-0" : "-translate-x-full"
        }`}
      >
        {/* =========================================================
            TOP SECTION: Brand Lockup & Org Switcher
            ========================================================= */}
        <div className="p-4 space-y-4 border-b border-border/70">
          {/* Brand Row */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2.5">
              <BrandLogo size="sm" />
              <div>
                <div className="flex items-center gap-1.5">
                  <span className="font-bold text-sm tracking-tight text-foreground">
                    Sealed Books
                  </span>
                  <span className="text-[9px] font-mono px-1.5 py-0.2 rounded-sm bg-muted text-muted-foreground border border-border/60">
                    HCS
                  </span>
                </div>
                <p className="text-[10px] text-muted-foreground font-mono">
                  Multi-Party Ledger
                </p>
              </div>
            </div>

            {/* Mobile Close Button */}
            {mobileOpen && (
              <button
                onClick={onMobileClose}
                className="md:hidden p-1 rounded-md text-muted-foreground hover:text-foreground hover:bg-muted"
              >
                <X className="size-4.5" />
              </button>
            )}
          </div>

          {/* Tenant Context Selector */}
          <div className="space-y-1">
            <div className="flex items-center justify-between px-0.5">
              <span className="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">
                Organization
              </span>
              <span className="text-[10px] font-mono text-muted-foreground">
                {activeOrg?.base_currency || "USD"}
              </span>
            </div>

            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button className="w-full flex items-center justify-between gap-2 px-3 py-2 rounded-lg border border-border bg-background/60 hover:bg-accent/50 text-left transition-colors cursor-pointer group">
                  <div className="flex items-center gap-2 min-w-0">
                    <div className="size-5 rounded-md bg-primary/10 text-primary flex items-center justify-center shrink-0">
                      <Building2 className="size-3.5" />
                    </div>
                    <span className="text-xs font-semibold text-foreground truncate">
                      {activeOrg?.name || "Select Organization"}
                    </span>
                  </div>
                  <ChevronDown className="size-3.5 text-muted-foreground group-hover:text-foreground shrink-0 transition-transform" />
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
          </div>
        </div>

        {/* =========================================================
            MIDDLE SECTION: Main Navigation Links
            ========================================================= */}
        <div className="flex-1 px-3 py-4 space-y-1 overflow-y-auto">
          <div className="px-2 pb-1.5 text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
            Ledger & Consensus
          </div>
          {navItems.map((item) => {
            const Icon = item.icon;
            const isActive = activeTab === item.id;
            return (
              <button
                key={item.id}
                onClick={() => handleSelectTab(item.id)}
                className={`w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs font-medium transition-colors cursor-pointer group ${
                  isActive
                    ? "bg-primary text-primary-foreground font-semibold shadow-xs"
                    : "text-muted-foreground hover:text-foreground hover:bg-accent/60"
                }`}
              >
                <div className="flex items-center gap-2.5 min-w-0">
                  <Icon
                    className={`size-4 shrink-0 transition-colors ${
                      isActive
                        ? "text-primary-foreground"
                        : "text-muted-foreground group-hover:text-foreground"
                    }`}
                  />
                  <span className="truncate">{item.label}</span>
                </div>
                {item.badge !== undefined && (
                  <Badge
                    variant={isActive ? "secondary" : "destructive"}
                    className="text-[10px] px-1.5 py-0 rounded-full shrink-0"
                  >
                    {item.badge}
                  </Badge>
                )}
              </button>
            );
          })}
        </div>

        {/* =========================================================
            BOTTOM SECTION: User Identity & Ledger Anchors
            ========================================================= */}
        <div className="p-3 border-t border-border/70 space-y-3 bg-card">
          {/* User Profile Card */}
          <div className="p-2.5 rounded-lg border border-border/70 bg-background/50 flex items-center justify-between gap-2">
            <div className="flex items-center gap-2.5 min-w-0">
              <div className="size-8 rounded-full bg-primary/10 text-primary border border-primary/20 flex items-center justify-center font-bold text-xs shrink-0">
                {userInitials}
              </div>
              <div className="min-w-0">
                <div className="text-xs font-semibold text-foreground truncate">
                  {currentUser?.name || "Accounting Officer"}
                </div>
                <div className="flex items-center gap-1.5">
                  <Badge
                    variant="outline"
                    className="text-[9px] px-1 py-0 h-4 capitalize font-mono text-muted-foreground bg-muted/40"
                  >
                    {userRole || "viewer"}
                  </Badge>
                  {currentUser?.eth_address && (
                    <button
                      onClick={handleCopyAddress}
                      title="Copy Public Signing Address"
                      className="text-[10px] font-mono text-muted-foreground hover:text-foreground flex items-center gap-0.5 cursor-pointer"
                    >
                      <span>{truncateAddress(currentUser.eth_address)}</span>
                      <Copy className="size-2.5" />
                      {copiedAddr && (
                        <span className="text-[9px] text-emerald-500 font-sans">
                          ✓
                        </span>
                      )}
                    </button>
                  )}
                </div>
              </div>
            </div>

            {/* Logout Icon Button */}
            <Button
              variant="ghost"
              size="icon"
              onClick={logout}
              title="Sign Out"
              className="size-7 text-muted-foreground hover:text-destructive shrink-0 cursor-pointer"
            >
              <LogOut className="size-3.5" />
            </Button>
          </div>
        </div>
      </aside>
    </>
  );
}
