import { useEffect, useState } from "react"
import { z } from "zod"
import { useCreateDiscount } from "../../api/queries"
import { Button, ErrorBanner, Field, Input, Modal, Select } from "../../components"
import type { CreateDiscountInput } from "../../types/schema/CreateDiscountInput"

const schema = z.object({
  name: z.string().trim().min(1, "Discount name is required"),
  kind: z.enum(["PERCENT", "FIXED"]),
  valueText: z.string().trim().min(1, "Enter a value"),
})

type FormValues = z.infer<typeof schema>

const EMPTY: FormValues = { name: "", kind: "PERCENT", valueText: "" }

export function CreateDiscountModal({
  open,
  onClose,
}: {
  open: boolean
  onClose: () => void
}) {
  const [values, setValues] = useState<FormValues>(EMPTY)
  const [errors, setErrors] = useState<Partial<Record<keyof FormValues, string>>>(
    {},
  )
  const mutation = useCreateDiscount()

  useEffect(() => {
    if (open) {
      setValues(EMPTY)
      setErrors({})
      mutation.reset()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open])

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

    // The form takes a friendly number; the backend stores basis points for
    // percentages and cents for fixed amounts.
    let raw: number
    if (parsed.data.kind === "PERCENT") {
      const percent = Number(parsed.data.valueText)
      if (!Number.isFinite(percent) || percent <= 0) {
        setErrors({ valueText: "Enter a percentage greater than zero" })
        return
      }
      if (percent > 100) {
        setErrors({ valueText: "A percentage discount cannot exceed 100%" })
        return
      }
      raw = Math.round(percent * 100)
    } else {
      const cents = Number(parsed.data.valueText)
      if (!Number.isFinite(cents) || cents <= 0) {
        setErrors({ valueText: "Enter an amount greater than zero" })
        return
      }
      raw = Math.round(cents)
    }

    const input: CreateDiscountInput = {
      name: parsed.data.name,
      kind: parsed.data.kind,
      value: raw,
    }
    mutation.mutate(input, { onSuccess: onClose })
  }

  return (
    <Modal
      open={open}
      onClose={onClose}
      title="Add a discount"
      description="Discounts stack. Percentages apply to the discountable part of the invoice, fixed amounts come off the whole invoice."
      footer={
        <>
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={submit} loading={mutation.isPending}>
            Add discount
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        {mutation.isError ? <ErrorBanner error={mutation.error} /> : null}

        <Field label="Name" htmlFor="discount-name" required error={errors.name}>
          <Input
            id="discount-name"
            value={values.name}
            placeholder="Sibling discount"
            onChange={(event) =>
              setValues((current) => ({ ...current, name: event.target.value }))
            }
            error={errors.name}
          />
        </Field>

        <div className="grid gap-4 sm:grid-cols-2">
          <Field label="Kind" htmlFor="discount-kind" required>
            <Select
              id="discount-kind"
              value={values.kind}
              onChange={(event) =>
                setValues((current) => ({
                  ...current,
                  kind: event.target.value as FormValues["kind"],
                  valueText: "",
                }))
              }
            >
              <option value="PERCENT">Percentage</option>
              <option value="FIXED">Fixed amount</option>
            </Select>
          </Field>

          <Field
            label={values.kind === "PERCENT" ? "Percentage" : "Amount in cents"}
            htmlFor="discount-value"
            required
            error={errors.valueText}
            hint={
              values.kind === "PERCENT"
                ? "For example 10 for ten percent, stored as 1000 basis points."
                : "The raw amount in cents. For example 2500 for 25.00."
            }
          >
            <Input
              id="discount-value"
              inputMode="decimal"
              value={values.valueText}
              onChange={(event) =>
                setValues((current) => ({
                  ...current,
                  valueText: event.target.value,
                }))
              }
              error={errors.valueText}
            />
          </Field>
        </div>
      </div>
    </Modal>
  )
}
