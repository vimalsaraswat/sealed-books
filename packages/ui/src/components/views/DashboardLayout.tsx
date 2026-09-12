import { useState } from "react";
import { Loader2 } from "lucide-react";
import { Sidebar } from "../layout/Sidebar";
import { Header } from "../layout/Header";
import { Footer } from "../layout/Footer";
import { ModalsContainer } from "../layout/ModalsContainer";
import { OverviewView } from "./OverviewView";
import { LedgerView } from "./LedgerView";
import { FinancialsView } from "./FinancialsView";
import { AuditCenterView } from "./AuditCenterView";
import { TeamView } from "./TeamView";
import { AuthScreen } from "./AuthScreen";

import { useAuth } from "../../context/AuthContext";
import { useLedger } from "../../context/LedgerContext";

export function DashboardLayout() {
  const { isAuthenticated, isLoadingAuth } = useAuth();
  const { activeTab } = useLedger();
  const [mobileSidebarOpen, setMobileSidebarOpen] = useState(false);

  if (isLoadingAuth) {
    return (
      <div className="min-h-screen bg-background flex flex-col items-center justify-center text-muted-foreground text-xs font-mono">
        <Loader2 className="size-6 animate-spin text-primary mb-2" />
        <span>Authenticating secure vault session...</span>
      </div>
    );
  }

  if (!isAuthenticated) {
    return <AuthScreen />;
  }

  return (
    <div className="min-h-screen bg-background text-foreground flex font-sans">
      {/* Dedicated Left Sidebar Navigation */}
      <Sidebar
        mobileOpen={mobileSidebarOpen}
        onMobileClose={() => setMobileSidebarOpen(false)}
      />

      {/* Main Workspace with Streamlined Top Header */}
      <div className="flex-1 flex flex-col md:pl-64 min-w-0">
        <Header onMobileMenuToggle={() => setMobileSidebarOpen(true)} />

        <main className="flex-1 max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-6">
          {activeTab === "overview" && <OverviewView />}
          {activeTab === "ledger" && <LedgerView />}
          {activeTab === "financials" && <FinancialsView />}
          {activeTab === "audit" && <AuditCenterView />}
          {activeTab === "team" && <TeamView />}
        </main>

        <Footer />
      </div>

      <ModalsContainer />
    </div>
  );
}
