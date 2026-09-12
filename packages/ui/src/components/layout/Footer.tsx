import { Database } from "lucide-react";

export function Footer() {
  return (
    <footer className="mt-auto border-t border-border bg-card/60 py-6">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col sm:flex-row items-center justify-between gap-4 text-xs text-muted-foreground">
        <div className="flex items-center gap-2">
          <Database className="size-4 text-muted-foreground" />
          <span>SQLite Multi-Tenant Engine • Cryptographic Journal</span>
        </div>
        <div className="font-mono text-[11px]">
          Dual-Custody Secp256k1 Signatures • Hedera HCS Anchor Topic
          0.0.10462941
        </div>
      </div>
    </footer>
  );
}
