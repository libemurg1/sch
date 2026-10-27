import { useEffect, useState } from "react"
import { z } from "zod"
import { useRegisterStudent, useUpdateStudent } from "../../api/queries"
import { Button, ErrorBanner, Field, Input, Modal, Select } from "../../components"
import type { RegisterStudentInput } from "../../types/schema/RegisterStudentInput"
import type { Student } from "../../types/schema/Student"
import type { UpdateStudentInput } from "../../types/schema/UpdateStudentInput"

const schema = z.object({
  firstName: z.string().trim().min(1, "First name is required"),
  lastName: z.string().trim().min(1, "Last name is required"),
  dob: z.string().min(1, "Date of birth is required"),
  gender: z.string(),
  admissionDate: z.string().min(1, "Admission date is required"),
  guardianName: z.string(),
  guardianPhone: z.string(),
  guardianEmail: z.string(),
})

type FormValues = z.infer<typeof schema>

const EMPTY: FormValues = {
  firstName: "",
  lastName: "",
  dob: "",
  gender: "",
  admissionDate: "",
  guardianName: "",
  guardianPhone: "",
  guardianEmail: "",
}

const GENDERS = ["", "Female", "Male", "Other"]

/** The empty string means "not given", which the backend stores as null. */
function optional(value: string): string | null {
  const trimmed = value.trim()
  return trimmed === "" ? null : trimmed
}

function fromStudent(student: Student): FormValues {
  return {
    firstName: student.firstName,
    lastName: student.lastName,
    dob: student.dob,
    gender: student.gender ?? "",
    admissionDate: student.admissionDate,
    guardianName: student.guardianName ?? "",
    guardianPhone: student.guardianPhone ?? "",
    guardianEmail: student.guardianEmail ?? "",
  }
}

export interface StudentFormModalProps {
  open: boolean
  onClose: () => void
  /** The server's current date, used as the default admission date. */
  today?: string
  /** When set, the form edits this student instead of registering a new one. */
  student?: Student | null
}

export function StudentFormModal({
  open,
  onClose,
  today,
  student,
}: StudentFormModalProps) {
  const [values, setValues] = useState<FormValues>(EMPTY)
  const [errors, setErrors] = useState<Partial<Record<keyof FormValues, string>>>(
    {},
  )
  const register = useRegisterStudent()
  const update = useUpdateStudent()
  const mutation = student ? update : register

  useEffect(() => {
    if (open) {
      setValues(
        student
          ? fromStudent(student)
          : { ...EMPTY, admissionDate: today ?? "" },
      )
      setErrors({})
      register.reset()
      update.reset()
    }
    // Only re-run when the dialog opens or switches student. Including the
    // whole student object would wipe edits after a background refetch.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, student?.id, today])

  const set = (key: keyof FormValues) => (value: string) => {
    setValues((current) => ({ ...current, [key]: value }))
    setErrors((current) => ({ ...current, [key]: undefined }))
  }

  const submit = () => {
    const parsed = schema.safeParse(values)
    if (!parsed.success) {
      const next: Partial<Record<keyof FormValues, string>> = {}
      for (const issue of parsed.error.issues) {
        const key = issue.path[0] as keyof FormValues
        if (!next[key]) {
          next[key] = issue.message
        }
      }
      setErrors(next)
      return
    }

    const details = {
      firstName: parsed.data.firstName,
      lastName: parsed.data.lastName,
      dob: parsed.data.dob,
      gender: optional(parsed.data.gender),
      admissionDate: parsed.data.admissionDate,
      guardianName: optional(parsed.data.guardianName),
      guardianPhone: optional(parsed.data.guardianPhone),
      guardianEmail: optional(parsed.data.guardianEmail),
    }

    if (student) {
      const input: UpdateStudentInput = {
        studentId: student.id,
        ...details,
      }
      update.mutate(input, { onSuccess: onClose })
      return
    }

    const input: RegisterStudentInput = details
    register.mutate(input, { onSuccess: onClose })
  }

  return (
    <Modal
      open={open}
      onClose={onClose}
      title={student ? "Edit student" : "Register a student"}
      description={
        student
          ? `Student number ${student.studentNumber} cannot be changed.`
          : "The student number is allocated automatically."
      }
      footer={
        <>
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={submit} loading={mutation.isPending}>
            {student ? "Save changes" : "Register student"}
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        {mutation.isError ? <ErrorBanner error={mutation.error} /> : null}

        <div className="grid gap-4 sm:grid-cols-2">
          <Field label="First name" htmlFor="firstName" required error={errors.firstName}>
            <Input
              id="firstName"
              value={values.firstName}
              onChange={(event) => set("firstName")(event.target.value)}
              error={errors.firstName}
            />
          </Field>
          <Field label="Last name" htmlFor="lastName" required error={errors.lastName}>
            <Input
              id="lastName"
              value={values.lastName}
              onChange={(event) => set("lastName")(event.target.value)}
              error={errors.lastName}
            />
          </Field>
          <Field label="Date of birth" htmlFor="dob" required error={errors.dob}>
            <Input
              id="dob"
              type="date"
              value={values.dob}
              onChange={(event) => set("dob")(event.target.value)}
              error={errors.dob}
            />
          </Field>
          <Field label="Gender" htmlFor="gender" error={errors.gender}>
            <Select
              id="gender"
              value={values.gender}
              onChange={(event) => set("gender")(event.target.value)}
            >
              {GENDERS.map((gender) => (
                <option key={gender} value={gender}>
                  {gender === "" ? "Not stated" : gender}
                </option>
              ))}
            </Select>
          </Field>
          <Field
            label="Admission date"
            htmlFor="admissionDate"
            required
            error={errors.admissionDate}
          >
            <Input
              id="admissionDate"
              type="date"
              value={values.admissionDate}
              onChange={(event) => set("admissionDate")(event.target.value)}
              error={errors.admissionDate}
            />
          </Field>
          <Field label="Guardian name" htmlFor="guardianName" error={errors.guardianName}>
            <Input
              id="guardianName"
              value={values.guardianName}
              onChange={(event) => set("guardianName")(event.target.value)}
            />
          </Field>
          <Field label="Guardian phone" htmlFor="guardianPhone" error={errors.guardianPhone}>
            <Input
              id="guardianPhone"
              type="tel"
              value={values.guardianPhone}
              onChange={(event) => set("guardianPhone")(event.target.value)}
            />
          </Field>
          <Field label="Guardian email" htmlFor="guardianEmail" error={errors.guardianEmail}>
            <Input
              id="guardianEmail"
              type="email"
              value={values.guardianEmail}
              onChange={(event) => set("guardianEmail")(event.target.value)}
            />
          </Field>
        </div>
      </div>
    </Modal>
  )
}
