import { useEffect, useMemo, useState } from "react"
import { Link } from "react-router-dom"
import {
  useAppInfo,
  useInvoices,
  usePaymentAllocations,
  usePayments,
  useRecordPayment,
  useStudentAccount,
  useStudents,
  useUsers,
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
import { useSession } from "../../app/session"
import { formatDate, studentName } from "../../lib/dates"
import { formatMoney, parseMoneyToCents } from "../../lib/money"
import { ReceiptDocument } from "../../reports/ReceiptDocument"
import { printDocument } from "../../reports/print"
import type { Payment } from "../../types/schema/Payment"
import type { RecordPaymentInput } from "../../types/schema/RecordPaymentInput"

const METHODS = ["CASH", "MOBILE_MONEY", "BANK_TRANSFER", "CHEQUE"]

export function PaymentsPage() {
  const appInfo = useAppInfo()
  const students = useStudents()
  const invoices = useInvoices()
  const payments = usePayments()
  const users = useUsers()
  const mutation = useRecordPayment()
  const { user } = useSession()

  const [studentId, setStudentId] = useState("")
  const [amount, setAmount] = useState("")
  const [paymentDate, setPaymentDate] = useState(appInfo.data?.today ?? "")
  const [method, setMethod] = useState("CASH")
  const [reference, setReference] = useState("")
  const [preview, setPreview] = useState<Payment | null>(null)

  const account = useStudentAccount(studentId || undefined)

  const outstanding = useMemo(() => {
    return (invoices.data ?? [])
      .filter(
        (invoice) =>
          invoice.studentId === studentId &&
          invoice.status !== "CANCELLED" &&
          invoice.totalCents > invoice.paidCents,
      )
      .sort((a, b) => a.dueDate.localeCompare(b.dueDate))
  }, [invoices.data, studentId])

  const outstandingCents = outstanding.reduce(
    (total, invoice) => total + invoice.totalCents - invoice.paidCents,
    0,
  )

  const amountCents = parseMoneyToCents(amount) ?? 0
  const creditCents = account.data?.creditCents ?? 0
  const availableCents = creditCents + amountCents
  const appliedCents = Math.min(availableCents, outstandingCents)
  const creditAfterCents = availableCents - appliedCents
  const outstandingAfterCents = outstandingCents - appliedCents

  const receiptAllocations = usePaymentAllocations(mutation.data?.id)
  const previewAllocations = usePaymentAllocations(preview?.id)

  const invoiceNumbers = useMemo(() => {
    const map: Record<string, string> = {}
    for (const invoice of invoices.data ?? []) {
      map[invoice.id] = invoice.invoiceNumber
    }
    return map
  }, [invoices.data])

  const studentNames = useMemo(() => {
    const map: Record<string, string> = {}
    for (const student of students.data ?? []) {
      map[student.id] = studentName(student)
    }
    return map
  }, [students.data])

  const selectedStudent = (students.data ?? []).find(
    (student) => student.id === studentId,
  )

  const previewStudent = (students.data ?? []).find(
    (student) => student.id === preview?.studentId,
  )

  const previewReceivedBy =
    (users.data ?? []).find((entry) => entry.id === preview?.createdBy)
      ?.displayName ?? null

  const openPreview = (payment: Payment) => {
    mutation.reset()
    setPreview(payment)
  }

  const historyColumns: Column<Payment>[] = [
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
      key: "student",
      header: "Student",
      render: (payment) => studentNames[payment.studentId] ?? "-",
    },
    {
      key: "method",
      header: "Method",
      hideOnSmall: true,
      render: (payment) => payment.paymentMethod.replace(/_/g, " "),
    },
    {
      key: "amount",
      header: "Amount",
      align: "right",
      render: (payment) => (
        <span className="font-medium text-slate-900">
          {formatMoney(payment.amountCents)}
        </span>
      ),
    },
    {
      key: "actions",
      header: "",
      align: "right",
      render: (payment) => (
        <Button
          size="sm"
          variant="secondary"
          onClick={() => openPreview(payment)}
        >
          Print receipt
        </Button>
      ),
    },
  ]

  useEffect(() => {
    setAmount("")
  }, [studentId])

  useEffect(() => {
    if (appInfo.data?.today && paymentDate === "") {
      setPaymentDate(appInfo.data.today)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [appInfo.data?.today])

  const submit = () => {
    const input: RecordPaymentInput = {
      studentId,
      amountCents,
      paymentDate,
      paymentMethod: method,
      referenceNumber: reference.trim() === "" ? null : reference.trim(),
      createdBy: user?.id ?? null,
    }
    mutation.mutate(input, {
      onSuccess: () => {
        setAmount("")
        setReference("")
      },
    })
  }

  const canSubmit =
    studentId !== "" && amountCents > 0 && paymentDate !== ""

  return (
    <Page>
      <div className="no-print">
        <PageHeader
          title="Payments"
          description="Record money received. It clears the student's oldest unpaid invoices first; anything left stays on their account as credit for later terms."
        />
      </div>

      {mutation.isSuccess && mutation.data ? (
        <Card className="print:shadow-none print:ring-0">
          <div className="no-print mb-4 flex flex-wrap items-start justify-between gap-3">
            <div>
              <h2 className="text-sm font-semibold text-slate-900">
                Receipt {mutation.data.receiptNumber}
              </h2>
              <p className="mt-0.5 text-xs text-slate-500">
                The payment has been recorded.
              </p>
            </div>
            <Button onClick={() => printDocument("receipt")}>
              Print receipt
            </Button>
          </div>
          {receiptAllocations.isLoading ? (
            <Loading />
          ) : (
            <ReceiptDocument
              payment={mutation.data}
              allocations={receiptAllocations.data ?? []}
              studentName={selectedStudent ? studentName(selectedStudent) : "-"}
              studentNumber={selectedStudent?.studentNumber ?? "-"}
              invoiceNumbers={invoiceNumbers}
              receivedByName={user?.displayName ?? null}
            />
          )}
        </Card>
      ) : null}

      <div className="no-print">
        <Card title="Record a payment">
          <div className="space-y-5">
            {mutation.isError ? <ErrorBanner error={mutation.error} /> : null}

            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <Field label="Student" htmlFor="pay-student" required>
                <Select
                  id="pay-student"
                  value={studentId}
                  onChange={(event) => setStudentId(event.target.value)}
                >
                  <option value="">Choose a student</option>
                  {(students.data ?? []).map((student) => (
                    <option key={student.id} value={student.id}>
                      {student.studentNumber} - {studentName(student)}
                    </option>
                  ))}
                </Select>
              </Field>

              <Field label="Payment date" htmlFor="pay-date" required>
                <Input
                  id="pay-date"
                  type="date"
                  value={paymentDate}
                  onChange={(event) => setPaymentDate(event.target.value)}
                />
              </Field>

              <Field label="Method" htmlFor="pay-method" required>
                <Select
                  id="pay-method"
                  value={method}
                  onChange={(event) => setMethod(event.target.value)}
                >
                  {METHODS.map((value) => (
                    <option key={value} value={value}>
                      {value.replace(/_/g, " ")}
                    </option>
                  ))}
                </Select>
              </Field>

              <Field
                label="Reference"
                htmlFor="pay-reference"
                hint={
                  method === "CASH"
                    ? "Cash payments do not need a reference."
                    : "Must be unique for this method."
                }
              >
                <Input
                  id="pay-reference"
                  value={reference}
                  onChange={(event) => setReference(event.target.value)}
                />
              </Field>
            </div>

            {studentId ? (
              <div className="grid gap-4 sm:grid-cols-3">
                <div className="rounded-md border border-slate-200 px-4 py-3">
                  <p className="text-xs font-medium uppercase tracking-wide text-slate-500">
                    Outstanding
                  </p>
                  <p className="mt-1 text-xl font-semibold text-slate-900">
                    {formatMoney(outstandingCents)}
                  </p>
                </div>
                <div className="rounded-md border border-slate-200 px-4 py-3">
                  <p className="text-xs font-medium uppercase tracking-wide text-slate-500">
                    Credit on account
                  </p>
                  <p className="mt-1 text-xl font-semibold text-emerald-600">
                    {formatMoney(creditCents)}
                  </p>
                </div>
                <div className="rounded-md border border-slate-200 px-4 py-3">
                  <p className="text-xs font-medium uppercase tracking-wide text-slate-500">
                    Net balance
                  </p>
                  <p className="mt-1 text-xl font-semibold text-slate-900">
                    {formatMoney(outstandingCents - creditCents)}
                  </p>
                  <p className="text-xs text-slate-500">
                    {outstandingCents - creditCents < 0
                      ? "The student is in credit."
                      : outstandingCents - creditCents === 0
                        ? "The account is settled."
                        : "Money still owed."}
                  </p>
                </div>
              </div>
            ) : null}

            <div>
              <p className="mb-2 text-sm font-medium text-slate-700">
                Outstanding invoices
              </p>

              {!studentId ? (
                <EmptyState title="Choose a student first" />
              ) : invoices.isLoading ? (
                <Loading />
              ) : outstanding.length === 0 ? (
                <EmptyState
                  title="Nothing billed yet"
                  description="This payment will sit on the student's account as credit and clear future invoices automatically."
                />
              ) : (
                <div className="space-y-2">
                  {outstanding.map((invoice) => {
                    const due = invoice.totalCents - invoice.paidCents
                    return (
                      <div
                        key={invoice.id}
                        className="flex flex-wrap items-center gap-3 rounded-md border border-slate-200 px-3 py-2"
                      >
                        <div className="min-w-0 flex-1">
                          <Link
                            to={`/billing/invoices/${invoice.id}`}
                            className="font-mono text-xs text-indigo-600 hover:text-indigo-800"
                          >
                            {invoice.invoiceNumber}
                          </Link>
                          <p className="text-xs text-slate-500">
                            Due {formatDate(invoice.dueDate)} &middot; outstanding{" "}
                            {formatMoney(due)} &middot;{" "}
                            <StatusBadge status={invoice.status} />
                          </p>
                        </div>
                        <span className="text-sm font-medium text-slate-900">
                          {formatMoney(due)}
                        </span>
                      </div>
                    )
                  })}
                </div>
              )}
            </div>

            <div className="grid gap-4 border-t border-slate-200 pt-4 sm:grid-cols-2 lg:grid-cols-4">
              <Field label="Amount received" htmlFor="pay-amount" required>
                <Input
                  id="pay-amount"
                  inputMode="decimal"
                  placeholder="0.00"
                  value={amount}
                  onChange={(event) => setAmount(event.target.value)}
                />
              </Field>
              <div className="sm:col-span-2 lg:col-span-3 lg:self-end">
                {studentId ? (
                  <p className="text-sm text-slate-600">
                    {amountCents > 0 ? (
                      <>
                        {appliedCents > 0 ? (
                          <>
                            Clears{" "}
                            <span className="font-semibold text-slate-900">
                              {formatMoney(appliedCents)}
                            </span>{" "}
                            of invoices.{" "}
                          </>
                        ) : null}
                        {creditAfterCents > 0 ? (
                          <>
                            <span className="font-semibold text-emerald-600">
                              {formatMoney(creditAfterCents)}
                            </span>{" "}
                            stays as credit.
                          </>
                        ) : null}
                        {outstandingAfterCents > 0 ? (
                          <>
                            {" "}
                            <span className="font-semibold text-rose-600">
                              {formatMoney(outstandingAfterCents)}
                            </span>{" "}
                            still owed.
                          </>
                        ) : null}
                      </>
                    ) : (
                      "Enter the amount the school received."
                    )}
                  </p>
                ) : null}
              </div>
            </div>

            <div className="flex flex-wrap items-center justify-between gap-3">
              <p className="text-sm text-slate-600">
                Payment total:{" "}
                <span className="text-base font-semibold text-slate-900">
                  {formatMoney(amountCents)}
                </span>
              </p>
              <Button onClick={submit} disabled={!canSubmit} loading={mutation.isPending}>
                Record payment
              </Button>
            </div>
          </div>
        </Card>
      </div>

      <div className="no-print">
        <Card
          title="Payments recorded"
          description="Every payment received, newest first. Open a receipt to reprint it."
        >
          {payments.isError ? <ErrorBanner error={payments.error} /> : null}
          {payments.isLoading ? (
            <Loading />
          ) : (
            <Table
              columns={historyColumns}
              rows={payments.data ?? []}
              keyOf={(payment) => payment.id}
              empty={
                <EmptyState
                  title="No payments yet"
                  description="Payments recorded on this page will appear here."
                />
              }
            />
          )}
        </Card>
      </div>

      {preview ? (
        <div className="fixed inset-0 z-50 overflow-y-auto bg-slate-900/40 p-4 sm:p-6 print:static print:overflow-visible print:bg-white print:p-0">
          <div className="mx-auto my-8 w-full max-w-md rounded-lg bg-white shadow-xl ring-1 ring-slate-900/10 print:my-0 print:max-w-none print:shadow-none print:ring-0">
            <header className="no-print flex items-start justify-between gap-4 border-b border-slate-200 px-5 py-4">
              <div className="min-w-0">
                <h2 className="text-base font-semibold text-slate-900">
                  Receipt {preview.receiptNumber}
                </h2>
                <p className="mt-0.5 text-xs text-slate-500">
                  Preview of the printed slip.
                </p>
              </div>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => setPreview(null)}
              >
                Close
              </Button>
            </header>
            <div className="px-4 py-3 print:p-0">
              {previewAllocations.isLoading ? (
                <Loading />
              ) : (
                <ReceiptDocument
                  payment={preview}
                  allocations={previewAllocations.data ?? []}
                  studentName={
                    previewStudent ? studentName(previewStudent) : "-"
                  }
                  studentNumber={previewStudent?.studentNumber ?? "-"}
                  invoiceNumbers={invoiceNumbers}
                  receivedByName={previewReceivedBy}
                />
              )}
            </div>
            <footer className="no-print flex justify-end gap-2 border-t border-slate-200 px-5 py-3">
              <Button variant="secondary" onClick={() => setPreview(null)}>
                Close
              </Button>
              <Button
                onClick={() => printDocument("receipt")}
                disabled={previewAllocations.isLoading}
              >
                Print receipt
              </Button>
            </footer>
          </div>
        </div>
      ) : null}
    </Page>
  )
}
