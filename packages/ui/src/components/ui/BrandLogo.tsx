import logoUrl from "../../assets/logo.png";

export interface BrandLogoProps {
  size?: "xs" | "sm" | "md" | "lg" | "xl" | number;
  className?: string;
  alt?: string;
}

const SIZE_MAP: Record<string, string> = {
  xs: "size-6",
  sm: "size-8",
  md: "size-10",
  lg: "size-12",
  xl: "size-16",
};

export function BrandLogo({
  size = "sm",
  className = "",
  alt = "Sealed Books Logo",
}: BrandLogoProps) {
  const sizeClass = typeof size === "string" ? SIZE_MAP[size] || "size-8" : "";
  const inlineStyle = typeof size === "number" ? { width: size, height: size } : undefined;

  return (
    <div
      style={inlineStyle}
      className={`relative inline-flex items-center justify-center shrink-0 rounded-xl overflow-hidden shadow-xs ring-1 ring-black/5 dark:ring-white/10 ${sizeClass} ${className}`}
    >
      <img
        src={logoUrl}
        alt={alt}
        className="w-full h-full object-cover select-none pointer-events-none"
        loading="eager"
      />
    </div>
  );
}
