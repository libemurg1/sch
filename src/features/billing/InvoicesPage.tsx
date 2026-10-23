import { Link } from "react-router-dom"
import { useInvoices } from "../../api/queries"
import {
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
import { formatDate } from "../../lib/dates"
import { formatMoney } from "../../lib/money"
import type { Invoice } from "../../types/schema/Invoice"

export function InvoicesPage() {
  const invoices = useInvoices()

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
      key: "student",
      header: "Student",
      render: (invoice) => invoice.studentId,
    },
    {
      key: "kind",
      header: "Kind",
      render: (invoice) => invoice.kind,
    },
    {
      key: "issued",
      header: "Issued",
      hideOnSmall: true,
      render: (invoice) => formatDate(invoice.issueDate),
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
      key: "balance",
      header: "Balance",
      align: "right",
      render: (invoice) => formatMoney(invoice.totalCents - invoice.paidCents),
    },
    {
      key: "status",
      header: "Status",
      align: "right",
      render: (invoice) => <StatusBadge status={invoice.status} />,
    },
  ]

  return (
    <Page>
      <PageHeader
        title="Invoices"
        description="Review issued invoices, balances, and payment status across the school."
      />

      <Card title="All invoices">
        {invoices.isLoading ? (
          <Loading />
        ) : invoices.isError ? (
          <ErrorBanner error={invoices.error} onRetry={() => invoices.refetch()} />
        ) : (invoices.data ?? []).length === 0 ? (
          <EmptyState
            title="No invoices yet"
            description="Invoices appear here once a class or student has been billed."
          />
        ) : (
          <Table
            columns={invoiceColumns}
            rows={invoices.data ?? []}
            keyOf={(invoice) => invoice.id}
          />
        )}
      </Card>
    </Page>
  )
}
