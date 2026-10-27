import { useEffect, useMemo, useState } from "react"
import {
  useActiveTerm,
  useClasses,
  useEnrollments,
  useStudents,
  useTerms,
} from "../../api/queries"
import {
  Button,
  Card,
  EmptyState,
  ErrorBanner,
  Loading,
  PageHeader,
  SearchablePicker,
  StatusBadge,
  Table,
  YearSelect,
  type Column,
} from "../../components"
import { Page } from "../../app/AppShell"
import { formatTermLabel, studentName } from "../../lib/dates"
import { ALL_YEARS, collectYears, defaultYear } from "../../lib/years"
import type { Enrollment } from "../../types/schema/Enrollment"
import { EnrollStudentModal } from "./EnrollStudentModal"
import { MoveStudentModal } from "./MoveStudentModal"

export function EnrollmentPage() {
  const terms = useTerms()
  const activeTerm = useActiveTerm()
  const students = useStudents()

  const [year, setYear] = useState("")
  const [classId, setClassId] = useState("")
  const [termId, setTermId] = useState("")
  const [enrolling, setEnrolling] = useState(false)
  const [moving, setMoving] = useState<Enrollment | null>(null)

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

  const selectedTerm = yearTerms.find((item) => item.id === termId)

  useEffect(() => {
    if (year === "") {
      return
    }
    if (termId && yearTerms.some((item) => item.id === termId)) {
      return
    }
    const preferred =
      activeTerm.data &&
      yearTerms.some((item) => item.id === activeTerm.data?.id)
        ? activeTerm.data
        : yearTerms[0]
    setTermId(preferred?.id ?? "")
  }, [year, yearTerms, activeTerm.data, termId])

  const classYear =
    selectedTerm?.year ??
    (year !== ALL_YEARS && year !== "" ? Number(year) : undefined)
  const classes = useClasses(classYear)

  useEffect(() => {
    if (!classId && classes.data && classes.data.length > 0) {
      setClassId(classes.data[0].id)
    }
  }, [classes.data, classId])

  useEffect(() => {
    if (
      classId &&
      classes.data &&
      !classes.data.some((item) => item.id === classId)
    ) {
      setClassId(classes.data[0]?.id ?? "")
    }
  }, [classes.data, classId])

  const enrollments = useEnrollments(classId || undefined, termId || undefined)
  const selectedClass = (classes.data ?? []).find((item) => item.id === classId)

  const studentNames = new Map(
    (students.data ?? []).map((student) => [student.id, studentName(student)]),
  )

  const enrollmentColumns: Column<Enrollment>[] = [
    {
      key: "student",
      header: "Student",
      render: (enrollment) => (
        <span className="font-medium text-slate-900">
          {studentNames.get(enrollment.studentId) ?? enrollment.studentId}
        </span>
      ),
    },
    {
      key: "status",
      header: "Status",
      render: (enrollment) => <StatusBadge status={enrollment.status} />,
    },
    {
      key: "actions",
      header: "",
      align: "right",
      render: (enrollment) => (
        <Button
          size="sm"
          variant="ghost"
          disabled={enrollment.status !== "ACTIVE"}
          onClick={() => setMoving(enrollment)}
        >
          Move
        </Button>
      ),
    },
  ]

  return (
    <Page>
      <PageHeader
        title="Enrollment"
        description="Manage which students are enrolled in each class for a term."
      />

      <Card
        title="Enrollments"
        description={
          selectedTerm && !selectedTerm.isActive
            ? "This is a past term. Its roster is available for viewing, but new enrolments are disabled."
            : "A student belongs to one class per term."
        }
        actions={
          <Button
            size="sm"
            disabled={!classId || !termId || !selectedTerm?.isActive}
            onClick={() => setEnrolling(true)}
          >
            Enrol student
          </Button>
        }
      >
        <div className="mb-4 flex flex-wrap gap-3">
          <YearSelect
            id="enrol-year"
            value={year}
            onChange={setYear}
            years={years}
          />
          <div className="w-56">
            <label
              htmlFor="enrol-term"
              className="mb-1 block text-sm font-medium text-slate-700"
            >
              Term
            </label>
            <SearchablePicker
              id="enrol-term"
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
          <div className="w-56">
            <label
              htmlFor="enrol-class"
              className="mb-1 block text-sm font-medium text-slate-700"
            >
              Class
            </label>
            <SearchablePicker
              id="enrol-class"
              value={classId}
              onChange={setClassId}
              options={(classes.data ?? []).map((klass) => ({
                id: klass.id,
                label: klass.name,
              }))}
              placeholder="Choose a class"
              searchPlaceholder="Search classes"
            />
          </div>
        </div>

        {!classId || !termId ? (
          <EmptyState title="Choose a term and a class" />
        ) : enrollments.isLoading ? (
          <Loading />
        ) : enrollments.isError ? (
          <ErrorBanner error={enrollments.error} />
        ) : (enrollments.data ?? []).length === 0 ? (
          <EmptyState
            title="No students in this class"
            description={
              selectedTerm?.isActive
                ? "Enrol students so they can be billed for this term."
                : "No students were enrolled in this class for this term."
            }
          />
        ) : (
          <Table
            columns={enrollmentColumns}
            rows={enrollments.data ?? []}
            keyOf={(enrollment) => enrollment.id}
          />
        )}
      </Card>

      {moving && selectedTerm ? (
        <MoveStudentModal
          open
          onClose={() => setMoving(null)}
          studentId={moving.studentId}
          studentLabel={
            studentNames.get(moving.studentId) ?? moving.studentId
          }
          termId={selectedTerm.id}
          termYear={selectedTerm.year}
          currentClassId={moving.classId}
        />
      ) : null}

      {selectedClass && selectedTerm ? (
        <EnrollStudentModal
          open={enrolling}
          onClose={() => setEnrolling(false)}
          classId={selectedClass.id}
          termId={selectedTerm.id}
          className={selectedClass.name}
          termLabel={formatTermLabel(selectedTerm)}
        />
      ) : null}
    </Page>
  )
}
