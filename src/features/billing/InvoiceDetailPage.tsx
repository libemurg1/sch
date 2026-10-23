import { Link, useParams } from "react-router-dom"
import { useInvoiceDetail, useStudents } from "../../api/queries"
import {
  Button,
  Card,
  EmptyState,
  ErrorBanner,
  Loading,
  PageHeader,
  StatusBadge,
  Table,
  type Column,
} from "../../components"
import { Page } from "../../app/AppShell"
import { formatDate, studentName } from "../../lib/dates"
import { formatMoney } from "../../lib/money"
import { InvoiceDocument } from "../../reports/InvoiceDocument"
import { printDocument } from "../../reports/print"
import type { InvoiceItem } from "../../types/schema/InvoiceItem"

export function InvoiceDetailPage() {
  const { invoiceId } = useParams()
  const detail = useInvoiceDetail(invoiceId)
  const students = useStudents()

  if (detail.isLoading) {
    return (
      <Page>
        <Loading label="Loading invoice..." />
      </Page>
    )
  }

  if (detail.isError) {
    return (
      <Page>
        <ErrorBanner error={detail.error} onRetry={() => detail.refetch()} />
      </Page>
    )
  }

  if (!detail.data) {
    return (
      <Page>
        <EmptyState
          title="Invoice not found"
          action={
            <Link to="/billing" className="text-sm font-medium text-indigo-600">
              Back to billing
            </Link>
          }
        />
      </Page>
    )
  }

  const invoice = detail.data.invoice
  const items = detail.data.items
  const student = (students.data ?? []).find(
    (candidate) => candidate.id === invoice.studentId,
  )

  const columns: Column<InvoiceItem>[] = [
    {
      key: "description",
      header: "Description",
      render: (item) => (
        <span className={item.kind === "DISCOUNT" ? "text-emerald-700" : undefined}>
          {item.kind === "DISCOUNT" ? `Discount: ${item.description}` : item.description}
        </span>
      ),
    },
    {
      key: "kind",
      header: "Kind",
      hideOnSmall: true,
      render: (item) => item.kind,
    },
    {
      key: "amount",
      header: "Amount",
      align: "right",
      render: (item) => (
        <span className={item.kind === "DISCOUNT" ? "text-emerald-700" : undefined}>
          {formatMoney(item.amountCents)}
        </span>
      ),
    },
  ]

  return (
    <Page>
      <div className="no-print space-y-6">
        <PageHeader
          title={invoice.invoiceNumber}
          description={
            <>
              <StatusBadge status={invoice.kind} />{" "}
              <StatusBadge status={invoice.status} />
            </>
          }
          actions={
            <>
              <Link
                to="/billing"
                className="rounded-md bg-white px-3.5 py-2 text-sm font-medium text-slate-700 shadow-sm ring-1 ring-inset ring-slate-300 hover:bg-slate-50"
              >
                Back
              </Link>
              <Button onClick={() => printDocument("a4")}>Print invoice</Button>
            </>
          }
        />

        <Card>
          <dl className="grid grid-cols-2 gap-x-4 gap-y-3 text-sm sm:grid-cols-4">
            <Field label="Student">
              {student ? (
                <Link
                  to={`/students/${student.id}`}
                  className="text-indigo-600 hover:text-indigo-800"
                >
                  {studentName(student)}
                </Link>
              ) : (
                "-"
              )}
            </Field>
            <Field label="Issued">{formatDate(invoice.issueDate)}</Field>
            <Field label="Due">{formatDate(invoice.dueDate)}</Field>
            <Field label="Balance">
              {formatMoney(invoice.totalCents - invoice.paidCents)}
            </Field>
          </dl>

          <div className="mt-4">
            <Table columns={columns} rows={items} keyOf={(item) => item.id} />
          </div>

          <dl className="mt-4 ml-auto w-full max-w-xs space-y-1 text-sm">
            <SummaryRow label="Subtotal" value={formatMoney(invoice.subtotalCents)} />
            <SummaryRow
              label="Discount"
              value={`-${formatMoney(invoice.discountCents)}`}
              tone="positive"
            />
            <SummaryRow
              label="Total"
              value={formatMoney(invoice.totalCents)}
              emphasis
            />
            <SummaryRow label="Paid" value={formatMoney(invoice.paidCents)} />
            <SummaryRow
              label="Balance"
              value={formatMoney(invoice.totalCents - invoice.paidCents)}
              emphasis
            />
          </dl>
        </Card>
      </div>

      <div className="print-only hidden">
        <InvoiceDocument
          invoiceNumber={invoice.invoiceNumber}
          kind={invoice.kind}
          studentName={student ? studentName(student) : "-"}
          studentNumber={student?.studentNumber ?? "-"}
          issueDate={invoice.issueDate}
          dueDate={invoice.dueDate}
          items={items}
          subtotalCents={invoice.subtotalCents}
          discountCents={invoice.discountCents}
          totalCents={invoice.totalCents}
          paidCents={invoice.paidCents}
          status={invoice.status}
        />
      </div>
    </Page>
  )
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div>
      <dt className="text-xs font-medium uppercase tracking-wide text-slate-500">
        {label}
      </dt>
      <dd className="mt-0.5 text-slate-900">{children}</dd>
    </div>
  )
}

function SummaryRow({
  label,
  value,
  emphasis,
  tone,
}: {
  label: string
  value: string
  emphasis?: boolean
  tone?: "positive"
}) {
  return (
    <div className="flex items-baseline justify-between gap-4">
      <dt className={emphasis ? "font-semibold text-slate-900" : "text-slate-600"}>
        {label}
      </dt>
      <dd
        className={[
          emphasis ? "text-base font-bold" : "font-medium",
          tone === "positive" ? "text-emerald-700" : "text-slate-900",
        ].join(" ")}
      >
        {value}
      </dd>
    </div>
  )
}
