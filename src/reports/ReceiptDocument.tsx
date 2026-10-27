import type { Payment } from "../types/schema/Payment"
import type { PaymentAllocation } from "../types/schema/PaymentAllocation"
import { formatMoney } from "../lib/money"
import { DocumentFooter, SchoolLetterhead } from "./SchoolBlocks"

export interface ReceiptDocumentProps {
  payment: Payment
  allocations: PaymentAllocation[]
  studentName: string
  studentNumber: string
  /** Invoice numbers, keyed by invoice id, so the slip can name them. */
  invoiceNumbers: Record<string, string>
  /** The signed-in user who recorded the payment. */
  receivedByName?: string | null
}

/**
 * A receipt for an 80 mm thermal roll. The layout is deliberately narrow and
 * single column, because that is all a thermal printer can lay out.
 */
export function ReceiptDocument(props: ReceiptDocumentProps) {
  const activeAllocations = props.allocations.filter(
    (allocation) => !allocation.reversedAt,
  )
  const allocatedCents = activeAllocations.reduce(
    (total, allocation) => total + allocation.amountCents,
    0,
  )
  const creditCents = props.payment.reversedAt
    ? 0
    : props.payment.amountCents - allocatedCents

  return (
    <div
      className="mx-auto bg-white p-3 text-slate-900"
      style={{ width: "76mm" }}
    >
      <SchoolLetterhead subtitle="Payment receipt" compact />

      <div className="mt-3 border-t border-dashed border-slate-400 pt-2 text-[11px] leading-snug">
        <p className="font-mono text-center text-sm font-bold">
          {props.payment.receiptNumber}
        </p>
        <div className="mt-1.5 space-y-0.5">
          <Row label="Date" value={props.payment.paymentDate} />
          <Row label="Student" value={props.studentName} />
          <Row label="Number" value={props.studentNumber} />
          <Row
            label="Method"
            value={props.payment.paymentMethod.replace(/_/g, " ").toLowerCase()}
          />
          {props.payment.referenceNumber ? (
            <Row label="Reference" value={props.payment.referenceNumber} />
          ) : null}
          {props.receivedByName ? (
            <Row label="Received by" value={props.receivedByName} />
          ) : null}
        </div>
      </div>

      {props.payment.reversedAt ? (
        <div className="mt-3 border border-rose-300 bg-rose-50 p-2 text-center text-[11px] font-bold text-rose-800">
          REVERSED {props.payment.reversedAt.slice(0, 10)}
          {props.payment.reversalReason
            ? ` - ${props.payment.reversalReason}`
            : ""}
        </div>
      ) : null}

      {props.allocations.length > 0 ? (
        <div className="mt-3 border-t border-dashed border-slate-400 pt-2">
          <p className="text-[10px] font-semibold uppercase tracking-wide">
            Applied to
          </p>
          <div className="mt-1 space-y-0.5 text-[11px]">
            {props.allocations.map((allocation) => (
              <div
                key={allocation.id}
                className="flex items-baseline justify-between gap-2"
              >
                <span className="font-mono">
                  {props.invoiceNumbers[allocation.invoiceId] ?? allocation.invoiceId}
                </span>
                <span
                  className={
                    allocation.reversedAt
                      ? "text-rose-700 line-through"
                      : "font-medium"
                  }
                >
                  {formatMoney(allocation.amountCents)}
                </span>
              </div>
            ))}
          </div>
        </div>
      ) : null}

      {creditCents > 0 ? (
        <div className="mt-3 border-t border-dashed border-slate-400 pt-2">
          <div className="flex items-baseline justify-between text-[11px]">
            <span className="text-slate-600">Credit carried forward</span>
            <span className="font-medium">{formatMoney(creditCents)}</span>
          </div>
        </div>
      ) : null}

      <div className="mt-3 border-t border-slate-400 pt-2">
        <div className="flex items-baseline justify-between text-sm">
          <span className="font-semibold">
            {props.payment.reversedAt ? "ORIGINAL AMOUNT" : "TOTAL PAID"}
          </span>
          <span className="font-bold">{formatMoney(props.payment.amountCents)}</span>
        </div>
      </div>

      <p className="mt-3 text-center text-[10px] text-slate-600">
        Thank you. Keep this receipt safe.
      </p>
      <DocumentFooter kind="receipt" />
    </div>
  )
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-baseline justify-between gap-2">
      <span className="text-slate-600">{label}</span>
      <span className="text-right">{value}</span>
    </div>
  )
}
