import { useEffect, useState } from "react"
import { Link } from "react-router-dom"
import {
  useActiveTerm,
  useAppInfo,
  useClasses,
  useGenerateInvoiceBatch,
  useGenerateSupplementaryInvoice,
  useAssignStudentFee,
  useStudents,
  useTerms,
} from "../../api/queries"
import {
  Button,
  Card,
  ConfirmDialog,
  ErrorBanner,
  Field,
  Input,
  PageHeader,
  SearchablePicker,
  StudentPicker,
} from "../../components"
import { Page } from "../../app/AppShell"
import { formatDate, formatTermLabel } from "../../lib/dates"
import { formatMoney, parseMoneyToCents } from "../../lib/money"

export function BillingPage() {
  const classes = useClasses()
  const terms = useTerms()
  const activeTerm = useActiveTerm()
  const appInfo = useAppInfo()
  const students = useStudents()

  const today = appInfo.data?.today ?? ""

  const [classId, setClassId] = useState("")
  const [termId, setTermId] = useState("")
  const [issueDate, setIssueDate] = useState("")
  const [dueDate, setDueDate] = useState("")

  const [supplementStudentId, setSupplementStudentId] = useState("")
  const [supplementTermId, setSupplementTermId] = useState("")
  const [supplementIssueDate, setSupplementIssueDate] = useState("")
  const [supplementDueDate, setSupplementDueDate] = useState("")

  const [feeStudentId, setFeeStudentId] = useState("")
  const [feeTermId, setFeeTermId] = useState("")
  const [feeDescription, setFeeDescription] = useState("")
  const [feeAmountText, setFeeAmountText] = useState("")
  const [feeDiscountable, setFeeDiscountable] = useState(true)
  const [feeError, setFeeError] = useState<string | null>(null)
  const [confirmAction, setConfirmAction] = useState<
    "class" | "supplementary" | "fee" | null
  >(null)

  const batch = useGenerateInvoiceBatch()
  const supplementary = useGenerateSupplementaryInvoice()
  const studentFee = useAssignStudentFee()

  useEffect(() => {
    if (!today) return
    if (!issueDate) setIssueDate(today)
    if (!dueDate) setDueDate(today)
    if (!supplementIssueDate) setSupplementIssueDate(today)
    if (!supplementDueDate) setSupplementDueDate(today)
  }, [today, issueDate, dueDate, supplementIssueDate, supplementDueDate])

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
    setConfirmAction("fee")
  }

  const selectedClass = (classes.data ?? []).find((item) => item.id === classId)
  const selectedTerm = (terms.data ?? []).find((item) => item.id === termId)
  const supplementaryStudent = (students.data ?? []).find(
    (student) => student.id === supplementStudentId,
  )
  const feeStudent = (students.data ?? []).find(
    (student) => student.id === feeStudentId,
  )
  const feeTerm = (terms.data ?? []).find((term) => term.id === feeTermId)

  const confirmSelectedAction = () => {
    if (confirmAction === "class") {
      batch.mutate(
        { classId, termId, issueDate, dueDate },
        { onSuccess: () => setConfirmAction(null) },
      )
    } else if (confirmAction === "supplementary") {
      supplementary.mutate(
        {
          studentId: supplementStudentId,
          termId: supplementTermId,
          issueDate: supplementIssueDate,
          dueDate: supplementDueDate,
        },
        { onSuccess: () => setConfirmAction(null) },
      )
    } else if (confirmAction === "fee") {
      const cents = parseMoneyToCents(feeAmountText)
      if (cents !== null) {
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
              setConfirmAction(null)
            },
          },
        )
      }
    }
  }

  const confirmLoading =
    (confirmAction === "class" && batch.isPending) ||
    (confirmAction === "supplementary" && supplementary.isPending) ||
    (confirmAction === "fee" && studentFee.isPending)
  const confirmError =
    confirmAction === "class"
      ? batch.error
      : confirmAction === "supplementary"
        ? supplementary.error
        : studentFee.error

  const createdCount = Array.isArray(batch.data) ? batch.data.length : 0
  const batchDatesInvalid =
    !issueDate || !dueDate || dueDate < issueDate
  const supplementaryDatesInvalid =
    !supplementIssueDate ||
    !supplementDueDate ||
    supplementDueDate < supplementIssueDate

  return (
    <Page>
      <PageHeader
        title="Billing"
        description="Add student-specific charges before class billing to include them on the student's term invoice."
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
                {createdCount > 0 ? (
                  <>
                    {" "}
                    <Link to="/invoices" className="font-semibold underline">
                      Review invoices
                    </Link>
                  </>
                ) : null}
              </div>
            ) : null}

            <div className="grid gap-4 sm:grid-cols-2">
              <Field label="Class" htmlFor="bill-class" required>
                <SearchablePicker
                  id="bill-class"
                  value={classId}
                  onChange={setClassId}
                  options={(classes.data ?? []).map((klass) => ({
                    id: klass.id,
                    label: klass.name,
                  }))}
                  placeholder="Choose a class"
                  searchPlaceholder="Search classes"
                />
              </Field>
              <Field label="Term" htmlFor="bill-term" required>
                <SearchablePicker
                  id="bill-term"
                  value={termId}
                  onChange={setTermId}
                  options={(terms.data ?? []).map((term) => ({
                    id: term.id,
                    label: formatTermLabel(term),
                  }))}
                  placeholder="Choose a term"
                  searchPlaceholder="Search terms"
                />
              </Field>
              <Field label="Issue date" htmlFor="bill-issue" required>
                <Input
                  id="bill-issue"
                  type="date"
                  value={issueDate}
                  onChange={(event) => setIssueDate(event.target.value)}
                />
              </Field>
              <Field
                label="Due date"
                htmlFor="bill-due"
                required
                error={
                  dueDate && issueDate && dueDate < issueDate
                    ? "Due date cannot be before the issue date"
                    : null
                }
              >
                <Input
                  id="bill-due"
                  type="date"
                  min={issueDate}
                  value={dueDate}
                  onChange={(event) => setDueDate(event.target.value)}
                />
              </Field>
            </div>

            <Button
              onClick={() => setConfirmAction("class")}
              loading={batch.isPending}
              disabled={!classId || !termId || batchDatesInvalid}
            >
              Raise invoices
            </Button>
          </div>
        </Card>

        <Card
          title="Supplementary invoice"
          description="Use this for charges added after the student's class invoice has already been issued."
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
              <StudentPicker
                id="supp-student"
                value={supplementStudentId}
                onChange={(value) => {
                  setSupplementStudentId(value)
                  supplementary.reset()
                }}
                students={students.data ?? []}
              />
            </Field>

            <Field label="Term" htmlFor="supp-term" required>
              <SearchablePicker
                id="supp-term"
                value={supplementTermId}
                onChange={setSupplementTermId}
                options={(terms.data ?? []).map((term) => ({
                  id: term.id,
                  label: formatTermLabel(term),
                }))}
                placeholder="Choose a term"
                searchPlaceholder="Search terms"
              />
            </Field>

            <div className="grid gap-4 sm:grid-cols-2">
              <Field label="Issue date" htmlFor="supp-issue" required>
                <Input
                  id="supp-issue"
                  type="date"
                  value={supplementIssueDate}
                  onChange={(event) =>
                    setSupplementIssueDate(event.target.value)
                  }
                />
              </Field>
              <Field
                label="Due date"
                htmlFor="supp-due"
                required
                error={
                  supplementDueDate &&
                  supplementIssueDate &&
                  supplementDueDate < supplementIssueDate
                    ? "Due date cannot be before the issue date"
                    : null
                }
              >
                <Input
                  id="supp-due"
                  type="date"
                  min={supplementIssueDate}
                  value={supplementDueDate}
                  onChange={(event) => setSupplementDueDate(event.target.value)}
                />
              </Field>
            </div>

            <Button
              variant="secondary"
              onClick={() => setConfirmAction("supplementary")}
              loading={supplementary.isPending}
              disabled={
                !supplementStudentId ||
                !supplementTermId ||
                supplementaryDatesInvalid
              }
            >
              Raise supplementary invoice
            </Button>
          </div>
        </Card>
      </div>

      <Card
        title="Student-specific charge"
        description="Record an extra charge for one student. Pending charges are included when you next raise that student's class invoice; use a supplementary invoice if the class invoice already exists."
      >
        <div className="space-y-4">
          {studentFee.isError ? <ErrorBanner error={studentFee.error} /> : null}
          {studentFee.isSuccess ? (
            <div className="rounded-md bg-emerald-50 px-4 py-3 text-sm text-emerald-800 ring-1 ring-inset ring-emerald-200">
              The charge has been recorded. It will be included on the class
              invoice if it has not been raised yet.
            </div>
          ) : null}

          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <Field label="Student" htmlFor="fee-student" required>
              <StudentPicker
                id="fee-student"
                value={feeStudentId}
                onChange={(value) => {
                  setFeeStudentId(value)
                  studentFee.reset()
                }}
                students={students.data ?? []}
              />
            </Field>
            <Field label="Term" htmlFor="fee-term" required>
              <SearchablePicker
                id="fee-term"
                value={feeTermId}
                onChange={(value) => {
                  setFeeTermId(value)
                  studentFee.reset()
                }}
                options={(terms.data ?? []).map((term) => ({
                  id: term.id,
                  label: formatTermLabel(term),
                }))}
                placeholder="Choose a term"
                searchPlaceholder="Search terms"
              />
            </Field>
            <Field label="Description" htmlFor="fee-description" required>
              <Input
                id="fee-description"
                value={feeDescription}
                placeholder="Late registration fee"
                onChange={(event) => {
                  setFeeDescription(event.target.value)
                  setFeeError(null)
                  studentFee.reset()
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
                  studentFee.reset()
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
                onChange={(event) => {
                  setFeeDiscountable(event.target.checked)
                  studentFee.reset()
                }}
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

      <ConfirmDialog
        open={confirmAction !== null}
        title={
          confirmAction === "class"
            ? "Review class billing"
            : confirmAction === "supplementary"
              ? "Review supplementary invoice"
              : "Review student charge"
        }
        confirmLabel={
          confirmAction === "class"
            ? "Create invoices"
            : confirmAction === "supplementary"
              ? "Create invoice"
              : "Add charge"
        }
        loading={confirmLoading}
        error={confirmError}
        onCancel={() => {
          if (confirmAction === "class") batch.reset()
          if (confirmAction === "supplementary") supplementary.reset()
          if (confirmAction === "fee") studentFee.reset()
          setConfirmAction(null)
        }}
        onConfirm={confirmSelectedAction}
      >
        {confirmAction === "class" ? (
          <p>
            Create term invoices for eligible students in{" "}
            <strong>{selectedClass?.name ?? "the selected class"}</strong> for{" "}
            <strong>
              {selectedTerm ? formatTermLabel(selectedTerm) : "the selected term"}
            </strong>
            . Students with an existing invoice for this term will not receive a
            duplicate. Issue date: {formatDate(issueDate)}; due date:{" "}
            {formatDate(dueDate)}. Review the class fee structure before
            continuing.
          </p>
        ) : confirmAction === "supplementary" ? (
          <p>
            Create a supplementary invoice for{" "}
            <strong>
              {supplementaryStudent
                ? `${supplementaryStudent.firstName} ${supplementaryStudent.lastName}`
                : "the selected student"}
            </strong>
            . Issue date: {formatDate(supplementIssueDate)}; due date:{" "}
            {formatDate(supplementDueDate)}. Confirm that this student needs a
            separate invoice for eligible student-specific charges.
          </p>
        ) : (
          <p>
            Add <strong>{feeDescription}</strong> for{" "}
            <strong>
              {feeStudent
                ? `${feeStudent.firstName} ${feeStudent.lastName}`
                : "the selected student"}
            </strong>{" "}
            in <strong>{feeTerm ? formatTermLabel(feeTerm) : "the selected term"}</strong>{" "}
            for <strong>{formatMoney(parseMoneyToCents(feeAmountText) ?? 0)}</strong>.
            {feeDiscountable
              ? " Applicable discounts may affect this charge."
              : " This charge is not discountable."}
          </p>
        )}
      </ConfirmDialog>
    </Page>
  )
}
