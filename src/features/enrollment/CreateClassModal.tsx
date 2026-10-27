import { useEffect, useMemo, useState } from "react"
import { z } from "zod"
import {
  useActiveTerm,
  useClasses,
  useCreateClass,
  useGrades,
  useUpdateClass,
} from "../../api/queries"
import { Button, ErrorBanner, Field, Input, Modal, Select } from "../../components"
import type { Class } from "../../types/schema/Class"
import type { CreateClassInput } from "../../types/schema/CreateClassInput"

const schema = z.object({
  gradeLevel: z.coerce.number().int("Choose a grade"),
  stream: z.string().trim().max(40, "The stream can be 40 characters or fewer"),
  academicYear: z.coerce
    .number()
    .int()
    .min(2000, "Enter a valid year")
    .max(2100, "Enter a valid year"),
  capacity: z.coerce
    .number()
    .int()
    .positive("Capacity must be greater than zero"),
})

type FormValues = z.infer<typeof schema>

const currentYear = new Date().getFullYear()

const EMPTY: FormValues = {
  gradeLevel: 1,
  stream: "",
  academicYear: currentYear,
  capacity: 30,
}

export function CreateClassModal({
  open,
  onClose,
  editing,
}: {
  open: boolean
  onClose: () => void
  editing?: Class | null
}) {
  const [values, setValues] = useState<FormValues>(EMPTY)
  const [errors, setErrors] = useState<Partial<Record<keyof FormValues, string>>>(
    {},
  )
  const create = useCreateClass()
  const update = useUpdateClass()
  const grades = useGrades()
  const classes = useClasses()
  const activeTerm = useActiveTerm()

  const years = useMemo(() => {
    const set = new Set<number>((classes.data ?? []).map((c) => c.academicYear))
    if (activeTerm.data) {
      set.add(activeTerm.data.year)
    }
    set.add(currentYear)
    set.add(currentYear + 1)
    return [...set].sort((a, b) => a - b)
  }, [classes.data, activeTerm.data])

  useEffect(() => {
    if (open) {
      setValues(
        editing
          ? {
              gradeLevel: editing.gradeLevel,
              stream: editing.stream,
              academicYear: editing.academicYear,
              capacity: editing.capacity,
            }
          : {
              ...EMPTY,
              academicYear: activeTerm.data?.year ?? currentYear,
            },
      )
      setErrors({})
      create.reset()
      update.reset()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, editing, activeTerm.data?.year])

  const mutation = editing ? update : create
  const grade = grades.data?.find((item) => item.level === values.gradeLevel)
  const previewName = grade
    ? values.stream.trim()
      ? `${grade.name} ${values.stream.trim()}`
      : grade.name
    : ""

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

    const input: CreateClassInput = {
      gradeLevel: parsed.data.gradeLevel,
      stream: parsed.data.stream.trim(),
      academicYear: parsed.data.academicYear,
      capacity: parsed.data.capacity,
    }

    if (editing) {
      update.mutate({ ...input, id: editing.id }, { onSuccess: onClose })
    } else {
      create.mutate(input, { onSuccess: onClose })
    }
  }

  return (
    <Modal
      open={open}
      onClose={onClose}
      title={editing ? "Edit class" : "Add a class"}
      description={
        editing
          ? "Classes belong to one academic year. Names are built from the grade and stream."
          : "Each year has its own copy of the classes, so history stays intact."
      }
      footer={
        <>
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={submit} loading={mutation.isPending}>
            {editing ? "Save changes" : "Add class"}
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        {mutation.isError ? <ErrorBanner error={mutation.error} /> : null}
        <div className="grid gap-4 sm:grid-cols-3">
          <Field
            label="Grade"
            htmlFor="class-grade"
            required
            error={errors.gradeLevel}
          >
            <Select
              id="class-grade"
              value={values.gradeLevel}
              onChange={(event) =>
                setValues((current) => ({
                  ...current,
                  gradeLevel: Number(event.target.value),
                }))
              }
              error={errors.gradeLevel}
            >
              {(grades.data ?? []).map((item) => (
                <option key={item.level} value={item.level}>
                  {item.name}
                </option>
              ))}
            </Select>
          </Field>
          <Field
            label="Stream"
            htmlFor="class-stream"
            error={errors.stream}
            hint="Optional, e.g. Blue."
          >
            <Input
              id="class-stream"
              value={values.stream}
              placeholder="Blue"
              onChange={(event) =>
                setValues((current) => ({
                  ...current,
                  stream: event.target.value,
                }))
              }
              error={errors.stream}
            />
          </Field>
          <Field
            label="Academic year"
            htmlFor="class-year"
            required
            error={errors.academicYear}
          >
            <Select
              id="class-year"
              value={values.academicYear}
              onChange={(event) =>
                setValues((current) => ({
                  ...current,
                  academicYear: Number(event.target.value),
                }))
              }
              error={errors.academicYear}
            >
              {years.map((year) => (
                <option key={year} value={year}>
                  {year}
                </option>
              ))}
            </Select>
          </Field>
        </div>
        <Field
          label="Capacity"
          htmlFor="capacity"
          required
          error={errors.capacity}
          hint="How many students the class can hold."
        >
          <Input
            id="capacity"
            type="number"
            min={1}
            value={values.capacity}
            onChange={(event) =>
              setValues((current) => ({
                ...current,
                capacity: Number(event.target.value),
              }))
            }
            error={errors.capacity}
          />
        </Field>
        <p className="rounded-md bg-slate-50 px-3 py-2 text-sm text-slate-600">
          Class name:{" "}
          <span className="font-medium text-slate-900">
            {previewName || "—"}
          </span>
        </p>
      </div>
    </Modal>
  )
}
