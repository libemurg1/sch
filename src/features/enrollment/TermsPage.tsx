import { useState } from "react"
import { useDeleteTerm, useTerms } from "../../api/queries"
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
  type Column,
} from "../../components"
import { Page } from "../../app/AppShell"
import { formatTermLabel } from "../../lib/dates"
import type { Term } from "../../types/schema/Term"
import { CreateTermModal } from "./CreateTermModal"

export function TermsPage() {
  const terms = useTerms()
  const [addingTerm, setAddingTerm] = useState(false)
  const [editingTerm, setEditingTerm] = useState<Term | null>(null)
  const [deletingTerm, setDeletingTerm] = useState<Term | null>(null)
  const removeTerm = useDeleteTerm()

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
        {terms.isLoading ? (
          <Loading />
        ) : terms.isError ? (
          <ErrorBanner error={terms.error} onRetry={() => terms.refetch()} />
        ) : (terms.data ?? []).length === 0 ? (
          <EmptyState
            title="No terms yet"
            description="Create a term to start billing and enrollment for a new cycle."
            action={<Button onClick={() => setAddingTerm(true)}>Add term</Button>}
          />
        ) : (
          <Table
            columns={termColumns}
            rows={terms.data ?? []}
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
