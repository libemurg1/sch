import { useEffect, useState } from "react"
import { z } from "zod"
import { useCreateClass, useUpdateClass } from "../../api/queries"
import { Button, ErrorBanner, Field, Input, Modal } from "../../components"
import type { Class } from "../../types/schema/Class"
import type { CreateClassInput } from "../../types/schema/CreateClassInput"

const schema = z.object({
  name: z.string().trim().min(1, "Class name is required"),
  gradeLevel: z.coerce.number().int().min(0, "Grade level cannot be negative"),
  capacity: z.coerce
    .number()
    .int()
    .positive("Capacity must be greater than zero"),
})

type FormValues = z.infer<typeof schema>

const EMPTY: FormValues = { name: "", gradeLevel: 0, capacity: 30 }

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

  useEffect(() => {
    if (open) {
      setValues(
        editing
          ? {
              name: editing.name,
              gradeLevel: editing.gradeLevel,
              capacity: editing.capacity,
            }
          : EMPTY,
      )
      setErrors({})
      create.reset()
      update.reset()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, editing])

  const mutation = editing ? update : create

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
      name: parsed.data.name,
      gradeLevel: parsed.data.gradeLevel,
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
          ? "Renaming a class does not change its enrolments or invoices."
          : undefined
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
        <Field label="Name" htmlFor="class-name" required error={errors.name}>
          <Input
            id="class-name"
            value={values.name}
            placeholder="Grade 5 - Blue"
            onChange={(event) =>
              setValues((current) => ({ ...current, name: event.target.value }))
            }
            error={errors.name}
          />
        </Field>
        <div className="grid gap-4 sm:grid-cols-2">
          <Field
            label="Grade level"
            htmlFor="grade-level"
            required
            error={errors.gradeLevel}
            hint="Nursery is 0, then Grade 1 is 1 and so on."
          >
            <Input
              id="grade-level"
              type="number"
              min={0}
              value={values.gradeLevel}
              onChange={(event) =>
                setValues((current) => ({
                  ...current,
                  gradeLevel: Number(event.target.value),
                }))
              }
              error={errors.gradeLevel}
            />
          </Field>
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
        </div>
      </div>
    </Modal>
  )
}
