import { Link } from "react-router-dom"
import { useActiveTerm, useAppInfo, useInvoices, useStudents } from "../../api/queries"
import {
  Card,
  EmptyState,
  ErrorBanner,
  Loading,
  PageHeader,
  StatTile,
  StatusBadge,
  Table,
  type Column,
} from "../../components"
import { Page } from "../../app/AppShell"
import { formatMoney } from "../../lib/money"
import { formatDate, formatTermLabel, studentName } from "../../lib/dates"
import type { Invoice } from "../../types/schema/Invoice"

/** The landing screen: where the school stands right now. */
export function DashboardPage() {
  const appInfo = useAppInfo()
  const activeTerm = useActiveTerm()
  const invoices = useInvoices()
  const students = useStudents()

  const studentNames = new Map(
    (students.data ?? []).map((student) => [student.id, studentName(student)]),
  )

  const recent = [...(invoices.data ?? [])]
    .sort((a, b) => b.issueDate.localeCompare(a.issueDate))
    .slice(0, 8)

  const outstanding = (invoices.data ?? []).reduce(
    (total, invoice) => total + (invoice.totalCents - invoice.paidCents),
    0,
  )

  const columns: Column<Invoice>[] = [
    {
      key: "number",
      header: "Invoice",
      render: (invoice) => (
        <Link
          to={`/billing/invoices/${invoice.id}`}
          className="font-medium text-indigo-600 hover:text-indigo-800"
          onClick={(event) => event.stopPropagation()}
        >
          {invoice.invoiceNumber}
        </Link>
      ),
    },
    {
      key: "student",
      header: "Student",
      render: (invoice) => studentNames.get(invoice.studentId) ?? "-",
    },
    {
      key: "kind",
      header: "Kind",
      render: (invoice) => (
        <StatusBadge status={invoice.kind} />
      ),
    },
    {
      key: "due",
      header: "Due",
      hideOnSmall: true,
      render: (invoice) => formatDate(invoice.dueDate),
    },
    {
      key: "total",
      header: "Total",
      align: "right",
      render: (invoice) => formatMoney(invoice.totalCents),
    },
    {
      key: "status",
      header: "Status",
      align: "right",
      render: (invoice) => <StatusBadge status={invoice.status} />,
    },
  ]

  if (appInfo.isError) {
    return (
      <Page>
        <PageHeader title="Dashboard" />
        <ErrorBanner error={appInfo.error} onRetry={() => appInfo.refetch()} />
      </Page>
    )
  }

  return (
    <Page>
      <PageHeader
        title="Dashboard"
        description={
          activeTerm.data
            ? `Active term: ${formatTermLabel(activeTerm.data)}`
            : "No term is currently active."
        }
        actions={
          <>
            <Link
              to="/students"
              className="rounded-md bg-white px-3.5 py-2 text-sm font-medium text-slate-700 shadow-sm ring-1 ring-inset ring-slate-300 hover:bg-slate-50"
            >
              Students
            </Link>
            <Link
              to="/billing"
              className="rounded-md bg-indigo-600 px-3.5 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700"
            >
              Billing
            </Link>
          </>
        }
      />

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatTile
          label="Students"
          value={appInfo.data ? appInfo.data.studentCount : "-"}
          hint="Registered in the database"
        />
        <StatTile
          label="Invoices"
          value={appInfo.data ? appInfo.data.invoiceCount : "-"}
          hint="Term and supplementary"
        />
        <StatTile
          label="Outstanding"
          value={formatMoney(outstanding)}
          tone={outstanding > 0 ? "negative" : "positive"}
          hint="Total still to be collected"
        />
        <StatTile
          label="Active term"
          value={activeTerm.data ? `Term ${activeTerm.data.termNumber}` : "-"}
          hint={activeTerm.data ? String(activeTerm.data.year) : "Set one in Classes & Terms"}
        />
      </div>

      <Card
        title="Recent invoices"
        description="The eight most recently issued invoices."
        actions={
          <Link
            to="/billing"
            className="text-sm font-medium text-indigo-600 hover:text-indigo-800"
          >
            View all
          </Link>
        }
      >
        {invoices.isLoading ? (
          <Loading />
        ) : invoices.isError ? (
          <ErrorBanner error={invoices.error} onRetry={() => invoices.refetch()} />
        ) : recent.length === 0 ? (
          <EmptyState
            title="No invoices yet"
            description="Invoices appear here once a class has been billed for a term."
          />
        ) : (
          <Table
            columns={columns}
            rows={recent}
            keyOf={(invoice) => invoice.id}
          />
        )}
      </Card>
    </Page>
  )
}
