import { useEffect, useState } from "react"
import { z } from "zod"
import { useCreateTerm, useUpdateTerm } from "../../api/queries"
import { Button, ErrorBanner, Field, Input, Modal } from "../../components"
import type { CreateTermInput } from "../../types/schema/CreateTermInput"
import type { Term } from "../../types/schema/Term"

const schema = z.object({
  year: z.coerce.number().int().min(1900, "Enter a valid year"),
  termNumber: z.coerce
    .number()
    .int()
    .min(1, "Term number must be between 1 and 4")
    .max(4, "Term number must be between 1 and 4"),
  startDate: z.string().min(1, "Start date is required"),
  endDate: z.string().min(1, "End date is required"),
  isActive: z.boolean(),
})

type FormValues = z.infer<typeof schema>

function currentYear(): number {
  return new Date().getFullYear()
}

const EMPTY: FormValues = {
  year: currentYear(),
  termNumber: 1,
  startDate: "",
  endDate: "",
  isActive: false,
}

export function CreateTermModal({
  open,
  onClose,
  editing,
}: {
  open: boolean
  onClose: () => void
  editing?: Term | null
}) {
  const [values, setValues] = useState<FormValues>(EMPTY)
  const [errors, setErrors] = useState<Partial<Record<keyof FormValues, string>>>(
    {},
  )
  const create = useCreateTerm()
  const update = useUpdateTerm()

  useEffect(() => {
    if (open) {
      setValues(
        editing
          ? {
              year: editing.year,
              termNumber: editing.termNumber,
              startDate: editing.startDate,
              endDate: editing.endDate,
              isActive: editing.isActive,
            }
          : { ...EMPTY, year: currentYear() },
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

    if (parsed.data.endDate < parsed.data.startDate) {
      setErrors({ endDate: "End date cannot precede the start date" })
      return
    }

    const input: CreateTermInput = {
      year: parsed.data.year,
      termNumber: parsed.data.termNumber,
      startDate: parsed.data.startDate,
      endDate: parsed.data.endDate,
      isActive: parsed.data.isActive,
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
      title={editing ? "Edit term" : "Add a term"}
      description="Only one term can be active at a time."
      footer={
        <>
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={submit} loading={mutation.isPending}>
            {editing ? "Save changes" : "Add term"}
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        {mutation.isError ? <ErrorBanner error={mutation.error} /> : null}
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label="Year" htmlFor="term-year" required error={errors.year}>
            <Input
              id="term-year"
              type="number"
              value={values.year}
              onChange={(event) =>
                setValues((current) => ({
                  ...current,
                  year: Number(event.target.value),
                }))
              }
              error={errors.year}
            />
          </Field>
          <Field
            label="Term number"
            htmlFor="term-number"
            required
            error={errors.termNumber}
          >
            <Input
              id="term-number"
              type="number"
              min={1}
              max={4}
              value={values.termNumber}
              onChange={(event) =>
                setValues((current) => ({
                  ...current,
                  termNumber: Number(event.target.value),
                }))
              }
              error={errors.termNumber}
            />
          </Field>
          <Field
            label="Start date"
            htmlFor="term-start"
            required
            error={errors.startDate}
          >
            <Input
              id="term-start"
              type="date"
              value={values.startDate}
              onChange={(event) =>
                setValues((current) => ({
                  ...current,
                  startDate: event.target.value,
                }))
              }
              error={errors.startDate}
            />
          </Field>
          <Field
            label="End date"
            htmlFor="term-end"
            required
            error={errors.endDate}
          >
            <Input
              id="term-end"
              type="date"
              value={values.endDate}
              onChange={(event) =>
                setValues((current) => ({
                  ...current,
                  endDate: event.target.value,
                }))
              }
              error={errors.endDate}
            />
          </Field>
        </div>
        <label className="flex items-center gap-2 text-sm text-slate-700">
          <input
            type="checkbox"
            checked={values.isActive}
            onChange={(event) =>
              setValues((current) => ({
                ...current,
                isActive: event.target.checked,
              }))
            }
            className="h-4 w-4 rounded border-slate-300 text-indigo-600 focus:ring-indigo-600"
          />
          Make this the active term
        </label>
      </div>
    </Modal>
  )
}
