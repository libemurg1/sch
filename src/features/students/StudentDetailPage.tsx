import { useState } from "react"
import { Link, useParams } from "react-router-dom"
import {
  useClasses,
  useStudent,
  useStudentAccount,
  useStudentEnrollments,
  useStudentInvoices,
  useStudentPayments,
  useTerms,
  useUpdateStudentStatus,
} from "../../api/queries"
import {
  Button,
  Card,
  EmptyState,
  ErrorBanner,
  Loading,
  PageHeader,
  Select,
  StatusBadge,
  Table,
  type Column,
} from "../../components"
import { Page } from "../../app/AppShell"
import { formatDate, formatDateTime, formatTermLabel, studentName } from "../../lib/dates"
import { formatMoney } from "../../lib/money"
import type { Enrollment } from "../../types/schema/Enrollment"
import type { Invoice } from "../../types/schema/Invoice"
import type { Payment } from "../../types/schema/Payment"
import { StudentFormModal } from "./StudentFormModal"

const STATUSES = ["ACTIVE", "INACTIVE", "GRADUATED"]

export function StudentDetailPage() {
  const { studentId } = useParams()
  const student = useStudent(studentId)
  const account = useStudentAccount(studentId)
  const enrollments = useStudentEnrollments(studentId)
  const invoices = useStudentInvoices(studentId)
  const payments = useStudentPayments(studentId)
  const classes = useClasses()
  const terms = useTerms()
  const updateStatus = useUpdateStudentStatus()

  const [nextStatus, setNextStatus] = useState("")
  const [editing, setEditing] = useState(false)

  if (student.isLoading) {
    return (
      <Page>
        <Loading label="Loading student..." />
      </Page>
    )
  }

  if (student.isError) {
    return (
      <Page>
        <ErrorBanner error={student.error} onRetry={() => student.refetch()} />
      </Page>
    )
  }

  if (!student.data) {
    return (
      <Page>
        <EmptyState
          title="Student not found"
          description="This student does not exist any more."
          action={
            <Link to="/students" className="text-sm font-medium text-indigo-600">
              Back to students
            </Link>
          }
        />
      </Page>
    )
  }

  const record = student.data
  const classNames = new Map(
    (classes.data ?? []).map((klass) => [klass.id, klass.name]),
  )
  const termLabels = new Map(
    (terms.data ?? []).map((term) => [term.id, formatTermLabel(term)]),
  )

  const enrollmentColumns: Column<Enrollment>[] = [
    {
      key: "term",
      header: "Term",
      render: (enrollment) => termLabels.get(enrollment.termId) ?? "-",
    },
    {
      key: "class",
      header: "Class",
      render: (enrollment) => classNames.get(enrollment.classId) ?? "-",
    },
    {
      key: "enrolled",
      header: "Enrolled",
      hideOnSmall: true,
      render: (enrollment) => formatDateTime(enrollment.enrolledAt),
    },
    {
      key: "status",
      header: "Status",
      align: "right",
      render: (enrollment) => <StatusBadge status={enrollment.status} />,
    },
  ]

  const invoiceColumns: Column<Invoice>[] = [
    {
      key: "number",
      header: "Invoice",
      render: (invoice) => (
        <Link
          to={`/billing/invoices/${invoice.id}`}
          className="font-medium text-indigo-600 hover:text-indigo-800"
        >
          {invoice.invoiceNumber}
        </Link>
      ),
    },
    {
      key: "kind",
      header: "Kind",
      render: (invoice) => invoice.kind,
    },
    {
      key: "total",
      header: "Total",
      align: "right",
      render: (invoice) => formatMoney(invoice.totalCents),
    },
    {
      key: "paid",
      header: "Paid",
      align: "right",
      render: (invoice) => formatMoney(invoice.paidCents),
    },
    {
      key: "balance",
      header: "Balance",
      align: "right",
      render: (invoice) =>
        formatMoney(invoice.totalCents - invoice.paidCents),
    },
    {
      key: "status",
      header: "Status",
      align: "right",
      render: (invoice) => <StatusBadge status={invoice.status} />,
    },
  ]

  const paymentColumns: Column<Payment>[] = [
    {
      key: "receipt",
      header: "Receipt",
      render: (payment) => (
        <span className="font-mono text-xs text-slate-500">
          {payment.receiptNumber}
        </span>
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
      render: (payment) => payment.paymentMethod.replace(/_/g, " "),
    },
    {
      key: "amount",
      header: "Amount",
      align: "right",
      render: (payment) => formatMoney(payment.amountCents),
    },
  ]

  const billed = (invoices.data ?? []).reduce(
    (total, invoice) => total + invoice.totalCents,
    0,
  )
  const paid = (invoices.data ?? []).reduce(
    (total, invoice) => total + invoice.paidCents,
    0,
  )
  // Credit is money received that no invoice has consumed yet.
  const credit = account.data?.creditCents ?? 0

  return (
    <Page>
      <PageHeader
        title={studentName(record)}
        description={
          <>
            <span className="font-mono text-xs">{record.studentNumber}</span>
            {" · "}
            <StatusBadge status={record.status} />
          </>
        }
        actions={
          <>
            <Button variant="secondary" onClick={() => setEditing(true)}>
              Edit
            </Button>
            <Link
              to={`/statements?studentId=${record.id}`}
              className="rounded-md bg-indigo-600 px-3.5 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700"
            >
              Statement
            </Link>
          </>
        }
      />

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Card title="Details" className="sm:col-span-2">
          <dl className="grid grid-cols-2 gap-x-4 gap-y-3 text-sm">
            <Detail label="First name" value={record.firstName} />
            <Detail label="Last name" value={record.lastName} />
            <Detail label="Date of birth" value={formatDate(record.dob)} />
            <Detail label="Gender" value={record.gender ?? "Not stated"} />
            <Detail label="Admission date" value={formatDate(record.admissionDate)} />
            <Detail label="Registered" value={formatDateTime(record.createdAt)} />
            <Detail label="Guardian" value={record.guardianName ?? "-"} />
            <Detail label="Guardian phone" value={record.guardianPhone ?? "-"} />
            <Detail
              label="Guardian email"
              value={record.guardianEmail ?? "-"}
              className="col-span-2"
            />
          </dl>
        </Card>

        <Card title="Status" className="sm:col-span-2">
          <p className="text-sm text-slate-500">
            Students are never deleted. Moving a student to another status keeps
            their financial history intact.
          </p>
          <div className="mt-4 flex flex-wrap items-end gap-3">
            <div className="w-40">
              <label
                htmlFor="status"
                className="mb-1 block text-sm font-medium text-slate-700"
              >
                New status
              </label>
              <Select
                id="status"
                value={nextStatus || record.status}
                onChange={(event) => setNextStatus(event.target.value)}
              >
                {STATUSES.map((value) => (
                  <option key={value} value={value}>
                    {value}
                  </option>
                ))}
              </Select>
            </div>
            <Button
              variant="secondary"
              disabled={!nextStatus || nextStatus === record.status}
              loading={updateStatus.isPending}
              onClick={() =>
                updateStatus.mutate(
                  { studentId: record.id, status: nextStatus },
                  { onSuccess: () => setNextStatus("") },
                )
              }
            >
              Update status
            </Button>
          </div>
          {updateStatus.isError ? (
            <ErrorBanner error={updateStatus.error} className="mt-3" />
          ) : null}
        </Card>
      </div>

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Card title="Billed">
          <p className="text-2xl font-semibold text-slate-900">
            {formatMoney(billed)}
          </p>
        </Card>
        <Card title="Received">
          <p className="text-2xl font-semibold text-emerald-600">
            {formatMoney(paid + credit)}
          </p>
        </Card>
        <Card title="Credit on account">
          <p className="text-2xl font-semibold text-emerald-600">
            {formatMoney(credit)}
          </p>
          <p className="mt-1 text-xs text-slate-500">
            Applies to future invoices automatically.
          </p>
        </Card>
        <Card title="Balance">
          <p className="text-2xl font-semibold text-slate-900">
            {formatMoney(billed - paid - credit)}
          </p>
          <p className="mt-1 text-xs text-slate-500">
            {billed - paid - credit < 0 ? "In credit." : "Still owed."}
          </p>
        </Card>
      </div>

      <Card title="Enrollments">
        {enrollments.isLoading ? (
          <Loading />
        ) : enrollments.isError ? (
          <ErrorBanner error={enrollments.error} />
        ) : (enrollments.data ?? []).length === 0 ? (
          <EmptyState
            title="Not enrolled yet"
            description="Enrol this student from the Classes & Terms screen."
          />
        ) : (
          <Table
            columns={enrollmentColumns}
            rows={enrollments.data ?? []}
            keyOf={(enrollment) => enrollment.id}
          />
        )}
      </Card>

      <Card title="Invoices">
        {invoices.isLoading ? (
          <Loading />
        ) : invoices.isError ? (
          <ErrorBanner error={invoices.error} />
        ) : (invoices.data ?? []).length === 0 ? (
          <EmptyState title="No invoices" />
        ) : (
          <Table
            columns={invoiceColumns}
            rows={invoices.data ?? []}
            keyOf={(invoice) => invoice.id}
          />
        )}
      </Card>

      <Card title="Payments">
        {payments.isLoading ? (
          <Loading />
        ) : payments.isError ? (
          <ErrorBanner error={payments.error} />
        ) : (payments.data ?? []).length === 0 ? (
          <EmptyState title="No payments recorded" />
        ) : (
          <Table
            columns={paymentColumns}
            rows={payments.data ?? []}
            keyOf={(payment) => payment.id}
          />
        )}
      </Card>

      <StudentFormModal
        open={editing}
        onClose={() => setEditing(false)}
        student={record}
      />
    </Page>
  )
}

function Detail({
  label,
  value,
  className,
}: {
  label: string
  value: string
  className?: string
}) {
  return (
    <div className={className}>
      <dt className="text-xs font-medium uppercase tracking-wide text-slate-500">
        {label}
      </dt>
      <dd className="mt-0.5 text-slate-900">{value}</dd>
    </div>
  )
}
