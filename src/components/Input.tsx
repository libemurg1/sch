import type {
  InputHTMLAttributes,
  SelectHTMLAttributes,
  TextareaHTMLAttributes,
} from "react"
import { controlClassName } from "./Field"

export interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  error?: string | null
}

export function Input({ error, className, ...rest }: InputProps) {
  return (
    <input
      className={cxControl(error, className)}
      aria-invalid={error ? true : undefined}
      {...rest}
    />
  )
}

export interface TextareaProps
  extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  error?: string | null
}

export function Textarea({ error, className, ...rest }: TextareaProps) {
  return (
    <textarea
      className={cxControl(error, className)}
      aria-invalid={error ? true : undefined}
      {...rest}
    />
  )
}

export interface SelectProps
  extends SelectHTMLAttributes<HTMLSelectElement> {
  error?: string | null
}

export function Select({ error, className, children, ...rest }: SelectProps) {
  return (
    <select
      className={cxControl(error, className)}
      aria-invalid={error ? true : undefined}
      {...rest}
    >
      {children}
    </select>
  )
}

function cxControl(error: string | null | undefined, className?: string) {
  return [controlClassName(error), className].filter(Boolean).join(" ")
}
