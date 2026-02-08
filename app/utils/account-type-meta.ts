import type { AccountTypeName } from "~/bindings";

export interface AccountTypeMeta {
  label: string
  /**
   * Primary color used in dashboard allocation chart.
   */
  color: string
  glow: string
  glowEmphasis: string
  /**
   * Tailwind classes used for the account type badge in the accounts table.
   *
   * Note: this intentionally omits the shared `ring ring-inset` prefix so callers can
   * provide a consistent baseline.
   */
  badgeClass: string
  /**
   * Light/foreground color used for sparklines on the accounts page.
   */
  lineColor: string
}

type AccountTypeConfig = Pick<AccountTypeMeta, "label" | "color" | "badgeClass">;

const ACCOUNT_TYPE_CONFIG = {
  current: {
    label: "Current",
    color: "#3b82f6",
    badgeClass: "bg-[#3B82F6]/20 text-[#93C5FD] ring-[#3B82F6]/45"
  },
  savings: {
    label: "Savings",
    color: "#22c55e",
    badgeClass: "bg-[#16A34A]/15 text-[#4ADE80] ring-[#16A34A]/35"
  },
  credit_card: {
    label: "Credit card",
    color: "#ef4444",
    badgeClass: "bg-[#DC2626]/15 text-[#FCA5A5] ring-[#DC2626]/35"
  },
  isa: {
    label: "ISA",
    color: "#f97316",
    badgeClass: "bg-[#EA580C]/15 text-[#FDBA74] ring-[#EA580C]/35"
  },
  investment: {
    label: "Investment",
    color: "#a855f7",
    badgeClass: "bg-[#7C3AED]/15 text-[#C4B5FD] ring-[#7C3AED]/35"
  },
  pension: {
    label: "Pension",
    color: "#db2777",
    badgeClass: "bg-[#DB2777]/15 text-[#FDA4AF] ring-[#DB2777]/35"
  },
  cash: {
    label: "Cash",
    color: "#eab308",
    badgeClass: "bg-[#CA8A04]/15 text-[#FDE047] ring-[#CA8A04]/35"
  },
  loan: {
    label: "Loan",
    color: "#14b8a6",
    badgeClass: "bg-[#0F766E]/15 text-[#5EEAD4] ring-[#0F766E]/35"
  }
} as const satisfies Record<AccountTypeName, AccountTypeConfig>;

function parseHexColor(hex: string): { r: number, g: number, b: number } | null {
  const m = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(hex.trim());
  const raw = m?.[1];
  if (!raw)
    return null;

  const full = raw.length === 3
    ? raw.split("").map((c) => c + c).join("")
    : raw;

  const n = Number.parseInt(full, 16);
  return {
    r: (n >> 16) & 255,
    g: (n >> 8) & 255,
    b: n & 255
  };
}

function rgbaFromHex(hex: string, alpha: number): string {
  const rgb = parseHexColor(hex);
  if (!rgb)
    return `rgba(148, 163, 184, ${alpha})`;

  return `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alpha})`;
}

function lineColorFromBadgeClass(badgeClass: string): string | null {
  const m = /\btext-\[#([0-9a-f]{6})\]/i.exec(badgeClass);
  if (!m)
    return null;

  return `#${m[1]}`;
}

export const ACCOUNT_TYPE_META = Object.fromEntries(
  (Object.keys(ACCOUNT_TYPE_CONFIG) as AccountTypeName[]).map((kind) => {
    const cfg = ACCOUNT_TYPE_CONFIG[kind];
    const glow = rgbaFromHex(cfg.color, 0.55);
    const glowEmphasis = rgbaFromHex(cfg.color, 0.85);
    const lineColor = lineColorFromBadgeClass(cfg.badgeClass) ?? "#94A3B8";
    return [kind, { ...cfg, glow, glowEmphasis, lineColor }] as const;
  })
) as Record<AccountTypeName, AccountTypeMeta>;

export function accountTypeLabel(kind: string) {
  return accountTypeMetaLoose(kind)?.label ?? kind;
}

export function accountTypeBadgeClass(kind: string) {
  const badgeClass = accountTypeMetaLoose(kind)?.badgeClass;
  return `ring ring-inset ${badgeClass ?? "bg-elevated text-default ring-accented"}`;
}

export function accountTypeLineColor(kind: string) {
  return accountTypeMetaLoose(kind)?.lineColor ?? "#94A3B8";
}

export function accountTypeMetaLoose(kind: string): AccountTypeMeta | undefined {
  return (ACCOUNT_TYPE_META as Record<string, AccountTypeMeta | undefined>)[kind];
}
