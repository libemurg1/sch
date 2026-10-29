import { useEffect, useMemo, useState } from "react"
import { Link, useNavigate } from "react-router-dom"
import {
  useActiveTerm,
  useAppInfo,
  useClasses,
  useEnrollmentsByTerm,
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
  Input,
  Loading,
  PageHeader,
  SearchablePicker,
  Select,
  StatusBadge,
  Table,
  YearSelect,
  type Column,
} from "../../components"
import { Page } from "../../app/AppShell"
import { useSession } from "../../app/session"
import { cx } from "../../lib/cx"
import { formatDate, formatTermLabel, studentName } from "../../lib/dates"
import { matchesSearch } from "../../lib/search"
import { ALL_YEARS, collectYears, defaultYear } from "../../lib/years"
import { publishActionNotice } from "../../lib/actionNotice"
import { exportStudentsCsv } from "../../lib/studentCsv"
import type { Student } from "../../types/schema/Student"
import { StudentFormModal } from "./StudentFormModal"
import { StudentImportModal } from "./StudentImportModal"

const STATUSES = ["ALL", "ACTIVE", "INACTIVE", "GRADUATED"]
type StudentView = "directory" | "roster"

interface RosterGroupData {
  id: string
  name: string
  description?: string
  students: Student[]
}

function sortStudents(students: Student[]) {
  return [...students].sort(
    (left, right) =>
      left.lastName.localeCompare(right.lastName) ||
      left.firstName.localeCompare(right.firstName),
  )
}

