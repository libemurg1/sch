import { useEffect, useMemo, useState } from "react"
import { useActiveTerm, useDeleteTerm, useTerms } from "../../api/queries"
import {
  Badge,
  Button,
  Card,
  ConfirmDialog,
  EmptyState,
  ErrorBanner,
  Loading,
  PageHeader,
  Table,
  YearSelect,
  type Column,
} from "../../components"
import { Page } from "../../app/AppShell"
import { formatTermLabel } from "../../lib/dates"
import { ALL_YEARS, collectYears, defaultYear } from "../../lib/years"
import type { Term } from "../../types/schema/Term"
import { CreateTermModal } from "./CreateTermModal"

export function TermsPage() {
  const terms = useTerms()
  const activeTerm = useActiveTerm()
  const [year, setYear] = useState("")
  const [addingTerm, setAddingTerm] = useState(false)
  const [editingTerm, setEditingTerm] = useState<Term | null>(null)
  const [deletingTerm, setDeletingTerm] = useState<Term | null>(null)
  const removeTerm = useDeleteTerm()

  const years = useMemo(
    () =>
      collectYears([
        ...(terms.data ?? []).map((term) => term.year),
        activeTerm.data?.year,
      ]),
    [terms.data, activeTerm.data],
  )

  useEffect(() => {
    if (year !== "") {
      return
    }
    if (terms.isLoading || activeTerm.isLoading) {
      return
    }
    setYear(defaultYear(activeTerm.data?.year, years))
  }, [year, terms.isLoading, activeTerm.isLoading, activeTerm.data, years])

  const allTerms = terms.data ?? []
  const rows =
    year === ALL_YEARS || year === ""
      ? allTerms
      : allTerms.filter((term) => term.year === Number(year))

  const termColumns: Column<Term>[] = [
    {
      key: "term",
      header: "Term",
      render: (term) => (
        <span className="font-medium text-slate-900">
          {formatTermLabel(term)}
        </span>
      ),
    },
    {
      key: "dates",
      header: "Dates",
      hideOnSmall: true,
      render: (term) => `${term.startDate} to ${term.endDate}`,
    },
    {
      key: "active",
      header: "",
      align: "right",
      render: (term) =>
        term.isActive ? <Badge tone="success">Active</Badge> : null,
    },
    {
      key: "actions",
      header: "",
      align: "right",
      render: (term) => (
        <div
          className="flex justify-end gap-1"
          onClick={(event) => event.stopPropagation()}
        >
          <Button
            size="sm"
            variant="ghost"
            onClick={() => setEditingTerm(term)}
          >
            Edit
          </Button>
          <Button
            size="sm"
            variant="ghost"
            className="text-rose-600 hover:bg-rose-50 hover:text-rose-700"
            onClick={() => setDeletingTerm(term)}
          >
            Delete
          </Button>
        </div>
      ),
    },
  ]

  return (
    <Page>
      <PageHeader
        title="Terms"
        description="Manage academic terms used for enrollment and billing."
      />

      <Card
        title="Terms"
        actions={
          <Button size="sm" variant="secondary" onClick={() => setAddingTerm(true)}>
            Add term
          </Button>
        }
      >
        <div className="mb-4 flex flex-wrap gap-3">
          <YearSelect
            id="term-year-filter"
            value={year}
            onChange={setYear}
            years={years}
          />
        </div>

        {terms.isLoading ? (
          <Loading />
        ) : terms.isError ? (
          <ErrorBanner error={terms.error} onRetry={() => terms.refetch()} />
        ) : rows.length === 0 ? (
          <EmptyState
            title={allTerms.length === 0 ? "No terms yet" : "No terms in this year"}
            description={
              allTerms.length === 0
                ? "Create a term to start billing and enrollment for a new cycle."
                : "Choose another year or add a term for this year."
            }
            action={<Button onClick={() => setAddingTerm(true)}>Add term</Button>}
          />
        ) : (
          <Table
            columns={termColumns}
            rows={rows}
            keyOf={(term) => term.id}
          />
        )}
      </Card>

      <CreateTermModal open={addingTerm} onClose={() => setAddingTerm(false)} />
      <CreateTermModal
        open={editingTerm !== null}
        editing={editingTerm}
        onClose={() => setEditingTerm(null)}
      />

      <ConfirmDialog
        open={deletingTerm !== null}
        title="Delete term"
        loading={removeTerm.isPending}
        error={removeTerm.error}
        onCancel={() => {
          removeTerm.reset()
          setDeletingTerm(null)
        }}
        onConfirm={() => {
          if (!deletingTerm) {
            return
          }
          removeTerm.mutate(deletingTerm.id, {
            onSuccess: () => {
              setDeletingTerm(null)
            },
          })
        }}
      >
        <p>
          Delete <strong>{deletingTerm ? formatTermLabel(deletingTerm) : ""}</strong>
          ? This cannot be undone.
        </p>
        <p className="mt-2">
          A term that already has enrolments, fees, invoices or discounts cannot
          be deleted.
        </p>
      </ConfirmDialog>
    </Page>
  )
}
