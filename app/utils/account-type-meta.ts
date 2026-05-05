import type { AccountTypeName } from "~/generated/bindings";

export interface AccountTypeMeta {
  label: string
  /**
   * Primary color used in dashboard allocation chart.
   */
  color: string
  glow: string
  glowEmphasis: string
  /**
   * Tailwind classes used for the account type badge.
   */
  badgeClass: string
  /**
   * Light/foreground color used for sparklines.
   */
  lineColor: string
  /**
   * Dark theme color used for sparklines.
   */
  lineColorDark: string
}

type AccountTypeConfig = Pick<AccountTypeMeta, "label" | "color" | "badgeClass" | "lineColorDark">;

const ACCOUNT_TYPE_CONFIG = {
  current: {
    label: "Current",
    color: "#3b82f6",
    badgeClass: "ring ring-inset bg-[#DBEAFE] text-[#1D4ED8] ring-[#60A5FA]/50 dark:bg-[#3B82F6]/20 dark:text-[#93C5FD] dark:ring-[#3B82F6]/45",
    lineColorDark: "#60A5FA"
  },
  savings: {
    label: "Savings",
    color: "#22c55e",
    badgeClass: "ring ring-inset bg-[#DCFCE7] text-[#15803D] ring-[#4ADE80]/50 dark:bg-[#16A34A]/15 dark:text-[#4ADE80] dark:ring-[#16A34A]/35",
    lineColorDark: "#34D399"
  },
  credit_card: {
    label: "Credit card",
    color: "#ef4444",
    badgeClass: "ring ring-inset bg-[#FEE2E2] text-[#B91C1C] ring-[#F87171]/55 dark:bg-[#DC2626]/15 dark:text-[#FCA5A5] dark:ring-[#DC2626]/35",
    lineColorDark: "#F87171"
  },
  isa: {
    label: "ISA",
    color: "#f97316",
    badgeClass: "ring ring-inset bg-[#FFEDD5] text-[#C2410C] ring-[#FB923C]/55 dark:bg-[#EA580C]/15 dark:text-[#FDBA74] dark:ring-[#EA580C]/35",
    lineColorDark: "#FB923C"
  },
  investment: {
    label: "Investment",
    color: "#a855f7",
    badgeClass: "ring ring-inset bg-[#F3E8FF] text-[#7E22CE] ring-[#C084FC]/55 dark:bg-[#7C3AED]/15 dark:text-[#C4B5FD] dark:ring-[#7C3AED]/35",
    lineColorDark: "#A78BFA"
  },
  pension: {
    label: "Pension",
    color: "#db2777",
    badgeClass: "ring ring-inset bg-[#FCE7F3] text-[#BE185D] ring-[#F472B6]/55 dark:bg-[#DB2777]/15 dark:text-[#FDA4AF] dark:ring-[#DB2777]/35",
    lineColorDark: "#F472B6"
  },
  cash: {
    label: "Cash",
    color: "#eab308",
    badgeClass: "ring ring-inset bg-[#FEF9C3] text-[#A16207] ring-[#FACC15]/65 dark:bg-[#CA8A04]/15 dark:text-[#FDE047] dark:ring-[#CA8A04]/35",
    lineColorDark: "#FACC15"
  },
  loan: {
    label: "Loan",
    color: "#14b8a6",
    badgeClass: "ring ring-inset bg-[#CCFBF1] text-[#0F766E] ring-[#2DD4BF]/55 dark:bg-[#0F766E]/15 dark:text-[#5EEAD4] dark:ring-[#0F766E]/35",
    lineColorDark: "#2DD4BF"
  }
} as const satisfies Record<AccountTypeName, AccountTypeConfig>;

function parseHexColor(hex: string): { r: number, g: number, b: number } | null {
  const m = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(hex.trim());
  const raw = m?.[1];
  if (raw == null) return null;

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

export const ACCOUNT_TYPE_META = Object.fromEntries(
  (Object.keys(ACCOUNT_TYPE_CONFIG) as AccountTypeName[]).map((kind) => {
    const cfg = ACCOUNT_TYPE_CONFIG[kind];
    const glow = rgbaFromHex(cfg.color, 0.55);
    const glowEmphasis = rgbaFromHex(cfg.color, 0.85);
    const lineColor = cfg.color;
    return [kind, { ...cfg, glow, glowEmphasis, lineColor }] as const;
  })
) as Record<AccountTypeName, AccountTypeMeta>;
