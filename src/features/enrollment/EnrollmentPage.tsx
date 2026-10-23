import { useEffect, useState } from "react"
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
  type Column,
} from "../../components"
import { Page } from "../../app/AppShell"
import { formatTermLabel, studentName } from "../../lib/dates"
import type { Enrollment } from "../../types/schema/Enrollment"
import { EnrollStudentModal } from "./EnrollStudentModal"

export function EnrollmentPage() {
  const classes = useClasses()
  const terms = useTerms()
  const activeTerm = useActiveTerm()
  const students = useStudents()

  const [classId, setClassId] = useState("")
  const [termId, setTermId] = useState("")
  const [enrolling, setEnrolling] = useState(false)

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

  const enrollments = useEnrollments(classId || undefined, termId || undefined)
  const selectedClass = (classes.data ?? []).find((item) => item.id === classId)
  const selectedTerm = (terms.data ?? []).find((item) => item.id === termId)

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
      align: "right",
      render: (enrollment) => <StatusBadge status={enrollment.status} />,
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
        description="A student belongs to one class per term."
        actions={
          <Button
            size="sm"
            disabled={!classId || !termId}
            onClick={() => setEnrolling(true)}
          >
            Enrol student
          </Button>
        }
      >
        <div className="mb-4 flex flex-wrap gap-3">
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
              options={(terms.data ?? []).map((term) => ({
                id: term.id,
                label: formatTermLabel(term),
              }))}
              placeholder="Choose a term"
              searchPlaceholder="Search terms"
            />
          </div>
        </div>

        {!classId || !termId ? (
          <EmptyState title="Choose a class and a term" />
        ) : enrollments.isLoading ? (
          <Loading />
        ) : enrollments.isError ? (
          <ErrorBanner error={enrollments.error} />
        ) : (enrollments.data ?? []).length === 0 ? (
          <EmptyState
            title="Nobody enrolled yet"
            description="Enrol students so they can be billed for this term."
          />
        ) : (
          <Table
            columns={enrollmentColumns}
            rows={enrollments.data ?? []}
            keyOf={(enrollment) => enrollment.id}
          />
        )}
      </Card>

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
