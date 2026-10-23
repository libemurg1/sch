import { useMemo, useState } from "react"
import { useNavigate } from "react-router-dom"
import { useAppInfo, useStudents } from "../../api/queries"
import {
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
  type Column,
} from "../../components"
import { Page } from "../../app/AppShell"
import { formatDate, studentName } from "../../lib/dates"
import type { Student } from "../../types/schema/Student"
import { RegisterStudentModal } from "./RegisterStudentModal"

const STATUSES = ["ALL", "ACTIVE", "INACTIVE", "GRADUATED"]

export function StudentsPage() {
  const navigate = useNavigate()
  const appInfo = useAppInfo()
  const [status, setStatus] = useState("ALL")
  const [search, setSearch] = useState("")
  const [registering, setRegistering] = useState(false)

  const students = useStudents(status === "ALL" ? undefined : status)

  const rows = useMemo(() => {
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

  return (
    <Page>
      <PageHeader
        title="Students"
        description={
          students.data
            ? `${students.data.length} student${students.data.length === 1 ? "" : "s"}`
            : undefined
        }
        actions={
          <Button onClick={() => setRegistering(true)}>Register student</Button>
        }
      />

      <Card>
        <div className="mb-4 flex flex-wrap items-end gap-3">
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

        {students.isLoading ? (
          <Loading />
        ) : students.isError ? (
          <ErrorBanner error={students.error} onRetry={() => students.refetch()} />
        ) : rows.length === 0 ? (
          <EmptyState
            title="No students found"
            description={
              search
                ? "No student matches that search."
                : "Register a student to get started."
            }
            action={
              <Button onClick={() => setRegistering(true)}>Register student</Button>
            }
          />
        ) : (
          <Table
            columns={columns}
            rows={rows}
            keyOf={(student) => student.id}
            onRowClick={(student) => navigate(`/students/${student.id}`)}
          />
        )}
      </Card>

      <RegisterStudentModal
        open={registering}
        onClose={() => setRegistering(false)}
        today={appInfo.data?.today ?? ""}
      />
    </Page>
  )
}
