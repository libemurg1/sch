import type { ReactNode } from "react"
import { cx } from "../lib/cx"

export interface FieldProps {
  label: string
  htmlFor?: string
  hint?: ReactNode
  error?: string | null
  required?: boolean
  className?: string
  children: ReactNode
}

/** A labelled form control with room for a hint and a validation message. */
export function Field({
  label,
  htmlFor,
  hint,
  error,
  required,
  className,
  children,
}: FieldProps) {
  return (
    <div className={cx("flex flex-col gap-1", className)}>
      <label
        htmlFor={htmlFor}
        className="text-sm font-medium text-slate-700"
      >
        {label}
        {required ? <span className="ml-0.5 text-rose-600">*</span> : null}
      </label>
      {children}
      {error ? (
        <p className="text-xs text-rose-600">{error}</p>
      ) : hint ? (
        <p className="text-xs text-slate-500">{hint}</p>
      ) : null}
    </div>
  )
}

const CONTROL_BASE =
  "block w-full rounded-md border-0 bg-white px-3 py-2 text-sm text-slate-900 shadow-sm " +
  "ring-1 ring-inset ring-slate-300 placeholder:text-slate-400 " +
  "focus:ring-2 focus:ring-inset focus:ring-indigo-600 " +
  "disabled:bg-slate-50 disabled:text-slate-500"

export function controlClassName(error?: string | null): string {
  return cx(
    CONTROL_BASE,
    error ? "ring-rose-400 focus:ring-rose-500" : null,
  )
}
