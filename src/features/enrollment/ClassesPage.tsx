import { useEffect, useMemo, useState } from "react"
import {
  useActiveTerm,
  useClasses,
  useDeleteClass,
  useGrades,
} from "../../api/queries"
import {
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
import { ALL_YEARS, collectYears, defaultYear } from "../../lib/years"
import type { Class } from "../../types/schema/Class"
import { CreateClassModal } from "./CreateClassModal"

export function ClassesPage() {
  const classes = useClasses()
  const grades = useGrades()
  const activeTerm = useActiveTerm()
  const [year, setYear] = useState("")
  const [addingClass, setAddingClass] = useState(false)
  const [editingClass, setEditingClass] = useState<Class | null>(null)
  const [deletingClass, setDeletingClass] = useState<Class | null>(null)
  const removeClass = useDeleteClass()

  const years = useMemo(
    () =>
      collectYears([
        ...(classes.data ?? []).map((klass) => klass.academicYear),
        activeTerm.data?.year,
      ]),
    [classes.data, activeTerm.data],
  )

  useEffect(() => {
    if (year !== "") {
      return
    }
    if (classes.isLoading || activeTerm.isLoading) {
      return
    }
    setYear(defaultYear(activeTerm.data?.year, years))
  }, [year, classes.isLoading, activeTerm.isLoading, activeTerm.data, years])

  const allClasses = classes.data ?? []
  const rows =
    year === ALL_YEARS || year === ""
      ? allClasses
      : allClasses.filter((klass) => klass.academicYear === Number(year))

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
      render: (klass) =>
        grades.data?.find((grade) => grade.level === klass.gradeLevel)?.name ??
        klass.gradeLevel,
    },
    {
      key: "stream",
      header: "Stream",
      hideOnSmall: true,
      render: (klass) => klass.stream || "-",
    },
    {
      key: "year",
      header: "Year",
      hideOnSmall: true,
      render: (klass) => klass.academicYear,
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
        description="Classes are grouped by grade, stream and academic year."
      />

      <Card
        title="Classes"
        actions={
          <Button size="sm" variant="secondary" onClick={() => setAddingClass(true)}>
            Add class
          </Button>
        }
      >
        <div className="mb-4 flex flex-wrap gap-3">
          <YearSelect
            id="class-year-filter"
            value={year}
            onChange={setYear}
            years={years}
          />
        </div>

        {classes.isLoading ? (
          <Loading />
        ) : classes.isError ? (
          <ErrorBanner error={classes.error} onRetry={() => classes.refetch()} />
        ) : rows.length === 0 ? (
          <EmptyState
            title={allClasses.length === 0 ? "No classes yet" : "No classes in this year"}
            description={
              allClasses.length === 0
                ? "Create a classroom grouping before enrolling students."
                : "Choose another year or add a class for this year."
            }
            action={<Button onClick={() => setAddingClass(true)}>Add class</Button>}
          />
        ) : (
          <Table
            columns={classColumns}
            rows={rows}
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
