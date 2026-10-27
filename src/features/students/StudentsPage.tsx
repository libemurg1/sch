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
  EmptyState,
  ErrorBanner,
  Input,
  Loading,
  PageHeader,
  Select,
  StatusBadge,
  Table,
  YearSelect,
  type Column,
} from "../../components"
import { Page } from "../../app/AppShell"
import { cx } from "../../lib/cx"
import { formatDate, formatTermLabel, studentName } from "../../lib/dates"
import { ALL_YEARS, collectYears, defaultYear } from "../../lib/years"
import type { Student } from "../../types/schema/Student"
import { StudentFormModal } from "./StudentFormModal"

const STATUSES = ["ALL", "ACTIVE", "INACTIVE", "GRADUATED"]

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
  const navigate = useNavigate()
  const appInfo = useAppInfo()
  const terms = useTerms()
  const activeTerm = useActiveTerm()
  const [year, setYear] = useState("")
  const [termId, setTermId] = useState("")
  const [status, setStatus] = useState("ALL")
  const [search, setSearch] = useState("")
  const [registering, setRegistering] = useState(false)

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
    const term = search.trim().toLowerCase()
    const all = students.data ?? []
    if (term === "") {
      return all
    }
    return all.filter((student) =>
      [
        student.studentNumber,
        student.firstName,
        student.lastName,
        student.guardianName ?? "",
        student.guardianPhone ?? "",
      ]
        .join(" ")
        .toLowerCase()
        .includes(term),
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

  const loading =
    terms.isLoading ||
    activeTerm.isLoading ||
    students.isLoading ||
    (selectedTerm !== undefined &&
      (classes.isLoading || enrollments.isLoading))
  const error =
    terms.error ??
    activeTerm.error ??
    students.error ??
    classes.error ??
    enrollments.error
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

  return (
    <Page>
      <PageHeader
        title="Students"
        description={description}
        actions={
          <Button onClick={() => setRegistering(true)}>Register student</Button>
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
        ) : !hasTerms ? (
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
        ) : (
          <>
            <div className="mb-4 flex flex-wrap items-end gap-3">
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
                <Select
                  id="student-term"
                  value={termId}
                  onChange={(event) => setTermId(event.target.value)}
                >
                  {yearTerms.map((term) => (
                    <option key={term.id} value={term.id}>
                      {formatTermLabel(term)}
                    </option>
                  ))}
                </Select>
              </div>
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
                  placeholder="Name, number or guardian"
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

            {!selectedTerm ? (
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
