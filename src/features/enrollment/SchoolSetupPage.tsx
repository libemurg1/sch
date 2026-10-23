import { useEffect, useState } from "react"
import {
  useActiveTerm,
  useClasses,
  useDeleteClass,
  useDeleteTerm,
  useTerms,
} from "../../api/queries"
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
import type { Class } from "../../types/schema/Class"
import type { Term } from "../../types/schema/Term"
import { CreateClassModal } from "./CreateClassModal"
import { CreateTermModal } from "./CreateTermModal"

export function SchoolSetupPage() {
  const classes = useClasses()
  const terms = useTerms()
  const activeTerm = useActiveTerm()

  const [classId, setClassId] = useState("")
  const [termId, setTermId] = useState("")
  const [addingClass, setAddingClass] = useState(false)
  const [addingTerm, setAddingTerm] = useState(false)
  const [editingClass, setEditingClass] = useState<Class | null>(null)
  const [editingTerm, setEditingTerm] = useState<Term | null>(null)
  const [deletingClass, setDeletingClass] = useState<Class | null>(null)
  const [deletingTerm, setDeletingTerm] = useState<Term | null>(null)

  const removeClass = useDeleteClass()
  const removeTerm = useDeleteTerm()

  useEffect(() => {
    if (!termId && activeTerm.data) {
      setTermId(activeTerm.data.id)
    }
  }, [activeTerm.data, termId])

  useEffect(() => {
    if (!classId && classes.data && classes.data.length > 0) {
      setClassId(classes.data[0].id)
    }
  }, [classes.data, classId])

  const classColumns: Column<Class>[] = [
    {
      key: "name",
      header: "Name",
      render: (klass) => (
        <span className="font-medium text-slate-900">{klass.name}</span>
      ),
    },
    {
      key: "grade",
      header: "Grade",
      hideOnSmall: true,
      render: (klass) => klass.gradeLevel,
    },
    {
      key: "capacity",
      header: "Capacity",
      align: "right",
      render: (klass) => klass.capacity,
    },
    {
      key: "actions",
      header: "",
      align: "right",
      render: (klass) => (
        <div
          className="flex justify-end gap-1"
          onClick={(event) => event.stopPropagation()}
        >
          <Button
            size="sm"
            variant="ghost"
            onClick={() => setEditingClass(klass)}
          >
            Edit
          </Button>
          <Button
            size="sm"
            variant="ghost"
            className="text-rose-600 hover:bg-rose-50 hover:text-rose-700"
            onClick={() => setDeletingClass(klass)}
          >
            Delete
          </Button>
        </div>
      ),
    },
  ]

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
        title="School Setup"
        description="Manage the classes and terms used for enrollment and billing."
      />

      <div className="grid gap-4 lg:grid-cols-2">
        <Card
          title="Classes"
          actions={
            <Button size="sm" variant="secondary" onClick={() => setAddingClass(true)}>
              Add class
            </Button>
          }
        >
          {classes.isLoading ? (
            <Loading />
          ) : classes.isError ? (
            <ErrorBanner error={classes.error} onRetry={() => classes.refetch()} />
          ) : (classes.data ?? []).length === 0 ? (
            <EmptyState
              title="No classes yet"
              description="Create a class before enrolling students."
              action={
                <Button onClick={() => setAddingClass(true)}>Add class</Button>
              }
            />
          ) : (
            <Table
              columns={classColumns}
              rows={classes.data ?? []}
              keyOf={(klass) => klass.id}
              onRowClick={(klass) => setClassId(klass.id)}
            />
          )}
        </Card>

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
              description="Create a term to start billing."
              action={
                <Button onClick={() => setAddingTerm(true)}>Add term</Button>
              }
            />
          ) : (
            <Table
              columns={termColumns}
              rows={terms.data ?? []}
              keyOf={(term) => term.id}
              onRowClick={(term) => setTermId(term.id)}
            />
          )}
        </Card>
      </div>

      <CreateClassModal open={addingClass} onClose={() => setAddingClass(false)} />
      <CreateClassModal
        open={editingClass !== null}
        editing={editingClass}
        onClose={() => setEditingClass(null)}
      />
      <CreateTermModal open={addingTerm} onClose={() => setAddingTerm(false)} />
      <CreateTermModal
        open={editingTerm !== null}
        editing={editingTerm}
        onClose={() => setEditingTerm(null)}
      />

      <ConfirmDialog
        open={deletingClass !== null}
        title="Delete class"
        loading={removeClass.isPending}
        error={removeClass.error}
        onCancel={() => {
          removeClass.reset()
          setDeletingClass(null)
        }}
        onConfirm={() => {
          if (!deletingClass) {
            return
          }
          removeClass.mutate(deletingClass.id, {
            onSuccess: () => {
              if (deletingClass.id === classId) {
                setClassId("")
              }
              setDeletingClass(null)
            },
          })
        }}
      >
        <p>
          Delete <strong>{deletingClass?.name}</strong>? This cannot be undone.
        </p>
        <p className="mt-2">
          A class that already has enrolments, fee structures or invoices cannot
          be deleted — rename it instead.
        </p>
      </ConfirmDialog>

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
              if (deletingTerm.id === termId) {
                setTermId("")
              }
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
