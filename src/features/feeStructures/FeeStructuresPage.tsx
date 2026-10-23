import { useEffect, useState } from "react"
import {
  useClasses,
  useFeeStructure,
  useSaveFeeStructure,
  useTerms,
} from "../../api/queries"
import {
  Button,
  Card,
  EmptyState,
  ErrorBanner,
  Input,
  Loading,
  PageHeader,
  Select,
} from "../../components"
import { Page } from "../../app/AppShell"
import { formatTermLabel } from "../../lib/dates"
import { centsToInputValue, formatMoney, parseMoneyToCents } from "../../lib/money"
import type { FeeItemInput } from "../../types/schema/FeeItemInput"

interface EditableItem extends FeeItemInput {
  key: string
  amountText: string
  error?: string
}

let nextKey = 0
function makeItem(item?: FeeItemInput): EditableItem {
  nextKey += 1
  const amountCents = item?.amountCents ?? 0
  return {
    key: `item-${nextKey}`,
    description: item?.description ?? "",
    amountCents,
    amountText: item ? centsToInputValue(amountCents) : "",
    discountable: item?.discountable ?? true,
    sortOrder: item?.sortOrder ?? 0,
  }
}

export function FeeStructuresPage() {
  const classes = useClasses()
  const terms = useTerms()
  const [classId, setClassId] = useState("")
  const [termId, setTermId] = useState("")
  const [items, setItems] = useState<EditableItem[]>([])
  const [dirty, setDirty] = useState(false)

  const structure = useFeeStructure(
    classId || undefined,
    termId || undefined,
  )
  const mutation = useSaveFeeStructure()

  useEffect(() => {
    if (!classId && classes.data && classes.data.length > 0) {
      setClassId(classes.data[0].id)
    }
  }, [classes.data, classId])

  useEffect(() => {
    if (!termId && terms.data && terms.data.length > 0) {
      setTermId(terms.data[0].id)
    }
  }, [terms.data, termId])

  // Load the saved structure whenever the class or term changes.
  useEffect(() => {
    if (structure.isLoading) {
      return
    }
    const bundle = structure.data
    if (!bundle) {
      setItems([])
      setDirty(false)
      return
    }
    setItems(bundle[1].map((item) => makeItem(item)))
    setDirty(false)
  }, [structure.data, structure.isLoading])

  const update = (key: string, patch: Partial<EditableItem>) => {
    setItems((current) =>
      current.map((item) => (item.key === key ? { ...item, ...patch } : item)),
    )
    setDirty(true)
  }

  const totalCents = items.reduce((total, item) => total + item.amountCents, 0)

  const save = () => {
    const cleaned = items.map((item, index) => ({
      description: item.description.trim(),
      amountCents: item.amountCents,
      discountable: item.discountable,
      sortOrder: index,
      error: item.description.trim() === "" ? "Description is required" : undefined,
    }))

    if (cleaned.some((item) => item.error)) {
      setItems((current) =>
        current.map((item, index) => ({
          ...item,
          error: cleaned[index].error,
        })),
      )
      return
    }

    if (cleaned.length === 0) {
      return
    }

    mutation.mutate(
      {
        classId,
        termId,
        items: cleaned.map(({ description, amountCents, discountable, sortOrder }) => ({
          description,
          amountCents,
          discountable,
          sortOrder,
        })),
      },
      { onSuccess: () => setDirty(false) },
    )
  }

  return (
    <Page>
      <PageHeader
        title="Fee Structures"
        description="What each class is charged for in each term. Saving replaces the whole structure."
        actions={
          <Button
            onClick={save}
            disabled={!classId || !termId || items.length === 0}
            loading={mutation.isPending}
          >
            Save structure
          </Button>
        }
      />

      <Card>
        <div className="mb-4 flex flex-wrap gap-3">
          <div className="w-56">
            <label
              htmlFor="fee-class"
              className="mb-1 block text-sm font-medium text-slate-700"
            >
              Class
            </label>
            <Select
              id="fee-class"
              value={classId}
              onChange={(event) => setClassId(event.target.value)}
            >
              <option value="">Choose a class</option>
              {(classes.data ?? []).map((klass) => (
                <option key={klass.id} value={klass.id}>
                  {klass.name}
                </option>
              ))}
            </Select>
          </div>
          <div className="w-56">
            <label
              htmlFor="fee-term"
              className="mb-1 block text-sm font-medium text-slate-700"
            >
              Term
            </label>
            <Select
              id="fee-term"
              value={termId}
              onChange={(event) => setTermId(event.target.value)}
            >
              <option value="">Choose a term</option>
              {(terms.data ?? []).map((term) => (
                <option key={term.id} value={term.id}>
                  {formatTermLabel(term)}
                </option>
              ))}
            </Select>
          </div>
        </div>

        {mutation.isError ? (
          <ErrorBanner error={mutation.error} className="mb-4" />
        ) : null}
        {mutation.isSuccess && !dirty ? (
          <div className="mb-4 rounded-md bg-emerald-50 px-4 py-3 text-sm text-emerald-800 ring-1 ring-inset ring-emerald-200">
            The fee structure has been saved.
          </div>
        ) : null}

        {!classId || !termId ? (
          <EmptyState title="Choose a class and a term" />
        ) : structure.isLoading ? (
          <Loading />
        ) : structure.isError ? (
          <ErrorBanner error={structure.error} onRetry={() => structure.refetch()} />
        ) : (
          <>
            {items.length === 0 ? (
              <EmptyState
                title="No fee items yet"
                description="Add the charges this class is billed for, then save."
                action={
                  <Button onClick={() => setItems([makeItem()])}>Add fee item</Button>
                }
              />
            ) : (
              <div className="space-y-3">
                {items.map((item) => (
                  <div
                    key={item.key}
                    className="grid grid-cols-1 items-start gap-3 rounded-md border border-slate-200 p-3 sm:grid-cols-[1fr_10rem_8rem_5rem]"
                  >
                    <Input
                      value={item.description}
                      placeholder="Tuition fee"
                      error={item.error}
                      onChange={(event) =>
                        update(item.key, { description: event.target.value })
                      }
                    />
                    <Input
                      value={item.amountText}
                      inputMode="decimal"
                      placeholder="0.00"
                      onChange={(event) => {
                        const text = event.target.value
                        const cents = parseMoneyToCents(text)
                        update(item.key, {
                          amountText: text,
                          amountCents: cents ?? 0,
                        })
                      }}
                    />
                    <label className="flex items-center gap-2 text-sm text-slate-700 sm:justify-center">
                      <input
                        type="checkbox"
                        checked={item.discountable}
                        onChange={(event) =>
                          update(item.key, { discountable: event.target.checked })
                        }
                        className="h-4 w-4 rounded border-slate-300 text-indigo-600 focus:ring-indigo-600"
                      />
                      Discountable
                    </label>
                    <div className="flex justify-end">
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => {
                          setItems((current) =>
                            current.filter((row) => row.key !== item.key),
                          )
                          setDirty(true)
                        }}
                      >
                        Remove
                      </Button>
                    </div>
                  </div>
                ))}
                <div className="flex items-center justify-between">
                  <Button
                    variant="secondary"
                    onClick={() => setItems((current) => [...current, makeItem()])}
                  >
                    Add fee item
                  </Button>
                  <p className="text-sm text-slate-600">
                    Total:{" "}
                    <span className="font-semibold text-slate-900">
                      {formatMoney(totalCents)}
                    </span>
                  </p>
                </div>
              </div>
            )}
          </>
        )}
      </Card>

      <Card title="What discountable means">
        <p className="text-sm text-slate-600">
          Only charges marked <strong>discountable</strong> attract percentage
          discounts. Fixed discounts always come off the whole invoice. Every
          discount is saved on the invoice as its own negative line, so the
          invoice shows exactly how it was calculated.
        </p>
      </Card>
    </Page>
  )
}
