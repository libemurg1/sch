import type { ReactNode } from "react"
import { cx } from "../lib/cx"

export interface CardProps {
  title?: ReactNode
  description?: ReactNode
  actions?: ReactNode
  className?: string
  bodyClassName?: string
  children: ReactNode
}

/** A titled panel. Every screen is built out of these. */
export function Card({
  title,
  description,
  actions,
  className,
  bodyClassName,
  children,
}: CardProps) {
  return (
    <section
      className={cx(
        "overflow-hidden rounded-lg bg-white shadow-sm ring-1 ring-slate-200",
        className,
      )}
    >
      {title || actions ? (
        <header className="flex flex-wrap items-start justify-between gap-3 border-b border-slate-200 px-4 py-3 sm:px-5">
          <div className="min-w-0">
            {title ? (
              <h2 className="text-sm font-semibold text-slate-900">{title}</h2>
            ) : null}
            {description ? (
              <p className="mt-0.5 text-xs text-slate-500">{description}</p>
            ) : null}
          </div>
          {actions ? (
            <div className="flex shrink-0 items-center gap-2">{actions}</div>
          ) : null}
        </header>
      ) : null}
      <div className={cx("px-4 py-4 sm:px-5", bodyClassName)}>{children}</div>
    </section>
  )
}

/** A heading for a page, with room for actions on the right. */
export function PageHeader({
  title,
  description,
  actions,
}: {
  title: string
  description?: ReactNode
  actions?: ReactNode
}) {
  return (
    <header className="flex flex-wrap items-end justify-between gap-4">
      <div>
        <h1 className="text-xl font-semibold tracking-tight text-slate-900">
          {title}
        </h1>
        {description ? (
          <p className="mt-1 text-sm text-slate-500">{description}</p>
        ) : null}
      </div>
      {actions ? (
        <div className="flex flex-wrap items-center gap-2">{actions}</div>
      ) : null}
    </header>
  )
}
