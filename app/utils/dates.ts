import { parseDate } from "@internationalized/date";

export function getTodayCalendarDateIsoString() {
  const now = new Date();
  const year = String(now.getFullYear());
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function addDaysToCalendarDateIsoString(isoDate: string, days: number) {
  const [year, month, day] = isoDate.split("-").map((part) => Number.parseInt(part, 10));
  const nextDate = new Date(Date.UTC(year!, month! - 1, day! + days));
  return nextDate.toISOString().slice(0, 10);
}

export function getCalendarDateModelValueFromIsoString(isoDate: string) {
  return isoDate === "" ? null : parseDate(isoDate);
}

export function getDateObjectFromCalendarDateIsoString(isoDate: string | null | undefined) {
  if (isoDate == null || isoDate === "") {
    return null;
  }

  return new Date(`${isoDate}T00:00:00`);
}

export function getUtcMillisecondsFromCalendarDateIsoString(isoDate: string) {
  const parts = isoDate.split("-");
  if (parts.length !== 3) return Number.NaN;

  const year = Number.parseInt(parts[0]!, 10);
  const month = Number.parseInt(parts[1]!, 10);
  const day = Number.parseInt(parts[2]!, 10);

  if (!Number.isFinite(year) || !Number.isFinite(month) || !Number.isFinite(day)) return Number.NaN;
  return Date.UTC(year, month - 1, day);
}

export function getCalendarDateIsoStringFromUtcMilliseconds(ms: number) {
  return new Date(ms).toISOString().slice(0, 10);
}

export function getCalendarDateIsoStringFromInputValue(value: unknown) {
  if (value == null || typeof value !== "object" || typeof value.toString !== "function") {
    return "";
  }

  const isoDate = value.toString();
  return /^\d{4}-\d{2}-\d{2}$/.test(isoDate) ? isoDate : "";
}
