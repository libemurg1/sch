import { useState } from "react"
import { useDiscounts } from "../../api/queries"
import {
  Badge,
  Button,
  Card,
  EmptyState,
  ErrorBanner,
  Loading,
  PageHeader,
  Table,
  type Column,
} from "../../components"
import { Page } from "../../app/AppShell"
import { formatBasisPoints, formatMoney } from "../../lib/money"
import type { Discount } from "../../types/schema/Discount"
import { AssignDiscountModal } from "./AssignDiscountModal"
import { CreateDiscountModal } from "./CreateDiscountModal"

export function DiscountsPage() {
  const discounts = useDiscounts()
  const [creating, setCreating] = useState(false)
  const [assigning, setAssigning] = useState(false)

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
    </Page>
  )
}
