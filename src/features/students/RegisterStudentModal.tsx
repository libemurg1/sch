import { useEffect, useState } from "react"
import { z } from "zod"
import { useRegisterStudent } from "../../api/queries"
import { Button, ErrorBanner, Field, Input, Modal, Select } from "../../components"
import type { RegisterStudentInput } from "../../types/schema/RegisterStudentInput"

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

export interface RegisterStudentModalProps {
  open: boolean
  onClose: () => void
  /** The server's current date, used as the default admission date. */
  today: string
}

export function RegisterStudentModal({
  open,
  onClose,
  today,
}: RegisterStudentModalProps) {
  const [values, setValues] = useState<FormValues>(EMPTY)
  const [errors, setErrors] = useState<Partial<Record<keyof FormValues, string>>>(
    {},
  )
  const mutation = useRegisterStudent()

  useEffect(() => {
    if (open) {
      setValues({ ...EMPTY, admissionDate: today })
      setErrors({})
      mutation.reset()
    }
    // `mutation` is stable enough for this purpose; re-running on every
    // render would wipe what the user is typing.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, today])

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

    const input: RegisterStudentInput = {
      firstName: parsed.data.firstName,
      lastName: parsed.data.lastName,
      dob: parsed.data.dob,
      gender: optional(parsed.data.gender),
      admissionDate: parsed.data.admissionDate,
      guardianName: optional(parsed.data.guardianName),
      guardianPhone: optional(parsed.data.guardianPhone),
      guardianEmail: optional(parsed.data.guardianEmail),
    }

    mutation.mutate(input, {
      onSuccess: () => {
        setValues(EMPTY)
        onClose()
      },
    })
  }

  return (
    <Modal
      open={open}
      onClose={onClose}
      title="Register a student"
      description="The student number is allocated automatically."
      footer={
        <>
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={submit} loading={mutation.isPending}>
            Register student
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
