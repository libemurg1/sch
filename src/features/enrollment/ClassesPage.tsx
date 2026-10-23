import { useState } from "react"
import { useClasses, useDeleteClass } from "../../api/queries"
import {
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
import type { Class } from "../../types/schema/Class"
import { CreateClassModal } from "./CreateClassModal"

export function ClassesPage() {
  const classes = useClasses()
  const [addingClass, setAddingClass] = useState(false)
  const [editingClass, setEditingClass] = useState<Class | null>(null)
  const [deletingClass, setDeletingClass] = useState<Class | null>(null)
  const removeClass = useDeleteClass()

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

  return (
    <Page>
      <PageHeader
        title="Classes"
        description="Manage the classes students can be enrolled into."
      />

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
            description="Create a classroom grouping before enrolling students."
            action={<Button onClick={() => setAddingClass(true)}>Add class</Button>}
          />
        ) : (
          <Table
            columns={classColumns}
            rows={classes.data ?? []}
            keyOf={(klass) => klass.id}
          />
        )}
      </Card>

      <CreateClassModal open={addingClass} onClose={() => setAddingClass(false)} />
      <CreateClassModal
        open={editingClass !== null}
        editing={editingClass}
        onClose={() => setEditingClass(null)}
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
    </Page>
  )
}
