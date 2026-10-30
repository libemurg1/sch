import { useEffect, useState } from "react"
import { useEnrollStudent, useStudents } from "../../api/queries"
import {
  Button,
  ErrorBanner,
  Field,
  Modal,
  StudentPicker,
} from "../../components"

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
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (open) {
      setStudentId("")
      setError(null)
      mutation.reset()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open])

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

        <Field
          label="Student"
          htmlFor="enrol-student"
          required
          error={error ?? undefined}
          hint="A student can only be enrolled in one class per term."
        >
          <StudentPicker
            id="enrol-student"
            value={studentId}
            onChange={(value) => {
              setStudentId(value)
              setError(null)
            }}
            students={students.data ?? []}
          />
        </Field>
      </div>
    </Modal>
  )
}
