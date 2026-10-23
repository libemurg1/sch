import type { ReactNode } from "react"
import { describeError } from "../api/errors"
import { cx } from "../lib/cx"

export interface ErrorBannerProps {
  error: unknown
  onRetry?: () => void
  className?: string
  children?: ReactNode
}

/** Shows a failed command in a way a person can act on. */
export function ErrorBanner({
  error,
  onRetry,
  className,
  children,
}: ErrorBannerProps) {
  if (error === null || error === undefined) {
    return null
  }

  return (
    <div
      role="alert"
      className={cx(
        "flex flex-wrap items-start gap-3 rounded-md bg-rose-50 px-4 py-3 ring-1 ring-inset ring-rose-200",
        className,
      )}
    >
      <svg
        className="mt-0.5 h-4 w-4 shrink-0 text-rose-600"
        viewBox="0 0 20 20"
        fill="currentColor"
        aria-hidden="true"
      >
        <path
          fillRule="evenodd"
          d="M10 18a8 8 0 100-16 8 8 0 000 16zM9 5a1 1 0 012 0v5a1 1 0 11-2 0V5zm1 10a1.25 1.25 0 100-2.5 1.25 1.25 0 000 2.5z"
          clipRule="evenodd"
        />
      </svg>
      <div className="min-w-0 flex-1">
        <p className="text-sm text-rose-800">{describeError(error)}</p>
        {children}
      </div>
      {onRetry ? (
        <button
          type="button"
          onClick={onRetry}
          className="shrink-0 rounded-md bg-white px-2.5 py-1 text-xs font-medium text-rose-700 shadow-sm ring-1 ring-inset ring-rose-200 hover:bg-rose-100"
        >
          Try again
        </button>
      ) : null}
    </div>
  )
}
