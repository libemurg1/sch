import type { Invoice } from "../types/schema/Invoice"
import type { Payment } from "../types/schema/Payment"
import type { Term } from "../types/schema/Term"
import { formatMoney } from "../lib/money"
import { BankDetails, DocumentFooter, SchoolLetterhead } from "./SchoolBlocks"

export interface StatementDocumentProps {
  studentName: string
  studentNumber: string
  term: Term | null
  invoices: Invoice[]
  payments: Payment[]
  totalBilledCents: number
  totalPaidCents: number
  creditCents: number
  balanceCents: number
  generatedOn: string
}

/** The printed statement: everything billed, everything paid, and the balance. */
export function StatementDocument(props: StatementDocumentProps) {
  return (
    <div className="mx-auto max-w-3xl bg-white p-8 text-slate-900">
      <SchoolLetterhead
        subtitle="Student statement"
        right={
          <>
            <p className="font-medium">{props.studentName}</p>
            <p className="font-mono text-xs text-slate-600">
              {props.studentNumber}
            </p>
            <p className="mt-1 text-xs text-slate-500">
              Generated {props.generatedOn}
            </p>
          </>
        }
      />

      <section className="mt-4 text-sm">
        <p className="text-xs font-semibold uppercase tracking-wide text-slate-500">
          Term
        </p>
        <p className="mt-1">
          {props.term
            ? `Term ${props.term.termNumber}, ${props.term.year} (${props.term.startDate} to ${props.term.endDate})`
            : "All terms"}
        </p>
      </section>

      <section className="mt-6">
        <h2 className="text-xs font-semibold uppercase tracking-wide text-slate-500">
          Invoices
        </h2>
        {props.invoices.length === 0 ? (
          <p className="mt-2 text-sm text-slate-500">No invoices.</p>
        ) : (
          <table className="mt-2 w-full text-sm">
            <thead>
              <tr className="border-b border-slate-300 text-left text-xs uppercase tracking-wide text-slate-500">
                <th className="py-1.5">Invoice</th>
                <th className="py-1.5">Type</th>
                <th className="py-1.5">Due</th>
                <th className="py-1.5 text-right">Total</th>
                <th className="py-1.5 text-right">Balance</th>
              </tr>
            </thead>
            <tbody>
              {props.invoices.map((invoice) => (
                <tr key={invoice.id} className="border-b border-slate-100">
                  <td className="py-1.5 font-mono text-xs">
                    {invoice.invoiceNumber}
                  </td>
                  <td className="py-1.5">
                    {invoice.kind === "SUPPLEMENTARY" ? "Supplementary" : "Term"}
                  </td>
                  <td className="py-1.5">{invoice.dueDate}</td>
                  <td className="py-1.5 text-right">
                    {formatMoney(invoice.totalCents)}
                  </td>
                  <td className="py-1.5 text-right">
                    {formatMoney(invoice.totalCents - invoice.paidCents)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>

      <section className="mt-6">
        <h2 className="text-xs font-semibold uppercase tracking-wide text-slate-500">
          Payments
        </h2>
        {props.payments.length === 0 ? (
          <p className="mt-2 text-sm text-slate-500">No payments recorded.</p>
        ) : (
          <table className="mt-2 w-full text-sm">
            <thead>
              <tr className="border-b border-slate-300 text-left text-xs uppercase tracking-wide text-slate-500">
                <th className="py-1.5">Receipt</th>
                <th className="py-1.5">Date</th>
                <th className="py-1.5">Method</th>
                <th className="py-1.5 text-right">Amount</th>
              </tr>
            </thead>
            <tbody>
              {props.payments.map((payment) => (
                <tr key={payment.id} className="border-b border-slate-100">
                  <td className="py-1.5 font-mono text-xs">
                    {payment.receiptNumber}
                  </td>
                  <td className="py-1.5">{payment.paymentDate}</td>
                  <td className="py-1.5">
                    {payment.paymentMethod.replace(/_/g, " ").toLowerCase()}
                  </td>
                  <td className="py-1.5 text-right">
                    {formatMoney(payment.amountCents)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>

      <section className="mt-6 flex justify-end">
        <table className="w-64 text-sm">
          <tbody>
            <tr>
              <td className="py-1">Total billed</td>
              <td className="py-1 text-right font-medium">
                {formatMoney(props.totalBilledCents)}
              </td>
            </tr>
            <tr>
              <td className="py-1">Total paid</td>
              <td className="py-1 text-right font-medium">
                {formatMoney(props.totalPaidCents)}
              </td>
            </tr>
            <tr>
              <td className="py-1">Credit on account</td>
              <td className="py-1 text-right font-medium">
                {formatMoney(props.creditCents)}
              </td>
            </tr>
            <tr className="border-t border-slate-300">
              <td className="pt-2 font-semibold">Balance</td>
              <td className="pt-2 text-right text-base font-bold">
                {formatMoney(props.balanceCents)}
              </td>
            </tr>
          </tbody>
        </table>
      </section>

      <BankDetails />
      <DocumentFooter kind="invoice" />
    </div>
  )
}
