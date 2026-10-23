import { useEffect, useState } from "react"
import { useSearchParams } from "react-router-dom"
import { useAppInfo, useStudentStatement, useStudents, useTerms } from "../../api/queries"
import {
  Button,
  Card,
  EmptyState,
  ErrorBanner,
  Field,
  Loading,
  PageHeader,
  Select,
  StudentPicker,
  Table,
  type Column,
} from "../../components"
import { Page } from "../../app/AppShell"
import { formatDate, formatTermLabel, studentName } from "../../lib/dates"
import { formatMoney } from "../../lib/money"
import { StatementDocument } from "../../reports/StatementDocument"
import { printDocument } from "../../reports/print"
import type { Invoice } from "../../types/schema/Invoice"
import type { Payment } from "../../types/schema/Payment"

export function StatementsPage() {
  const [params, setParams] = useSearchParams()
  const students = useStudents()
  const terms = useTerms()
  const appInfo = useAppInfo()

  const studentId = params.get("studentId") ?? ""
  const [termId, setTermId] = useState(params.get("termId") ?? "")

  const statement = useStudentStatement(studentId || undefined, termId || null)

  useEffect(() => {
    if (!studentId && students.data && students.data.length > 0) {
      setParams({ studentId: students.data[0].id }, { replace: true })
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [students.data, studentId])

  const selectedStudent = (students.data ?? []).find(
    (student) => student.id === studentId,
  )

  const invoiceColumns: Column<Invoice>[] = [
    {
      key: "number",
      header: "Invoice",
      render: (invoice) => (
        <span className="font-mono text-xs">{invoice.invoiceNumber}</span>
      ),
    },
    {
      key: "kind",
      header: "Type",
      render: (invoice) =>
        invoice.kind === "SUPPLEMENTARY" ? "Supplementary" : "Term",
    },
    {
      key: "due",
      header: "Due",
      render: (invoice) => formatDate(invoice.dueDate),
    },
    {
      key: "total",
      header: "Total",
      align: "right",
      render: (invoice) => formatMoney(invoice.totalCents),
    },
    {
      key: "balance",
      header: "Balance",
      align: "right",
      render: (invoice) =>
        formatMoney(invoice.totalCents - invoice.paidCents),
    },
  ]

  const paymentColumns: Column<Payment>[] = [
    {
      key: "receipt",
      header: "Receipt",
      render: (payment) => (
        <span className="font-mono text-xs">{payment.receiptNumber}</span>
      ),
    },
    {
      key: "date",
      header: "Date",
      render: (payment) => formatDate(payment.paymentDate),
    },
    {
      key: "method",
      header: "Method",
      render: (payment) => payment.paymentMethod.replace(/_/g, " ").toLowerCase(),
    },
    {
      key: "amount",
      header: "Amount",
      align: "right",
      render: (payment) => formatMoney(payment.amountCents),
    },
  ]

  return (
    <Page>
      <div className="no-print space-y-6">
        <PageHeader
          title="Statements"
          description="Everything a student has been billed and paid."
          actions={
            <Button
              onClick={() => printDocument("a4")}
              disabled={!statement.data}
            >
              Print statement
            </Button>
          }
        />

        <Card>
          <div className="grid gap-4 sm:grid-cols-2">
            <Field label="Student" htmlFor="statement-student">
              <StudentPicker
                id="statement-student"
                value={studentId}
                onChange={(nextStudentId) => setParams({ studentId: nextStudentId })}
                students={students.data ?? []}
              />
            </Field>
            <Field
              label="Term"
              htmlFor="statement-term"
              hint="Leave blank to include every term."
            >
              <Select
                id="statement-term"
                value={termId}
                onChange={(event) => setTermId(event.target.value)}
              >
                <option value="">All terms</option>
                {(terms.data ?? []).map((term) => (
                  <option key={term.id} value={term.id}>
                    {formatTermLabel(term)}
                  </option>
                ))}
              </Select>
            </Field>
          </div>
        </Card>

        {!studentId ? (
          <Card>
            <EmptyState title="Choose a student" />
          </Card>
        ) : statement.isLoading ? (
          <Loading />
        ) : statement.isError ? (
          <ErrorBanner error={statement.error} onRetry={() => statement.refetch()} />
        ) : statement.data ? (
          <>
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <Card title="Total billed">
                <p className="text-2xl font-semibold text-slate-900">
                  {formatMoney(statement.data.totalBilledCents)}
                </p>
              </Card>
              <Card title="Total paid">
                <p className="text-2xl font-semibold text-emerald-600">
                  {formatMoney(statement.data.totalPaidCents)}
                </p>
              </Card>
              <Card title="Credit on account">
                <p className="text-2xl font-semibold text-emerald-600">
                  {formatMoney(statement.data.creditCents)}
                </p>
                <p className="mt-1 text-xs text-slate-500">
                  Not yet applied to an invoice.
                </p>
              </Card>
              <Card title="Balance">
                <p className="text-2xl font-semibold text-slate-900">
                  {formatMoney(statement.data.balanceCents)}
                </p>
                <p className="mt-1 text-xs text-slate-500">
                  {statement.data.balanceCents < 0
                    ? "The student is in credit."
                    : "Still owed."}
                </p>
              </Card>
            </div>

            <Card title="Invoices">
              {statement.data.invoices.length === 0 ? (
                <EmptyState title="No invoices" />
              ) : (
                <Table
                  columns={invoiceColumns}
                  rows={statement.data.invoices}
                  keyOf={(invoice) => invoice.id}
                />
              )}
            </Card>

            <Card title="Payments">
              {statement.data.payments.length === 0 ? (
                <EmptyState title="No payments" />
              ) : (
                <Table
                  columns={paymentColumns}
                  rows={statement.data.payments}
                  keyOf={(payment) => payment.id}
                />
              )}
            </Card>
          </>
        ) : null}
      </div>

      {statement.data && studentId ? (
        <div className="print-only hidden">
          <StatementDocument
            studentName={selectedStudent ? studentName(selectedStudent) : "-"}
            studentNumber={selectedStudent?.studentNumber ?? "-"}
            term={statement.data.term}
            invoices={statement.data.invoices}
            payments={statement.data.payments}
            totalBilledCents={statement.data.totalBilledCents}
            totalPaidCents={statement.data.totalPaidCents}
            creditCents={statement.data.creditCents}
            balanceCents={statement.data.balanceCents}
            generatedOn={appInfo.data?.today ?? ""}
          />
        </div>
      ) : null}
    </Page>
  )
}
