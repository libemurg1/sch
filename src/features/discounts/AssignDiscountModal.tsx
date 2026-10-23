import { useEffect, useMemo, useState } from "react"
import {
  useAssignDiscount,
  useDiscounts,
  useFeeStructure,
  useStudentEnrollments,
  useStudents,
  useTerms,
} from "../../api/queries"
import {
  Button,
  ErrorBanner,
  Field,
  Modal,
  Select,
  StudentPicker,
} from "../../components"
import { formatTermLabel } from "../../lib/dates"
import { formatBasisPoints, formatMoney } from "../../lib/money"
import type { AssignDiscountInput } from "../../types/schema/AssignDiscountInput"

export interface AssignDiscountModalProps {
  open: boolean
  onClose: () => void
  /** Pre-selected student, when the dialog is opened from a student's page. */
  studentId?: string
}

export function AssignDiscountModal({
  open,
  onClose,
  studentId: presetStudentId,
}: AssignDiscountModalProps) {
  const discounts = useDiscounts()
  const terms = useTerms()
  const students = useStudents()
  const mutation = useAssignDiscount()

  const [studentId, setStudentId] = useState("")
  const [discountId, setDiscountId] = useState("")
  const [termId, setTermId] = useState("")
  const [targetFeeItemId, setTargetFeeItemId] = useState("")
  const [reason, setReason] = useState("")

  const effectiveStudentId = presetStudentId ?? studentId
  const enrollments = useStudentEnrollments(effectiveStudentId || undefined)

  // The class the student sits in for the chosen term decides which fee items
  // a discount can target.
  const classId = useMemo(() => {
    if (!termId) {
      return undefined
    }
    const match = (enrollments.data ?? []).find(
      (enrollment) => enrollment.termId === termId,
    )
    return match?.classId
  }, [enrollments.data, termId])

  const structure = useFeeStructure(classId, termId || undefined)
  const feeItems = structure.data ? structure.data[1] : []

  useEffect(() => {
    if (open) {
      setStudentId(presetStudentId ?? "")
      setDiscountId("")
      setTermId("")
      setTargetFeeItemId("")
      setReason("")
      mutation.reset()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, presetStudentId])

  const submit = () => {
    const input: AssignDiscountInput = {
      studentId: effectiveStudentId,
      discountId,
      termId: termId || null,
      targetFeeItemId: targetFeeItemId || null,
      targetStudentFeeId: null,
      reason: reason.trim() === "" ? null : reason.trim(),
    }
    mutation.mutate(input, { onSuccess: onClose })
  }

  const selectedDiscount = (discounts.data ?? []).find(
    (discount) => discount.id === discountId,
  )

  return (
    <Modal
      open={open}
      onClose={onClose}
      title="Grant a discount"
      description="A discount without a target applies to the whole invoice."
      footer={
        <>
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button
            onClick={submit}
            loading={mutation.isPending}
            disabled={!effectiveStudentId || !discountId}
          >
            Grant discount
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        {mutation.isError ? <ErrorBanner error={mutation.error} /> : null}

        {!presetStudentId ? (
          <Field label="Student" htmlFor="discount-student" required>
            <StudentPicker
              id="discount-student"
              value={studentId}
              onChange={setStudentId}
              students={students.data ?? []}
            />
          </Field>
        ) : null}

        <Field label="Discount" htmlFor="discount-choice" required>
          <Select
            id="discount-choice"
            value={discountId}
            onChange={(event) => setDiscountId(event.target.value)}
          >
            <option value="">Choose a discount</option>
            {(discounts.data ?? []).map((discount) => (
              <option key={discount.id} value={discount.id}>
                {discount.name} (
                {discount.kind === "PERCENT"
                  ? formatBasisPoints(discount.value)
                  : formatMoney(discount.value)}
                )
              </option>
            ))}
          </Select>
        </Field>

        <Field
          label="Term"
          htmlFor="discount-term"
          hint="Leave blank to apply the discount in every term."
        >
          <Select
            id="discount-term"
            value={termId}
            onChange={(event) => {
              setTermId(event.target.value)
              setTargetFeeItemId("")
            }}
          >
            <option value="">Every term</option>
            {(terms.data ?? []).map((term) => (
              <option key={term.id} value={term.id}>
                {formatTermLabel(term)}
              </option>
            ))}
          </Select>
        </Field>

        <Field
          label="Applies to"
          htmlFor="discount-target"
          hint={
            termId
              ? "Targeting a single fee item keeps the discount off the other charges."
              : "Choose a term to target a single fee item."
          }
        >
          <Select
            id="discount-target"
            value={targetFeeItemId}
            onChange={(event) => setTargetFeeItemId(event.target.value)}
            disabled={!termId}
          >
            <option value="">The whole invoice</option>
            {feeItems.map((item) => (
              <option key={item.id} value={item.id}>
                {item.description}
              </option>
            ))}
          </Select>
        </Field>

        <Field label="Reason" htmlFor="discount-reason">
          <Select
            id="discount-reason"
            value={reason}
            onChange={(event) => setReason(event.target.value)}
          >
            <option value="">No reason given</option>
            <option value="Sibling attends the school">Sibling attends the school</option>
            <option value="Staff child">Staff child</option>
            <option value="Scholarship">Scholarship</option>
            <option value="Agreed with the guardian">Agreed with the guardian</option>
          </Select>
        </Field>

        {selectedDiscount ? (
          <p className="text-xs text-slate-500">
            {selectedDiscount.kind === "PERCENT"
              ? `This takes ${formatBasisPoints(selectedDiscount.value)} off the discountable charges.`
              : `This takes ${formatMoney(selectedDiscount.value)} off the whole invoice.`}
          </p>
        ) : null}
      </div>
    </Modal>
  )
}
