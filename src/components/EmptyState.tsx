import type { ReactNode } from "react"
import { cx } from "../lib/cx"

export interface EmptyStateProps {
  title: string
  description?: ReactNode
  action?: ReactNode
  className?: string
}

/** Shown when a list has nothing in it. */
export function EmptyState({
  title,
  description,
  action,
  className,
}: EmptyStateProps) {
  return (
    <div
      className={cx(
        "flex flex-col items-center justify-center gap-2 rounded-md border border-dashed border-slate-300 px-6 py-10 text-center",
        className,
      )}
    >
      <p className="text-sm font-medium text-slate-700">{title}</p>
      {description ? (
        <p className="max-w-sm text-xs text-slate-500">{description}</p>
      ) : null}
      {action ? <div className="mt-2">{action}</div> : null}
    </div>
  )
}
