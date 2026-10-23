import { useEffect, useMemo, useState } from "react"
import { useEnrollStudent, useStudents } from "../../api/queries"
import { Button, ErrorBanner, Field, Input, Modal, Select } from "../../components"
import { studentName } from "../../lib/dates"

export interface EnrollStudentModalProps {
  open: boolean
  onClose: () => void
  classId: string
  termId: string
  className: string
  termLabel: string
}

export function EnrollStudentModal({
  open,
  onClose,
  classId,
  termId,
  className,
  termLabel,
}: EnrollStudentModalProps) {
  const students = useStudents("ACTIVE")
  const mutation = useEnrollStudent()
  const [studentId, setStudentId] = useState("")
  const [search, setSearch] = useState("")
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (open) {
      setStudentId("")
      setSearch("")
      setError(null)
      mutation.reset()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open])

  const options = useMemo(() => {
    const term = search.trim().toLowerCase()
    const all = students.data ?? []
    if (term === "") {
      return all
    }
    return all.filter((student) =>
      `${student.studentNumber} ${studentName(student)}`
        .toLowerCase()
        .includes(term),
    )
  }, [students.data, search])

  const submit = () => {
    if (!studentId) {
      setError("Choose a student to enrol")
      return
    }
    setError(null)
    mutation.mutate(
      { studentId, classId, termId },
      { onSuccess: onClose },
    )
  }

  return (
    <Modal
      open={open}
      onClose={onClose}
      title="Enrol a student"
      description={`${className} · ${termLabel}`}
      footer={
        <>
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={submit} loading={mutation.isPending}>
            Enrol
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        {mutation.isError ? <ErrorBanner error={mutation.error} /> : null}

        <Field label="Find a student" htmlFor="enrol-search">
          <Input
            id="enrol-search"
            value={search}
            placeholder="Name or student number"
            onChange={(event) => setSearch(event.target.value)}
          />
        </Field>

        <Field
          label="Student"
          htmlFor="enrol-student"
          required
          error={error ?? undefined}
          hint="A student can only be enrolled in one class per term."
        >
          <Select
            id="enrol-student"
            value={studentId}
            onChange={(event) => {
              setStudentId(event.target.value)
              setError(null)
            }}
          >
            <option value="">Choose a student</option>
            {options.map((student) => (
              <option key={student.id} value={student.id}>
                {student.studentNumber} - {studentName(student)}
              </option>
            ))}
          </Select>
        </Field>
      </div>
    </Modal>
  )
}
