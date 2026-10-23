import { useEffect, useState } from "react"
import { Link } from "react-router-dom"
import {
  useActiveTerm,
  useAppInfo,
  useClasses,
  useGenerateInvoiceBatch,
  useGenerateSupplementaryInvoice,
  useAssignStudentFee,
  useInvoices,
  useStudents,
  useTerms,
} from "../../api/queries"
import {
  Button,
  Card,
  EmptyState,
  ErrorBanner,
  Field,
  Input,
  Loading,
  PageHeader,
  Select,
  StatusBadge,
  Table,
  type Column,
} from "../../components"
import { Page } from "../../app/AppShell"
import { formatDate, formatTermLabel, studentName } from "../../lib/dates"
import { formatMoney, parseMoneyToCents } from "../../lib/money"
import type { Invoice } from "../../types/schema/Invoice"

export function BillingPage() {
  const classes = useClasses()
  const terms = useTerms()
  const activeTerm = useActiveTerm()
  const appInfo = useAppInfo()
  const students = useStudents()
  const invoices = useInvoices()

  const today = appInfo.data?.today ?? ""

  const [classId, setClassId] = useState("")
  const [termId, setTermId] = useState("")
  const [issueDate, setIssueDate] = useState(today)
  const [dueDate, setDueDate] = useState(today)

  const [supplementStudentId, setSupplementStudentId] = useState("")
  const [supplementTermId, setSupplementTermId] = useState("")

  const [feeStudentId, setFeeStudentId] = useState("")
  const [feeTermId, setFeeTermId] = useState("")
  const [feeDescription, setFeeDescription] = useState("")
  const [feeAmountText, setFeeAmountText] = useState("")
  const [feeDiscountable, setFeeDiscountable] = useState(true)
  const [feeError, setFeeError] = useState<string | null>(null)

  const batch = useGenerateInvoiceBatch()
  const supplementary = useGenerateSupplementaryInvoice()
  const studentFee = useAssignStudentFee()

  useEffect(() => {
    if (!termId && activeTerm.data) {
      setTermId(activeTerm.data.id)
    }
  }, [activeTerm.data, termId])

  useEffect(() => {
    if (!supplementTermId && activeTerm.data) {
      setSupplementTermId(activeTerm.data.id)
    }
  }, [activeTerm.data, supplementTermId])

  useEffect(() => {
    if (!feeTermId && activeTerm.data) {
      setFeeTermId(activeTerm.data.id)
    }
  }, [activeTerm.data, feeTermId])

  useEffect(() => {
    if (!classId && classes.data && classes.data.length > 0) {
      setClassId(classes.data[0].id)
    }
  }, [classes.data, classId])

  const studentNames = new Map(
    (students.data ?? []).map((student) => [student.id, studentName(student)]),
  )

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
      render: (invoice) => studentNames.get(invoice.studentId) ?? "-",
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

  const runBatch = () => {
    batch.mutate(
      { classId, termId, issueDate, dueDate },
      { onSuccess: () => batch.reset() },
    )
  }

  const runSupplementary = () => {
    supplementary.mutate(
      {
        studentId: supplementStudentId,
        termId: supplementTermId,
        issueDate,
        dueDate,
      },
      { onSuccess: () => supplementary.reset() },
    )
  }

  const runStudentFee = () => {
    const cents = parseMoneyToCents(feeAmountText)
    if (feeDescription.trim() === "") {
      setFeeError("Description is required")
      return
    }
    if (cents === null || cents <= 0) {
      setFeeError("Enter an amount greater than zero")
      return
    }
    setFeeError(null)
    studentFee.mutate(
      {
        studentId: feeStudentId,
        termId: feeTermId,
        description: feeDescription.trim(),
        amountCents: cents,
        discountable: feeDiscountable,
      },
      {
        onSuccess: () => {
          setFeeDescription("")
          setFeeAmountText("")
          studentFee.reset()
        },
      },
    )
  }

  const createdCount = Array.isArray(batch.data) ? batch.data.length : 0

  return (
    <Page>
      <PageHeader
        title="Billing"
        description="Raise term invoices for a class, supplementary invoices for late fees, and ad-hoc charges."
      />

      <div className="grid gap-4 lg:grid-cols-2">
        <Card
          title="Bill a class"
          description="Creates one TERM invoice for every student enrolled in the class who does not already have one for the term."
        >
          <div className="space-y-4">
            {batch.isError ? <ErrorBanner error={batch.error} /> : null}
            {batch.isSuccess ? (
              <div className="rounded-md bg-emerald-50 px-4 py-3 text-sm text-emerald-800 ring-1 ring-inset ring-emerald-200">
                {createdCount === 0
                  ? "Nothing to bill. Every enrolled student already has an invoice for this term."
                  : `${createdCount} invoice${createdCount === 1 ? "" : "s"} created.`}
              </div>
            ) : null}

            <div className="grid gap-4 sm:grid-cols-2">
              <Field label="Class" htmlFor="bill-class" required>
                <Select
                  id="bill-class"
                  value={classId}
                  onChange={(event) => setClassId(event.target.value)}
                >
                  <option value="">Choose a class</option>
                  {(classes.data ?? []).map((klass) => (
                    <option key={klass.id} value={klass.id}>
                      {klass.name}
                    </option>
                  ))}
                </Select>
              </Field>
              <Field label="Term" htmlFor="bill-term" required>
                <Select
                  id="bill-term"
                  value={termId}
                  onChange={(event) => setTermId(event.target.value)}
                >
                  <option value="">Choose a term</option>
                  {(terms.data ?? []).map((term) => (
                    <option key={term.id} value={term.id}>
                      {formatTermLabel(term)}
                    </option>
                  ))}
                </Select>
              </Field>
              <Field label="Issue date" htmlFor="bill-issue" required>
                <Input
                  id="bill-issue"
                  type="date"
                  value={issueDate}
                  onChange={(event) => setIssueDate(event.target.value)}
                />
              </Field>
              <Field label="Due date" htmlFor="bill-due" required>
                <Input
                  id="bill-due"
                  type="date"
                  value={dueDate}
                  onChange={(event) => setDueDate(event.target.value)}
                />
              </Field>
            </div>

            <Button
              onClick={runBatch}
              loading={batch.isPending}
              disabled={!classId || !termId}
            >
              Raise invoices
            </Button>
          </div>
        </Card>

        <Card
          title="Supplementary invoice"
          description="Bills a student's pending ad-hoc charges onto their own invoice."
        >
          <div className="space-y-4">
            {supplementary.isError ? (
              <ErrorBanner error={supplementary.error} />
            ) : null}
            {supplementary.isSuccess ? (
              <div className="rounded-md bg-emerald-50 px-4 py-3 text-sm text-emerald-800 ring-1 ring-inset ring-emerald-200">
                {supplementary.data
                  ? `Supplementary invoice ${supplementary.data.invoiceNumber} created.`
                  : "That student has no pending charges to bill."}
              </div>
            ) : null}

            <Field label="Student" htmlFor="supp-student" required>
              <Select
                id="supp-student"
                value={supplementStudentId}
                onChange={(event) => setSupplementStudentId(event.target.value)}
              >
                <option value="">Choose a student</option>
                {(students.data ?? []).map((student) => (
                  <option key={student.id} value={student.id}>
                    {student.studentNumber} - {studentName(student)}
                  </option>
                ))}
              </Select>
            </Field>

            <Field label="Term" htmlFor="supp-term" required>
              <Select
                id="supp-term"
                value={supplementTermId}
                onChange={(event) => setSupplementTermId(event.target.value)}
              >
                <option value="">Choose a term</option>
                {(terms.data ?? []).map((term) => (
                  <option key={term.id} value={term.id}>
                    {formatTermLabel(term)}
                  </option>
                  ))}
              </Select>
            </Field>

            <Button
              variant="secondary"
              onClick={runSupplementary}
              loading={supplementary.isPending}
              disabled={!supplementStudentId || !supplementTermId}
            >
              Raise supplementary invoice
            </Button>
          </div>
        </Card>
      </div>

      <Card
        title="Ad-hoc student fee"
        description="Records a one-off charge, such as a late fee. It stays pending until it is billed on a supplementary invoice."
      >
        <div className="space-y-4">
          {studentFee.isError ? <ErrorBanner error={studentFee.error} /> : null}
          {studentFee.isSuccess ? (
            <div className="rounded-md bg-emerald-50 px-4 py-3 text-sm text-emerald-800 ring-1 ring-inset ring-emerald-200">
              The charge has been recorded. Bill it on a supplementary invoice.
            </div>
          ) : null}

          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <Field label="Student" htmlFor="fee-student" required>
              <Select
                id="fee-student"
                value={feeStudentId}
                onChange={(event) => setFeeStudentId(event.target.value)}
              >
                <option value="">Choose a student</option>
                {(students.data ?? []).map((student) => (
                  <option key={student.id} value={student.id}>
                    {student.studentNumber} - {studentName(student)}
                  </option>
                ))}
              </Select>
            </Field>
            <Field label="Term" htmlFor="fee-term" required>
              <Select
                id="fee-term"
                value={feeTermId}
                onChange={(event) => setFeeTermId(event.target.value)}
              >
                <option value="">Choose a term</option>
                {(terms.data ?? []).map((term) => (
                  <option key={term.id} value={term.id}>
                    {formatTermLabel(term)}
                  </option>
                ))}
              </Select>
            </Field>
            <Field label="Description" htmlFor="fee-description" required>
              <Input
                id="fee-description"
                value={feeDescription}
                placeholder="Late registration fee"
                onChange={(event) => {
                  setFeeDescription(event.target.value)
                  setFeeError(null)
                }}
              />
            </Field>
            <Field label="Amount" htmlFor="fee-amount" required error={feeError}>
              <Input
                id="fee-amount"
                inputMode="decimal"
                value={feeAmountText}
                placeholder="0.00"
                onChange={(event) => {
                  setFeeAmountText(event.target.value)
                  setFeeError(null)
                }}
                error={feeError}
              />
            </Field>
          </div>

          <div className="flex flex-wrap items-center gap-4">
            <label className="flex items-center gap-2 text-sm text-slate-700">
              <input
                type="checkbox"
                checked={feeDiscountable}
                onChange={(event) => setFeeDiscountable(event.target.checked)}
                className="h-4 w-4 rounded border-slate-300 text-indigo-600 focus:ring-indigo-600"
              />
              Percentage discounts can apply to this charge
            </label>
            <Button
              variant="secondary"
              onClick={runStudentFee}
              loading={studentFee.isPending}
              disabled={!feeStudentId || !feeTermId}
            >
              Record charge
            </Button>
          </div>
        </div>
      </Card>

      <Card title="All invoices">
        {invoices.isLoading ? (
          <Loading />
        ) : invoices.isError ? (
          <ErrorBanner error={invoices.error} onRetry={() => invoices.refetch()} />
        ) : (invoices.data ?? []).length === 0 ? (
          <EmptyState
            title="No invoices yet"
            description="Bill a class to raise the first invoices."
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
