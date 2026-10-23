import type { InvoiceItem } from "../types/schema/InvoiceItem"
import { formatMoney } from "../lib/money"
import { BankDetails, DocumentFooter, SchoolLetterhead } from "./SchoolBlocks"

export interface InvoiceDocumentProps {
  invoiceNumber: string
  kind: string
  studentName: string
  studentNumber: string
  issueDate: string
  dueDate: string
  items: InvoiceItem[]
  subtotalCents: number
  discountCents: number
  totalCents: number
  paidCents: number
  status: string
}

/**
 * The printed invoice. It carries no interactive elements so it prints
 * cleanly, and the charge lines and discount lines are kept apart so a
 * reader can see how the total was reached.
 */
export function InvoiceDocument(props: InvoiceDocumentProps) {
  const charges = props.items.filter((item) => item.kind !== "DISCOUNT")
  const discounts = props.items.filter((item) => item.kind === "DISCOUNT")
  const balance = props.totalCents - props.paidCents

  return (
    <div className="mx-auto max-w-3xl bg-white p-8 text-slate-900">
      <SchoolLetterhead
        subtitle="Fee invoice"
        right={
          <>
            <p className="font-mono text-base font-semibold">
              {props.invoiceNumber}
            </p>
            <p className="mt-0.5 text-slate-600">
              {props.kind === "SUPPLEMENTARY" ? "Supplementary" : "Term"} invoice
            </p>
          </>
        }
      />

      <section className="mt-4 grid grid-cols-2 gap-4 text-sm">
        <div>
          <p className="text-xs font-semibold uppercase tracking-wide text-slate-500">
            Billed to
          </p>
          <p className="mt-1 font-medium">{props.studentName}</p>
          <p className="font-mono text-xs text-slate-600">
            {props.studentNumber}
          </p>
        </div>
        <div className="text-right">
          <p className="text-xs font-semibold uppercase tracking-wide text-slate-500">
            Dates
          </p>
          <p className="mt-1">Issued: {props.issueDate}</p>
          <p>Due: {props.dueDate}</p>
        </div>
      </section>

      <table className="mt-6 w-full text-sm">
        <thead>
          <tr className="border-b border-slate-300 text-left text-xs uppercase tracking-wide text-slate-500">
            <th className="py-2">Description</th>
            <th className="py-2 text-right">Amount</th>
          </tr>
        </thead>
        <tbody>
          {charges.map((item) => (
            <tr key={item.id} className="border-b border-slate-100">
              <td className="py-2">{item.description}</td>
              <td className="py-2 text-right">
                {formatMoney(item.amountCents)}
              </td>
            </tr>
          ))}
          {discounts.length > 0 ? (
            <tr className="border-b border-slate-100">
              <td className="py-2">Subtotal</td>
              <td className="py-2 text-right">{formatMoney(props.subtotalCents)}</td>
            </tr>
          ) : null}
          {discounts.map((item) => (
            <tr key={item.id} className="border-b border-slate-100 text-slate-600">
              <td className="py-2">Discount: {item.description}</td>
              <td className="py-2 text-right">{formatMoney(item.amountCents)}</td>
            </tr>
          ))}
        </tbody>
        <tfoot>
          <tr>
            <td className="pt-3 text-right font-medium">Total</td>
            <td className="pt-3 text-right font-semibold">
              {formatMoney(props.totalCents)}
            </td>
          </tr>
          <tr>
            <td className="text-right font-medium">Paid</td>
            <td className="text-right">{formatMoney(props.paidCents)}</td>
          </tr>
          <tr className="border-t border-slate-300">
            <td className="pt-2 text-right font-semibold">Balance due</td>
            <td className="pt-2 text-right text-base font-bold">
              {formatMoney(balance)}
            </td>
          </tr>
        </tfoot>
      </table>

      <p className="mt-8 text-xs text-slate-500">
        Status: {props.status}. Please quote {props.invoiceNumber} when paying.
      </p>

      <BankDetails />
      <DocumentFooter kind="invoice" />
    </div>
  )
}
