import { useState } from "react"
import {
  useAllStudentDiscounts,
  useDiscounts,
  useRevokeDiscount,
  useStudents,
  useTerms,
} from "../../api/queries"
import {
  Badge,
  Button,
  Card,
  ConfirmDialog,
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
import { formatTermLabel, studentName } from "../../lib/dates"
import { formatBasisPoints, formatMoney } from "../../lib/money"
import type { Discount } from "../../types/schema/Discount"
import type { StudentDiscount } from "../../types/schema/StudentDiscount"
import { AssignDiscountModal } from "./AssignDiscountModal"
import { CreateDiscountModal } from "./CreateDiscountModal"

export function DiscountsPage() {
  const discounts = useDiscounts()
  const grants = useAllStudentDiscounts()
  const students = useStudents()
  const terms = useTerms()
  const revokeMutation = useRevokeDiscount()
  const [creating, setCreating] = useState(false)
  const [assigning, setAssigning] = useState(false)
  const [revoking, setRevoking] = useState<StudentDiscount | null>(null)
  const [revokeReason, setRevokeReason] = useState("")

  const studentNames = new Map(
    (students.data ?? []).map((student) => [student.id, studentName(student)]),
  )
  const discountNames = new Map(
    (discounts.data ?? []).map((discount) => [discount.id, discount.name]),
  )
  const termLabels = new Map(
    (terms.data ?? []).map((term) => [term.id, formatTermLabel(term)]),
  )

  const columns: Column<Discount>[] = [
    {
      key: "name",
      header: "Name",
      render: (discount) => (
        <span className="font-medium text-slate-900">{discount.name}</span>
      ),
    },
    {
      key: "kind",
      header: "Kind",
      render: (discount) =>
        discount.kind === "PERCENT" ? (
          <Badge tone="info">Percentage</Badge>
        ) : (
          <Badge tone="neutral">Fixed</Badge>
        ),
    },
    {
      key: "value",
      header: "Value",
      align: "right",
      render: (discount) =>
        discount.kind === "PERCENT"
          ? formatBasisPoints(discount.value)
          : formatMoney(discount.value),
    },
    {
      key: "active",
      header: "Status",
      align: "right",
      render: (discount) =>
        discount.isActive ? (
          <Badge tone="success">Active</Badge>
        ) : (
          <Badge tone="neutral">Inactive</Badge>
        ),
    },
  ]

  const grantColumns: Column<StudentDiscount>[] = [
    {
      key: "student",
      header: "Student",
      render: (grant) => studentNames.get(grant.studentId) ?? grant.studentId,
    },
    {
      key: "discount",
      header: "Discount",
      render: (grant) => discountNames.get(grant.discountId) ?? grant.discountId,
    },
    {
      key: "term",
      header: "Term",
      render: (grant) => (grant.termId ? termLabels.get(grant.termId) ?? grant.termId : "Every term"),
    },
    {
      key: "status",
      header: "Status",
      render: (grant) =>
        grant.revokedAt ? (
          <Badge tone="neutral">Revoked</Badge>
        ) : (
          <Badge tone="success">Active</Badge>
        ),
    },
    {
      key: "reason",
      header: "Reason",
      render: (grant) => grant.revokeReason ?? grant.reason ?? "-",
    },
    {
      key: "actions",
      header: "",
      align: "right",
      render: (grant) =>
        grant.revokedAt ? null : (
          <Button
            size="sm"
            variant="danger"
            onClick={() => {
              revokeMutation.reset()
              setRevokeReason("")
              setRevoking(grant)
            }}
          >
            Revoke
          </Button>
        ),
    },
  ]

  return (
    <Page>
      <PageHeader
        title="Discounts"
        description="Discount definitions are shared. Grant them to individual students."
        actions={
          <>
            <Button variant="secondary" onClick={() => setAssigning(true)}>
              Grant to student
            </Button>
            <Button onClick={() => setCreating(true)}>Add discount</Button>
          </>
        }
      />

      <Card title="Discount definitions">
        {discounts.isLoading ? (
          <Loading />
        ) : discounts.isError ? (
          <ErrorBanner error={discounts.error} onRetry={() => discounts.refetch()} />
        ) : (discounts.data ?? []).length === 0 ? (
          <EmptyState
            title="No discounts yet"
            description="Add a percentage or fixed discount, then grant it to a student."
            action={<Button onClick={() => setCreating(true)}>Add discount</Button>}
          />
        ) : (
          <Table
            columns={columns}
            rows={discounts.data ?? []}
            keyOf={(discount) => discount.id}
          />
        )}
      </Card>

      <Card title="Student discount grants" description="Revoking a grant affects future invoices; issued invoices remain unchanged.">
        {grants.isLoading ? (
          <Loading />
        ) : grants.isError ? (
          <ErrorBanner error={grants.error} onRetry={() => grants.refetch()} />
        ) : (
          <Table
            columns={grantColumns}
            rows={grants.data ?? []}
            keyOf={(grant) => grant.id}
            empty={<EmptyState title="No student discounts granted yet" />}
          />
        )}
      </Card>

      <Card title="How discounts work">
        <ul className="list-disc space-y-1.5 pl-5 text-sm text-slate-600">
          <li>
            Percentage discounts use basis points on the wire (1000 is ten
            percent) and only touch charges marked discountable.
          </li>
          <li>
            Percentages do not compound. Each one is rounded on its own and
            they are added together.
          </li>
          <li>Percentages that share a target stop at one hundred percent of it.</li>
          <li>Fixed discounts come off the whole invoice.</li>
          <li>
            Everything stacks, but the invoice total never drops below zero.
          </li>
          <li>
            Each discount is stored on the invoice as its own negative line, so
            the invoice always shows how it was worked out.
          </li>
        </ul>
      </Card>

      <CreateDiscountModal open={creating} onClose={() => setCreating(false)} />
      <AssignDiscountModal open={assigning} onClose={() => setAssigning(false)} />
      <ConfirmDialog
        open={revoking !== null}
        title="Revoke student discount"
        confirmLabel="Revoke discount"
        loading={revokeMutation.isPending}
        error={revokeMutation.error}
        onCancel={() => {
          revokeMutation.reset()
          setRevoking(null)
        }}
        onConfirm={() => {
          if (revoking && revokeReason.trim()) {
            revokeMutation.mutate(
              { grantId: revoking.id, reason: revokeReason.trim() },
              { onSuccess: () => setRevoking(null) },
            )
          }
        }}
      >
        <p>
          Revoke the grant of{" "}
          <strong>{discountNames.get(revoking?.discountId ?? "") ?? "this discount"}</strong>
          ? This does not change any invoice that has already been issued.
        </p>
        <Field label="Reason" htmlFor="discount-revoke-reason" required>
          <Input
            id="discount-revoke-reason"
            value={revokeReason}
            onChange={(event) => setRevokeReason(event.target.value)}
            required
            maxLength={500}
            placeholder="Explain why this discount is being revoked"
          />
        </Field>
      </ConfirmDialog>
    </Page>
  )
}
