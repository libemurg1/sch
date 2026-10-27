import { useEffect, useMemo, useState } from "react"
import { useAuditEntries, useFinanceReport, useAppInfo } from "../../api/queries"
import {
  Button,
  Card,
  EmptyState,
  ErrorBanner,
  Field,
  Input,
  Loading,
  PageHeader,
  Table,
  type Column,
} from "../../components"
import { Page } from "../../app/AppShell"
import { formatDateTime } from "../../lib/dates"
import { formatMoney } from "../../lib/money"
import type { AuditEntry, TermBalanceSummary } from "../../api/reports"

export function ReportsPage() {
  const appInfo = useAppInfo()
  const [endDate, setEndDate] = useState("")
  const [startDate, setStartDate] = useState("")
  const report = useFinanceReport(startDate, endDate)
  const audit = useAuditEntries()
  const today = appInfo.data?.today

  useEffect(() => {
    if (today) {
      setEndDate((current) => current || today)
      setStartDate((current) => current || `${today.slice(0, 7)}-01`)
    }
  }, [today])

  const termColumns: Column<TermBalanceSummary>[] = [
    { key: "term", header: "Term", render: (row) => row.label },
    {
      key: "count",
      header: "Invoices",
      align: "right",
      render: (row) => row.invoiceCount,
    },
    {
      key: "billed",
      header: "Billed",
      align: "right",
      render: (row) => formatMoney(row.billedCents),
    },
    {
      key: "paid",
      header: "Paid",
      align: "right",
      render: (row) => formatMoney(row.paidCents),
    },
    {
      key: "outstanding",
      header: "Outstanding",
      align: "right",
      render: (row) => formatMoney(row.outstandingCents),
    },
    {
      key: "overdue",
      header: "Overdue",
      align: "right",
      render: (row) => formatMoney(row.overdueCents),
    },
  ]

  const auditColumns: Column<AuditEntry>[] = [
    {
      key: "date",
      header: "When",
      render: (entry) => formatDateTime(entry.at),
    },
    {
      key: "actor",
      header: "User",
      render: (entry) => entry.actorName ?? "System",
    },
    {
      key: "action",
      header: "Action",
      render: (entry) => `${entry.entity} ${entry.action}`,
    },
    {
      key: "reference",
      header: "Record",
      render: (entry) => <span className="font-mono text-xs">{entry.entityId}</span>,
    },
    {
      key: "details",
      header: "Details",
      render: (entry) => (
        <span className="block max-w-sm truncate text-xs text-slate-500" title={entry.detailJson ?? undefined}>
          {entry.detailJson ?? "-"}
        </span>
      ),
    },
  ]

  const exportCsv = () => {
    if (!report.data) return
    const lines = [
      ["Report", "School fees collection and balances"],
      ["Date range", `${report.data.startDate} to ${report.data.endDate}`],
      [],
      ["Collection by payment method"],
      ["Method", "Payment count", "Amount"],
      ...report.data.byMethod.map((row) => [
        row.paymentMethod,
        String(row.paymentCount),
        (row.amountCents / 100).toFixed(2),
      ]),
      [],
      ["Balances by term"],
      ["Term", "Invoices", "Billed", "Paid", "Outstanding", "Overdue"],
      ...report.data.byTerm.map((row) => [
        row.label,
        String(row.invoiceCount),
        (row.billedCents / 100).toFixed(2),
        (row.paidCents / 100).toFixed(2),
        (row.outstandingCents / 100).toFixed(2),
        (row.overdueCents / 100).toFixed(2),
      ]),
    ]
    const csv = lines.map((row) => row.map(csvCell).join(",")).join("\r\n")
    const url = URL.createObjectURL(new Blob([csv], { type: "text/csv;charset=utf-8" }))
    const link = document.createElement("a")
    link.href = url
    link.download = `school-finance-report-${startDate}-${endDate}.csv`
    link.click()
    URL.revokeObjectURL(url)
  }

  const methodRows = useMemo(() => report.data?.byMethod ?? [], [report.data])

  return (
    <Page>
      <PageHeader
        title="Finance reports"
        description="Review collections by payment method, balances by term, overdue fees, and the audit trail."
        actions={
          <Button
            variant="secondary"
            onClick={exportCsv}
            disabled={!report.data}
          >
            Export CSV
          </Button>
        }
      />

      <Card>
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label="From" htmlFor="report-start">
            <Input
              id="report-start"
              type="date"
              value={startDate}
              onChange={(event) => setStartDate(event.target.value)}
            />
          </Field>
          <Field label="To" htmlFor="report-end">
            <Input
              id="report-end"
              type="date"
              value={endDate}
              onChange={(event) => setEndDate(event.target.value)}
            />
          </Field>
        </div>
      </Card>

      {report.isError ? <ErrorBanner error={report.error} onRetry={() => report.refetch()} /> : null}
      {report.isLoading ? (
        <Loading />
      ) : report.data ? (
        <>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <Summary label="Payments received" value={formatMoney(report.data.paymentsReceivedCents)} />
            <Summary label="Payment count" value={String(report.data.paymentCount)} />
            <Summary label="Outstanding" value={formatMoney(report.data.outstandingCents)} />
            <Summary label="Overdue" value={formatMoney(report.data.overdueCents)} />
          </div>
          <Card title="Collections by payment method">
            {methodRows.length === 0 ? (
              <EmptyState title="No payments in this date range" />
            ) : (
              <div className="divide-y divide-slate-100">
                {methodRows.map((row) => (
                  <div key={row.paymentMethod} className="flex justify-between gap-4 py-2 text-sm">
                    <span>{row.paymentMethod.replace(/_/g, " ")}</span>
                    <span className="text-right">
                      {row.paymentCount} payments · {formatMoney(row.amountCents)}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </Card>
          <Card title="Balances by term" description="Outstanding and overdue amounts include only active invoices.">
            <Table
              columns={termColumns}
              rows={report.data.byTerm}
              keyOf={(row) => row.termId}
              empty={<EmptyState title="No invoices recorded yet" />}
            />
          </Card>
        </>
      ) : null}

      <Card title="Audit trail" description="Latest 200 recorded financial and account actions.">
        {audit.isLoading ? (
          <Loading />
        ) : audit.isError ? (
          <ErrorBanner error={audit.error} onRetry={() => audit.refetch()} />
        ) : (
          <Table
            columns={auditColumns}
            rows={audit.data ?? []}
            keyOf={(entry) => String(entry.id)}
            empty={<EmptyState title="No audited actions yet" />}
          />
        )}
      </Card>
    </Page>
  )
}

function Summary({ label, value }: { label: string; value: string }) {
  return (
    <Card title={label}>
      <p className="text-2xl font-semibold text-slate-900">{value}</p>
    </Card>
  )
}

function csvCell(value: string) {
  return `"${value.replace(/"/g, '""')}"`
}
