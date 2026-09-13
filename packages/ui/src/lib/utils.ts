import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatCurrency(amountMinor: number): string {
  const dollars = amountMinor / 100
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 2,
  }).format(dollars)
}

export function formatDate(dateStr: string): string {
  if (!dateStr) return ""
  return dateStr
}

export function truncateHash(hash: string, lead = 8, trail = 6): string {
  if (!hash) return ""
  if (hash.length <= lead + trail) return hash
  return `${hash.slice(0, lead)}...${hash.slice(-trail)}`
}


/**
 * Resolves the official, valid HashScan explorer URL for an attestation.
 * HashScan does not support /message/:seq paths; it expects:
 * - /transaction/:consensusTimestamp (exact transaction & payload)
 * - /topic/:topicId (topic message feed)
 */
export function getHashscanUrl(options: {
  consensusTimestamp?: string | null;
  topicId?: string | null;
  rawUrl?: string | null;
}): string | null {
  // If consensusTimestamp is valid Hedera format (e.g. 1789299468.637783404), link directly to transaction
  if (
    options.consensusTimestamp &&
    options.consensusTimestamp.trim().length > 0 &&
    !options.consensusTimestamp.includes("T")
  ) {
    return `https://hashscan.io/testnet/transaction/${options.consensusTimestamp.trim()}`;
  }
  if (options.topicId && options.topicId.trim().length > 0) {
    return `https://hashscan.io/testnet/topic/${options.topicId.trim()}`;
  }
  if (options.rawUrl && !options.rawUrl.includes("/message/")) {
    return options.rawUrl;
  }
  return null;
}

export function truncateAddress(addr?: string | null): string {
  if (!addr) return "";
  if (addr.length <= 10) return addr;
  return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
}
