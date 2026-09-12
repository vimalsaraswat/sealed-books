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