export function StudentsPage() {
  const { user } = useSession()
  const navigate = useNavigate()
  const appInfo = useAppInfo()
  const terms = useTerms()
  const activeTerm = useActiveTerm()
  const [view, setView] = useState<StudentView>("directory")
  const [year, setYear] = useState("")
  const [termId, setTermId] = useState("")
  const [status, setStatus] = useState("ALL")
  const [search, setSearch] = useState("")
  const [registering, setRegistering] = useState(false)
  const [importing, setImporting] = useState(false)
  const [confirmExport, setConfirmExport] = useState(false)

  const students = useStudents(status === "ALL" ? undefined : status)

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

  const yearTerms = useMemo(() => {
    const allTerms = terms.data ?? []
    return year === ALL_YEARS || year === ""
      ? allTerms
      : allTerms.filter((term) => term.year === Number(year))
  }, [terms.data, year])

  useEffect(() => {
    if (year === "") {
      return
    }
    if (termId && yearTerms.some((term) => term.id === termId)) {
      return
    }
    const preferred =
      activeTerm.data && yearTerms.some((term) => term.id === activeTerm.data?.id)
        ? activeTerm.data
        : yearTerms[0]
    setTermId(preferred?.id ?? "")
  }, [year, yearTerms, activeTerm.data, termId])

  const selectedTerm = (terms.data ?? []).find((term) => term.id === termId)
  const classes = useClasses(selectedTerm?.year)
  const enrollments = useEnrollmentsByTerm(selectedTerm?.id)

  const visibleStudents = useMemo(() => {
    const all = students.data ?? []
    if (!search.trim()) {
      return all
    }
    return all.filter((student) =>
      matchesSearch(
        search,
        [
          student.studentNumber,
          student.firstName,
          student.lastName,
          student.guardianName,
          student.guardianPhone,
          student.guardianEmail,
        ]
          .filter(Boolean)
          .join(" "),
      ),
    )
  }, [students.data, search])

  const roster = useMemo(() => {
    const studentsById = new Map(
      visibleStudents.map((student) => [student.id, student]),
    )
    const studentsByClass = new Map<string, Student[]>()
    const enrolledStudentIds = new Set<string>()

    for (const enrollment of enrollments.data ?? []) {
      const student = studentsById.get(enrollment.studentId)
      if (!student) {
        continue
      }
      enrolledStudentIds.add(student.id)
      const bucket = studentsByClass.get(enrollment.classId) ?? []
      bucket.push(student)
      studentsByClass.set(enrollment.classId, bucket)
    }

    const groups = (classes.data ?? [])
      .map((klass) => ({
        id: klass.id,
        name: klass.name,
        students: sortStudents(studentsByClass.get(klass.id) ?? []),
      }))
      .filter((group) => group.students.length > 0)

    const unassigned = sortStudents(
      visibleStudents.filter((student) => !enrolledStudentIds.has(student.id)),
    )

    return { groups, unassigned }
  }, [classes.data, enrollments.data, visibleStudents])

  const columns: Column<Student>[] = [
    {
      key: "number",
      header: "Number",
      render: (student) => (
        <span className="font-mono text-xs text-slate-500">
          {student.studentNumber}
        </span>
      ),
    },
    {
      key: "name",
      header: "Name",
      render: (student) => (
        <span className="font-medium text-slate-900">
          {studentName(student)}
        </span>
      ),
    },
    {
      key: "gender",
      header: "Gender",
      hideOnSmall: true,
      render: (student) => student.gender ?? "-",
    },
    {
      key: "dob",
      header: "Date of birth",
      hideOnSmall: true,
      render: (student) => formatDate(student.dob),
    },
    {
      key: "guardian",
      header: "Guardian",
      hideOnSmall: true,
      render: (student) => student.guardianName ?? "-",
    },
    {
      key: "status",
      header: "Status",
      align: "right",
      render: (student) => <StatusBadge status={student.status} />,
    },
  ]

  const directoryColumns: Column<Student>[] = [
    {
      key: "number",
      header: "Student number",
      render: (student) => (
        <span className="font-mono text-xs text-slate-500">
          {student.studentNumber}
        </span>
      ),
    },
    {
      key: "name",
      header: "Student",
      render: (student) => (
        <span className="font-medium text-slate-900">
          {studentName(student)}
        </span>
      ),
    },
    {
      key: "guardian",
      header: "Guardian",
      hideOnSmall: true,
      render: (student) =>
        [student.guardianName, student.guardianPhone].filter(Boolean).join(" · ") || "-",
    },
    {
      key: "admission",
      header: "Admitted",
      hideOnSmall: true,
      render: (student) => formatDate(student.admissionDate),
    },
    {
      key: "status",
      header: "Status",
      align: "right",
      render: (student) => <StatusBadge status={student.status} />,
    },
    {
      key: "record",
      header: "",
      align: "right",
      render: (student) => (
        <Link
          to={`/students/${student.id}`}
          className="font-medium text-indigo-600 hover:text-indigo-800"
        >
          View record
        </Link>
      ),
    },
  ]

  const loading =
    terms.isLoading ||
    activeTerm.isLoading ||
    students.isLoading ||
    (view === "roster" &&
      selectedTerm !== undefined &&
      (classes.isLoading || enrollments.isLoading))
  const error =
    students.error ??
    (view === "roster"
      ? terms.error ?? activeTerm.error ?? classes.error ?? enrollments.error
      : null)
  const hasTerms = (terms.data ?? []).length > 0
  const hasClasses = (classes.data ?? []).length > 0
  const unassignedDescription = selectedTerm
    ? `No enrollment in ${formatTermLabel(selectedTerm)}.`
    : undefined
  const description = selectedTerm
    ? `${visibleStudents.length} student${
        visibleStudents.length === 1 ? "" : "s"
      } in ${formatTermLabel(selectedTerm)}`
    : students.data
      ? `${students.data.length} student${students.data.length === 1 ? "" : "s"}`
      : undefined

  const viewSwitcher = (
    <div
      className="mb-4 inline-flex rounded-md bg-slate-100 p-1"
      aria-label="Student view"
    >
      <button
        type="button"
        aria-pressed={view === "directory"}
        onClick={() => setView("directory")}
        className={cx(
          "rounded px-3 py-1.5 text-sm font-medium",
          view === "directory"
            ? "bg-white text-indigo-700 shadow-sm"
            : "text-slate-600 hover:text-slate-900",
        )}
      >
        Find a student
      </button>
      <button
        type="button"
        aria-pressed={view === "roster"}
        onClick={() => setView("roster")}
        className={cx(
          "rounded px-3 py-1.5 text-sm font-medium",
          view === "roster"
            ? "bg-white text-indigo-700 shadow-sm"
            : "text-slate-600 hover:text-slate-900",
        )}
      >
        Class roster by term
      </button>
    </div>
  )
  const exportFilteredStudents = () => {
    const date = appInfo.data?.today ?? new Date().toISOString().slice(0, 10)
    const url = URL.createObjectURL(
      new Blob([exportStudentsCsv(visibleStudents)], {
        type: "text/csv;charset=utf-8",
      }),
    )
    const link = document.createElement("a")
    link.href = url
    link.download = `students-${date}.csv`
    link.click()
    window.setTimeout(() => URL.revokeObjectURL(url), 0)
    setConfirmExport(false)
    publishActionNotice(
      `Exported ${visibleStudents.length} student record${visibleStudents.length === 1 ? "" : "s"} to CSV.`,
    )
  }

  return (
    <Page>
      <PageHeader
        title="Students"
        description={
          view === "directory"
            ? `${visibleStudents.length} of ${students.data?.length ?? 0} students · search across all years`
            : selectedTerm
              ? `Class rosters for ${formatTermLabel(selectedTerm)}`
              : description
        }
        actions={
          user?.role === "SCHOOL_ADMINISTRATOR" ? (
            <div className="flex flex-wrap gap-2">
              <Button onClick={() => setRegistering(true)}>Register student</Button>
              {view === "directory" ? (
                <>
                  <Button
                    variant="secondary"
                    onClick={() => setImporting(true)}
                  >
                    Import students
                  </Button>
                  <Button
                    variant="secondary"
                    onClick={() => setConfirmExport(true)}
                    disabled={visibleStudents.length === 0}
                  >
                    Export filtered students
                  </Button>
                </>
              ) : null}
            </div>
          ) : null
        }
      />

      <Card>
        {loading ? (
          <Loading />
        ) : error ? (
          <ErrorBanner
            error={error}
            onRetry={() => {
              terms.refetch()
              activeTerm.refetch()
              students.refetch()
              classes.refetch()
              enrollments.refetch()
            }}
          />
        ) : view === "roster" && !hasTerms ? (
          <>
            {viewSwitcher}
            <EmptyState
              title="No terms yet"
              description="Create a term before grouping students by class."
              action={
                <Link
                  to="/terms"
                  className="text-sm font-medium text-indigo-600 hover:text-indigo-800"
                >
                  Manage terms
                </Link>
              }
            />
          </>
        ) : (
          <>
            {viewSwitcher}
            <div className="mb-4 flex flex-wrap items-end gap-3">
              {view === "roster" ? (
                <>
                  <YearSelect
                    id="student-year"
                    value={year}
                    onChange={setYear}
                    years={years}
                  />
                  <div className="w-56">
                    <label
                      htmlFor="student-term"
                      className="mb-1 block text-sm font-medium text-slate-700"
                    >
                      Term
                    </label>
                    <SearchablePicker
                      id="student-term"
                      value={termId}
                      onChange={setTermId}
                      options={yearTerms.map((term) => ({
                        id: term.id,
                        label: formatTermLabel(term),
                      }))}
                      placeholder="Choose a term"
                      searchPlaceholder="Search terms"
                    />
                  </div>
                </>
              ) : null}
              <div className="min-w-56 flex-1">
                <label
                  htmlFor="student-search"
                  className="mb-1 block text-sm font-medium text-slate-700"
                >
                  Search
                </label>
                <Input
                  id="student-search"
                  value={search}
                  placeholder={
                    view === "directory"
                      ? "Search all students by name, number, or guardian"
                      : "Name, number or guardian"
                  }
                  onChange={(event) => setSearch(event.target.value)}
                />
              </div>
              <div className="w-44">
                <label
                  htmlFor="student-status"
                  className="mb-1 block text-sm font-medium text-slate-700"
                >
                  Status
                </label>
                <Select
                  id="student-status"
                  value={status}
                  onChange={(event) => setStatus(event.target.value)}
                >
                  {STATUSES.map((value) => (
                    <option key={value} value={value}>
                      {value === "ALL" ? "All statuses" : value}
                    </option>
                  ))}
                </Select>
              </div>
            </div>

            {view === "directory" ? (
              visibleStudents.length === 0 ? (
                <EmptyState
                  title={search ? "No students found" : "No students yet"}
                  description={
                    search
                      ? "Try another name, student number, or guardian detail."
                      : "Register a student to create the first permanent student record."
                  }
                  action={
                    user?.role === "SCHOOL_ADMINISTRATOR" ? (
                      <Button onClick={() => setRegistering(true)}>
                        Register student
                      </Button>
                    ) : null
                  }
                />
              ) : (
                <>
                  <p className="mb-3 text-xs text-slate-500">
                    Search includes former and current students. Open a record
                    to review the student&apos;s full enrollment history.
                  </p>
                  <Table
                    columns={directoryColumns}
                    rows={sortStudents(visibleStudents)}
                    keyOf={(student) => student.id}
                  />
                </>
              )
            ) : !selectedTerm ? (
              <EmptyState
                title="Choose a term"
                description="Students are grouped by class for one term at a time."
              />
            ) : !hasClasses ? (
              <EmptyState
                title="No classes in this year"
                description="Create classes for this academic year before enrolling and grouping students."
                action={
                  <Link
                    to="/classes"
                    className="text-sm font-medium text-indigo-600 hover:text-indigo-800"
                  >
                    Manage classes
                  </Link>
                }
              />
            ) : visibleStudents.length === 0 ? (
              <EmptyState
                title="No students found"
                description={
                  search
                    ? "No student matches that search."
                    : "Register a student to get started."
                }
                action={
                  <Button onClick={() => setRegistering(true)}>
                    Register student
                  </Button>
                }
              />
            ) : (
              <div className="space-y-3">
                {roster.groups.map((group) => (
                  <RosterGroup
                    key={`${selectedTerm.id}:${group.id}`}
                    group={group}
                    columns={columns}
                    onSelect={(student) => navigate(`/students/${student.id}`)}
                  />
                ))}
                {roster.unassigned.length > 0 ? (
                  <RosterGroup
                    key={`${selectedTerm.id}:unassigned`}
                    group={{
                      id: "unassigned",
                      name: "Not enrolled in this term",
                      description: unassignedDescription,
                      students: roster.unassigned,
                    }}
                    columns={columns}
                    onSelect={(student) => navigate(`/students/${student.id}`)}
                    defaultOpen={false}
                  />
                ) : null}
              </div>
            )}
          </>
        )}
      </Card>

      <StudentFormModal
        open={registering}
        onClose={() => setRegistering(false)}
        today={appInfo.data?.today ?? ""}
      />
      <StudentImportModal
        open={importing}
        onClose={() => setImporting(false)}
        students={students.data ?? []}
      />
      <ConfirmDialog
        open={confirmExport}
        title="Export student details"
        confirmLabel="Export CSV"
        onCancel={() => setConfirmExport(false)}
        onConfirm={exportFilteredStudents}
      >
        <p>
          This exports {visibleStudents.length} filtered student records,
          including dates of birth and guardian contact details. Store the CSV
          securely and share it only with authorized people.
        </p>
      </ConfirmDialog>
    </Page>
  )
}

