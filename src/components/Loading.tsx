import { Spinner } from "./Button"
import { cx } from "../lib/cx"

/** The busy state shown while a command is in flight. */
export function Loading({
  label = "Loading...",
  className,
}: {
  label?: string
  className?: string
}) {
  return (
    <div
      className={cx(
        "flex items-center justify-center gap-2 py-10 text-sm text-slate-500",
        className,
      )}
    >
      <Spinner className="h-4 w-4" />
      {label}
    </div>
  )
}

/** A single headline number on the dashboard. */
export function StatTile({
  label,
  value,
  hint,
  tone = "default",
}: {
  label: string
  value: React.ReactNode
  hint?: React.ReactNode
  tone?: "default" | "money" | "positive" | "negative"
}) {
  return (
    <div className="rounded-lg bg-white px-4 py-4 shadow-sm ring-1 ring-slate-200">
      <p className="text-xs font-medium uppercase tracking-wide text-slate-500">
        {label}
      </p>
      <p
        className={cx(
          "mt-1.5 text-2xl font-semibold tracking-tight",
          tone === "positive"
            ? "text-emerald-600"
            : tone === "negative"
              ? "text-rose-600"
              : "text-slate-900",
        )}
      >
        {value}
      </p>
      {hint ? <p className="mt-1 text-xs text-slate-500">{hint}</p> : null}
    </div>
  )
}
