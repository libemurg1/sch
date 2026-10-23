import type { ReactNode } from "react"
import { cx } from "../lib/cx"

export type Tone = "neutral" | "info" | "success" | "warning" | "danger"

const TONES: Record<Tone, string> = {
  neutral: "bg-slate-100 text-slate-700 ring-slate-200",
  info: "bg-indigo-50 text-indigo-700 ring-indigo-200",
  success: "bg-emerald-50 text-emerald-700 ring-emerald-200",
  warning: "bg-amber-50 text-amber-800 ring-amber-200",
  danger: "bg-rose-50 text-rose-700 ring-rose-200",
}

export interface BadgeProps {
  tone?: Tone
  children: ReactNode
  className?: string
}

/** A small coloured label, used for statuses and kinds. */
export function Badge({ tone = "neutral", children, className }: BadgeProps) {
  return (
    <span
      className={cx(
        "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ring-1 ring-inset",
        TONES[tone],
        className,
      )}
    >
      {children}
    </span>
  )
}

/** Maps the backend's status strings onto a badge tone. */
export function statusTone(status: string): Tone {
  switch (status) {
    case "ACTIVE":
    case "PAID":
      return "success"
    case "PARTIAL":
    case "PENDING":
      return "warning"
    case "CANCELLED":
    case "INACTIVE":
      return "danger"
    case "GRADUATED":
      return "info"
    default:
      return "neutral"
  }
}

/** Renders a backend status string as a badge. */
export function StatusBadge({ status }: { status: string }) {
  return <Badge tone={statusTone(status)}>{status.replace(/_/g, " ")}</Badge>
}
