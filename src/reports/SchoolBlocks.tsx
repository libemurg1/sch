import type { ReactNode } from "react"
import { useSchoolSettings } from "../app/session"

/**
 * The printed identity of the school. Uses the school settings so an invoice
 * carries the school's own name, address and logo rather than the app's name.
 */
export function SchoolLetterhead({
  subtitle,
  right,
  compact = false,
}: {
  subtitle: string
  right?: ReactNode
  compact?: boolean
}) {
  const { settings } = useSchoolSettings()
  const name = settings?.schoolName ?? "School Fees System"
  const contact = [settings?.address, settings?.phone, settings?.email].filter(
    (value): value is string => Boolean(value && value.trim() !== ""),
  )

  if (compact) {
    return (
      <div className="text-center">
        {settings?.logoData ? (
          <img
            src={settings.logoData}
            alt=""
            className="mx-auto mb-1 h-9 object-contain"
          />
        ) : null}
        <h1 className="text-sm font-bold uppercase tracking-wide leading-tight">
          {name}
        </h1>
        <p className="mt-0.5 text-[11px] text-slate-600">{subtitle}</p>
        {contact.map((line) => (
          <p key={line} className="text-[10px] leading-snug text-slate-500">
            {line}
          </p>
        ))}
      </div>
    )
  }

  return (
    <header className="flex items-start justify-between gap-6 border-b border-slate-300 pb-4">
      <div className="flex min-w-0 items-start gap-3">
        {settings?.logoData ? (
          <img
            src={settings.logoData}
            alt=""
            className="h-14 w-14 shrink-0 object-contain"
          />
        ) : null}
        <div className="min-w-0">
          <h1 className="text-lg font-bold uppercase tracking-wide leading-tight">
            {name}
          </h1>
          <p className="mt-0.5 text-sm text-slate-600">{subtitle}</p>
          {contact.length > 0 ? (
            <div className="mt-1 space-y-0.5 text-xs text-slate-500">
              {contact.map((line) => (
                <p key={line}>{line}</p>
              ))}
            </div>
          ) : null}
        </div>
      </div>
      {right ? <div className="shrink-0 text-right text-sm">{right}</div> : null}
    </header>
  )
}

/** The "where to pay" block on an invoice, only shown when it is configured. */
export function BankDetails({ compact = false }: { compact?: boolean }) {
  const { settings } = useSchoolSettings()
  if (!settings) {
    return null
  }

  const rows: Array<[string, string | null]> = [
    ["Bank", settings.bankName],
    ["Account name", settings.bankAccountName],
    ["Account number", settings.bankAccountNumber],
    ["Branch", settings.bankBranch],
    ["SWIFT / sort code", settings.bankSwift],
  ]
  const filled = rows.filter(
    (entry): entry is [string, string] => Boolean(entry[1] && entry[1].trim() !== ""),
  )

  if (filled.length === 0) {
    return null
  }

  return (
    <section className={compact ? "mt-3" : "mt-6"}>
      <p
        className={
          compact
            ? "text-[10px] font-semibold uppercase tracking-wide"
            : "text-xs font-semibold uppercase tracking-wide text-slate-500"
        }
      >
        Payment details
      </p>
      <dl className={compact ? "mt-1 space-y-0.5 text-[11px]" : "mt-2 grid gap-x-8 gap-y-1 text-sm sm:grid-cols-2"}>
        {filled.map(([label, value]) => (
          <div
            key={label}
            className={compact ? "flex justify-between gap-2" : "flex gap-2"}
          >
            <dt className="text-slate-500">{label}</dt>
            <dd className={compact ? "text-right font-medium" : "font-medium text-slate-800"}>
              {value}
            </dd>
          </div>
        ))}
      </dl>
    </section>
  )
}

/** The optional free-text footer configured in the school settings. */
export function DocumentFooter({ kind }: { kind: "invoice" | "receipt" }) {
  const { settings } = useSchoolSettings()
  const text =
    kind === "invoice" ? settings?.invoiceFooter : settings?.receiptFooter

  if (!text) {
    return null
  }

  return (
    <p
      className={
        kind === "invoice"
          ? "mt-6 border-t border-slate-200 pt-3 text-xs text-slate-500"
          : "mt-3 text-center text-[10px] text-slate-600"
      }
    >
      {text}
    </p>
  )
}
