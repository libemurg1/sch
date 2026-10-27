import { useEffect, useMemo, useState } from "react"
import {
  useActiveTerm,
  useApplyPromotion,
  useClasses,
  useCopyClasses,
  usePromotionPreview,
  useTerms,
} from "../../api/queries"
import {
  Button,
  Card,
  ConfirmDialog,
  EmptyState,
  ErrorBanner,
  Loading,
  PageHeader,
  Select,
  Table,
  type Column,
} from "../../components"
import { Page } from "../../app/AppShell"
import type { PromotionCandidate } from "../../types/schema/PromotionCandidate"
import type { PromotionEntryInput } from "../../types/schema/PromotionEntryInput"
import type { PromotionResult } from "../../types/schema/PromotionResult"

const currentYear = new Date().getFullYear()

type Choice = { outcome: string; toClassId: string }

export function PromotionPage() {
  const terms = useTerms()
  const classes = useClasses()
  const activeTerm = useActiveTerm()

  const [fromYear, setFromYear] = useState<number | null>(null)
  const [toYear, setToYear] = useState<number | null>(null)
  const [choices, setChoices] = useState<Record<string, Choice>>({})
  const [confirming, setConfirming] = useState(false)
  const [result, setResult] = useState<PromotionResult | null>(null)

  const years = useMemo(() => {
    const set = new Set<number>((terms.data ?? []).map((term) => term.year))
    for (const klass of classes.data ?? []) {
      set.add(klass.academicYear)
    }
    set.add(currentYear)
    set.add(currentYear + 1)
    if (fromYear !== null) {
      set.add(fromYear + 1)
    }
    return [...set].sort((a, b) => a - b)
  }, [terms.data, classes.data, fromYear])

  useEffect(() => {
    if (fromYear === null) {
      setFromYear(
        activeTerm.data?.year ??
          ((terms.data ?? []).reduce(
            (latest, term) => Math.max(latest, term.year),
            0,
          ) ||
            currentYear),
      )
    }
  }, [activeTerm.data, terms.data, fromYear])

  useEffect(() => {
    if (fromYear !== null && (toYear === null || toYear !== fromYear + 1)) {
      setToYear(fromYear + 1)
    }
  }, [fromYear, toYear])

  useEffect(() => {
    setResult(null)
  }, [fromYear, toYear])

  const preview = usePromotionPreview(
    fromYear ?? undefined,
    toYear ?? undefined,
  )
  const copy = useCopyClasses()
  const apply = useApplyPromotion()

  const candidates = preview.data?.candidates ?? []

  useEffect(() => {
    if (!preview.data) {
      return
    }
    const next: Record<string, Choice> = {}
    for (const candidate of preview.data.candidates) {
      next[candidate.studentId] = {
        outcome: candidate.suggestedOutcome,
        toClassId: candidate.suggestedClassId ?? "",
      }
    }
    setChoices(next)
  }, [preview.data])

  const targetClasses = useMemo(
    () => (classes.data ?? []).filter((klass) => klass.academicYear === toYear),
    [classes.data, toYear],
  )

  const classesFor = (candidate: PromotionCandidate, outcome: string) => {
    const grade =
      outcome === "PROMOTED"
        ? candidate.fromGradeLevel + 1
        : candidate.fromGradeLevel
    return targetClasses.filter((klass) => klass.gradeLevel === grade)
  }

  const entries: PromotionEntryInput[] = candidates
    .map((candidate) => {
      const choice = choices[candidate.studentId]
      if (!choice) {
        return null
      }
      return {
        studentId: candidate.studentId,
        outcome: choice.outcome,
        toClassId:
          choice.outcome === "GRADUATED" ? null : choice.toClassId || null,
      }
    })
    .filter((entry): entry is PromotionEntryInput => entry !== null)

  const incomplete = candidates.some((candidate) => {
    const choice = choices[candidate.studentId]
    return (
      !choice ||
      (choice.outcome !== "GRADUATED" && !choice.toClassId)
    )
  })

  const counts = entries.reduce(
    (totals, entry) => {
      if (entry.outcome === "PROMOTED") totals.promoted += 1
      else if (entry.outcome === "REPEATED") totals.repeated += 1
      else totals.graduated += 1
      return totals
    },
    { promoted: 0, repeated: 0, graduated: 0 },
  )

  const columns: Column<PromotionCandidate>[] = [
    {
      key: "student",
      header: "Student",
      render: (candidate) => (
        <div>
          <p className="font-medium text-slate-900">
            {candidate.firstName} {candidate.lastName}
          </p>
          <p className="text-xs text-slate-500">{candidate.studentNumber}</p>
        </div>
      ),
    },
    {
      key: "from",
      header: "Current class",
      hideOnSmall: true,
      render: (candidate) => candidate.fromClassName,
    },
    {
      key: "outcome",
      header: "Result",
      render: (candidate) => {
        const choice = choices[candidate.studentId]
        return (
          <Select
            aria-label={`Result for ${candidate.firstName} ${candidate.lastName}`}
            value={choice?.outcome ?? candidate.suggestedOutcome}
            onChange={(event) => {
              const outcome = event.target.value
              const grade =
                outcome === "PROMOTED"
                  ? candidate.fromGradeLevel + 1
                  : candidate.fromGradeLevel
              setChoices((current) => {
                const previous = current[candidate.studentId]?.toClassId ?? ""
                const keep =
                  outcome !== "GRADUATED" &&
                  targetClasses.some(
                    (klass) => klass.id === previous && klass.gradeLevel === grade,
                  )
                return {
                  ...current,
                  [candidate.studentId]: {
                    outcome,
                    toClassId: keep ? previous : "",
                  },
                }
              })
            }}
          >
            {candidate.fromGradeLevel < 7 ? (
              <option value="PROMOTED">Promote</option>
            ) : null}
            <option value="REPEATED">Repeat</option>
            {candidate.fromGradeLevel >= 7 ? (
              <option value="GRADUATED">Graduate</option>
            ) : null}
          </Select>
        )
      },
    },
    {
      key: "to",
      header: "Next class",
      render: (candidate) => {
        const choice = choices[candidate.studentId]
        if (!choice || choice.outcome === "GRADUATED") {
          return <span className="text-slate-400">Leaves the school</span>
        }
        const options = classesFor(candidate, choice.outcome)
        return options.length === 0 ? (
          <span className="text-rose-600">No class available</span>
        ) : (
          <Select
            aria-label={`Next class for ${candidate.firstName} ${candidate.lastName}`}
            value={choice.toClassId}
            onChange={(event) =>
              setChoices((current) => ({
                ...current,
                [candidate.studentId]: {
                  outcome: choice.outcome,
                  toClassId: event.target.value,
                },
              }))
            }
          >
            <option value="">Choose a class</option>
            {options.map((klass) => (
              <option key={klass.id} value={klass.id}>
                {klass.name}
              </option>
            ))}
          </Select>
        )
      },
    },
  ]

  return (
    <Page>
      <PageHeader
        title="Promotion"
        description="Move students into the next academic year, repeat a grade, or graduate Grade 7."
      />

      <Card
        title="Academic year"
        description="Promotion always moves students one year forward. Create the new year's Term 1 first."
      >
        <div className="flex flex-wrap items-end gap-4">
          <div className="w-40">
            <label
              htmlFor="promotion-from"
              className="mb-1 block text-sm font-medium text-slate-700"
            >
              From year
            </label>
            <Select
              id="promotion-from"
              value={fromYear ?? ""}
              onChange={(event) => setFromYear(Number(event.target.value))}
            >
              {years.map((year) => (
                <option key={year} value={year}>
                  {year}
                </option>
              ))}
            </Select>
          </div>
          <div className="w-40">
            <label
              htmlFor="promotion-to"
              className="mb-1 block text-sm font-medium text-slate-700"
            >
              To year
            </label>
            <Select
              id="promotion-to"
              value={toYear ?? ""}
              onChange={(event) => setToYear(Number(event.target.value))}
            >
              {years.map((year) => (
                <option key={year} value={year}>
                  {year}
                </option>
              ))}
            </Select>
          </div>
          <Button
            variant="secondary"
            loading={copy.isPending}
            disabled={fromYear === null || toYear === null || toYear <= fromYear}
            onClick={() => {
              if (fromYear !== null && toYear !== null) {
                copy.mutate({ fromYear, toYear })
              }
            }}
          >
            Copy classes to {toYear ?? "next year"}
          </Button>
        </div>
        {copy.isError ? (
          <div className="mt-3">
            <ErrorBanner error={copy.error} />
          </div>
        ) : null}
        {copy.isSuccess ? (
          <p className="mt-3 text-sm text-slate-600">
            Copied {copy.data} class{copy.data === 1 ? "" : "es"} into{" "}
            {toYear}. Existing classes were left untouched.
          </p>
        ) : null}
        {fromYear !== null && toYear !== null && toYear !== fromYear + 1 ? (
          <p className="mt-3 text-sm text-rose-600">
            Students can only be promoted into the following year.
          </p>
        ) : null}
        {preview.data && !preview.data.termReady && toYear !== null ? (
          <p className="mt-3 rounded-md bg-amber-50 px-3 py-2 text-sm text-amber-800">
            {toYear} has no Term 1 yet. Create it under Terms before applying
            the promotion.
          </p>
        ) : null}
      </Card>

      <Card
        title={`Students in ${fromYear ?? ""}`}
        description="Check each result before applying. Rows disappear once a student has been moved."
        actions={
          <Button
            size="sm"
            loading={apply.isPending}
            disabled={
              candidates.length === 0 ||
              incomplete ||
              !preview.data?.termReady ||
              fromYear === null ||
              toYear === null ||
              toYear !== fromYear + 1
            }
            onClick={() => setConfirming(true)}
          >
            Apply promotion
          </Button>
        }
      >
        {preview.isLoading ? (
          <Loading />
        ) : preview.isError ? (
          <ErrorBanner
            error={preview.error}
            onRetry={() => preview.refetch()}
          />
        ) : candidates.length === 0 ? (
          <EmptyState
            title="Nobody left to promote"
            description="Students are listed once when they are enrolled in the chosen year and not yet moved to the next one."
          />
        ) : (
          <Table
            columns={columns}
            rows={candidates}
            keyOf={(candidate) => candidate.studentId}
          />
        )}

        {result ? (
          <p className="mt-4 rounded-md bg-emerald-50 px-3 py-2 text-sm text-emerald-800">
            Promoted {result.promoted}, repeated {result.repeated}, graduated{" "}
            {result.graduated}.
          </p>
        ) : null}
        {incomplete && candidates.length > 0 ? (
          <p className="mt-4 text-sm text-amber-700">
            Choose a next class for every promoted or repeating student.
          </p>
        ) : null}
      </Card>

      <ConfirmDialog
        open={confirming}
        title="Apply promotion"
        confirmLabel="Apply promotion"
        loading={apply.isPending}
        error={apply.error}
        onCancel={() => {
          apply.reset()
          setConfirming(false)
        }}
        onConfirm={() => {
          if (fromYear === null || toYear === null) {
            return
          }
          apply.mutate(
            { fromYear, toYear, entries },
            {
              onSuccess: (data) => {
                setResult(data)
                setConfirming(false)
              },
            },
          )
        }}
      >
        <p>
          Move <strong>{entries.length}</strong> student
          {entries.length === 1 ? "" : "s"} from {fromYear} into {toYear}?
        </p>
        <p className="mt-2">
          {counts.promoted} promoted, {counts.repeated} repeating,{" "}
          {counts.graduated} graduating.
        </p>
        <p className="mt-2">
          New enrolments are created in Term 1 of {toYear}. This cannot be
          undone from the app.
        </p>
      </ConfirmDialog>
    </Page>
  )
}
