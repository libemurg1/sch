/**
 * The backend stores dates as plain `YYYY-MM-DD` strings and compares them
 * lexicographically, so the UI must always send that exact format. Times are
 * stored as `YYYY-MM-DDTHH:MM:SSZ`.
 */
import { format, parseISO } from "date-fns"

/** Formats a `YYYY-MM-DD` date for display, e.g. `2026-01-12` -> `12 Jan 2026`. */
export function formatDate(value: string | null | undefined): string {
  if (!value) {
    return "-"
  }
  const parsed = parseISO(value)
  if (Number.isNaN(parsed.getTime())) {
    return value
  }
  return format(parsed, "d MMM yyyy")
}

/** Formats an ISO timestamp for display, e.g. `12 Jan 2026, 14:03`. */
export function formatDateTime(value: string | null | undefined): string {
  if (!value) {
    return "-"
  }
  const parsed = parseISO(value)
  if (Number.isNaN(parsed.getTime())) {
    return value
  }
  return format(parsed, "d MMM yyyy, HH:mm")
}

/** Returns a `YYYY-MM-DD` string for the current browser date. */
export function todayInputValue(): string {
  return format(new Date(), "yyyy-MM-dd")
}

/** Turns a `YYYY-MM-DD` string into the browser's `YYYY-MM-DD` input format. */
export function toInputDate(value: string | null | undefined): string {
  if (!value) {
    return ""
  }
  return value.slice(0, 10)
}

/** Formats a term as `Term 1, 2026`. */
export function formatTermLabel(term: { year: number; termNumber: number }): string {
  return `Term ${term.termNumber}, ${term.year}`
}

/** Formats a term as `Term 1, 2026 (1 Jan - 31 Mar)`. */
export function formatTermRange(term: {
  year: number
  termNumber: number
  startDate: string
  endDate: string
}): string {
  return `${formatTermLabel(term)} (${formatDate(term.startDate)} - ${formatDate(term.endDate)})`
}

/** Formats a student's full name. */
export function studentName(student: {
  firstName: string
  lastName: string
}): string {
  return `${student.firstName} ${student.lastName}`.trim()
}