function RosterGroup({
  group,
  columns,
  onSelect,
  defaultOpen = true,
}: {
  group: RosterGroupData
  columns: Column<Student>[]
  onSelect: (student: Student) => void
  defaultOpen?: boolean
}) {
  const [open, setOpen] = useState(defaultOpen)

  return (
    <section className="overflow-hidden rounded-lg border border-slate-200 bg-white">
      <button
        type="button"
        aria-expanded={open}
        onClick={() => setOpen((current) => !current)}
        className="flex w-full items-center justify-between gap-4 px-4 py-3 text-left transition-colors hover:bg-slate-50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-slate-500 sm:px-5"
      >
        <span className="min-w-0">
          <span className="flex items-center gap-2">
            <span className="text-sm font-semibold text-slate-900">
              {group.name}
            </span>
            <Badge>{group.students.length}</Badge>
          </span>
          {group.description ? (
            <span className="mt-0.5 block text-xs text-slate-500">
              {group.description}
            </span>
          ) : null}
        </span>
        <svg
          className={cx(
            "h-4 w-4 shrink-0 text-slate-400 transition-transform",
            open ? "rotate-180" : null,
          )}
          viewBox="0 0 20 20"
          fill="currentColor"
          aria-hidden="true"
        >
          <path
            fillRule="evenodd"
            d="M5.23 7.21a.75.75 0 0 1 1.06.02L10 11.168l3.71-3.938a.75.75 0 1 1 1.08 1.04l-4.25 4.5a.75.75 0 0 1-1.08 0l-4.25-4.5a.75.75 0 0 1 .02-1.06Z"
            clipRule="evenodd"
          />
        </svg>
      </button>

      {open ? (
        <div className="border-t border-slate-200 px-4 pb-1 pt-2 sm:px-5">
          <Table
            columns={columns}
            rows={group.students}
            keyOf={(student) => student.id}
            onRowClick={onSelect}
          />
        </div>
      ) : null}
    </section>
  )
}
