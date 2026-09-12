import { useState } from "react";
import {
  Lock,
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

  const navItems: { id: NavTab; label: string; icon: React.ElementType; badge?: number }[] = [
    { id: "overview", label: "Overview", icon: LayoutDashboard },
    { id: "ledger", label: "General Ledger", icon: BookOpen },
    { id: "financials", label: "Financials", icon: Scale },
    {
      id: "audit",
      label: "Audit & Consensus",
      icon: ShieldCheck,
      badge: userRole === "auditor" && pendingAudits.length > 0 ? pendingAudits.length : undefined,
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
              <div className="size-8 rounded-lg bg-primary text-primary-foreground flex items-center justify-center shadow-xs">
                <Lock className="size-4.5" />
              </div>
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
                  Autonomous Accounting
                </p>
              </div>
            </div>

            {/* Mobile Close Button */}
            <button
              onClick={onMobileClose}
              className="md:hidden p-1.5 text-muted-foreground hover:text-foreground"
            >
              <X className="size-4" />
            </button>
          </div>

          {/* Organization Switcher Dropdown */}
          {activeOrg && (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button className="w-full flex items-center justify-between p-2 rounded-lg border border-border/70 bg-muted/40 hover:bg-muted/70 transition-colors text-left group">
                  <div className="flex items-center gap-2.5 min-w-0">
                    <div className="size-7 rounded bg-background border border-border/60 flex items-center justify-center shrink-0 text-muted-foreground group-hover:text-foreground">
                      <Building2 className="size-3.5" />
                    </div>
                    <div className="min-w-0">
                      <p className="text-xs font-bold text-foreground truncate">
                        {activeOrg.name}
                      </p>
                      <p className="text-[10px] font-mono uppercase text-muted-foreground">
                        {userRole}
                      </p>
                    </div>
                  </div>
                  <ChevronDown className="size-3.5 text-muted-foreground shrink-0 ml-1" />
                </button>
              </DropdownMenuTrigger>

              <DropdownMenuContent align="start" className="w-60">
                <DropdownMenuLabel className="text-[10px] uppercase font-mono tracking-wider">
                  Select Organization Vault
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
                        {isSelected && <Check className="size-3 text-primary" />}
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
        </div>

        {/* =========================================================
            MIDDLE SECTION: Vertical Workspaces Navigation
            ========================================================= */}
        <div className="flex-1 overflow-y-auto px-3 py-4 space-y-1">
          <div className="px-2 pb-2">
            <span className="text-[10px] font-mono uppercase font-semibold text-muted-foreground tracking-wider">
              Workspaces
            </span>
          </div>

          {navItems.map((item) => {
            const Icon = item.icon;
            const isActive = activeTab === item.id;
            return (
              <button
                key={item.id}
                onClick={() => handleSelectTab(item.id)}
                className={`w-full flex items-center justify-between px-3 py-2 text-xs font-medium rounded-lg transition-all ${
                  isActive
                    ? "bg-primary text-primary-foreground font-semibold shadow-xs"
                    : "text-muted-foreground hover:text-foreground hover:bg-muted/60"
                }`}
              >
                <div className="flex items-center gap-2.5">
                  <Icon className="size-4 shrink-0" />
                  <span>{item.label}</span>
                </div>

                {item.badge !== undefined && (
                  <span
                    className={`px-1.5 py-0.2 rounded-full font-mono font-bold text-[10px] ${
                      isActive
                        ? "bg-white/20 text-white"
                        : "bg-amber-500/20 text-amber-700 dark:text-amber-300 animate-pulse"
                    }`}
                  >
                    {item.badge}
                  </span>
                )}
              </button>
            );
          })}
        </div>

        {/* =========================================================
            BOTTOM SECTION: User Card & Sign Out
            ========================================================= */}
        <div className="p-3 border-t border-border/70 space-y-2">
          {currentUser && (
            <div className="p-2.5 rounded-lg border border-border/60 bg-muted/30 space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 min-w-0">
                  <div className="size-7 rounded-full bg-primary/15 text-primary border border-primary/20 flex items-center justify-center font-bold text-[10px] shrink-0">
                    {userInitials}
                  </div>
                  <div className="min-w-0">
                    <p className="text-xs font-bold text-foreground truncate">
                      {currentUser.name}
                    </p>
                    <p className="text-[10px] text-muted-foreground font-mono truncate">
                      {currentUser.email}
                    </p>
                  </div>
                </div>

                <Badge
                  variant="outline"
                  className="text-[9px] font-mono uppercase py-0 shrink-0"
                >
                  {userRole}
                </Badge>
              </div>

              {currentUser.eth_address && (
                <div className="flex items-center justify-between p-1.5 rounded bg-background border border-border/40 font-mono text-[10px] text-muted-foreground">
                  <span>{truncateAddress(currentUser.eth_address)}</span>
                  <button
                    onClick={handleCopyAddress}
                    title="Copy Ethereum address"
                    className="hover:text-foreground inline-flex items-center gap-1"
                  >
                    {copiedAddr ? (
                      <Check className="size-3 text-emerald-500" />
                    ) : (
                      <Copy className="size-3" />
                    )}
                  </button>
                </div>
              )}

              <Button
                variant="ghost"
                size="sm"
                onClick={() => logout()}
                className="w-full h-7 text-xs text-muted-foreground hover:text-destructive hover:bg-destructive/10 justify-start gap-2 px-2"
              >
                <LogOut className="size-3.5" />
                <span>Sign Out</span>
              </Button>
            </div>
          )}
        </div>
      </aside>
    </>
  );
}
