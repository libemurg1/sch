/** The value used by year filters to show data from every academic year. */
export const ALL_YEARS = "all"

/** Distinct years, newest first. */
export function collectYears(
  values: Iterable<number | null | undefined>,
): number[] {
  const years = new Set<number>()
  for (const value of values) {
    if (typeof value === "number") {
      years.add(value)
    }
  }
  return [...years].sort((left, right) => right - left)
}

/**
 * The year a filter should start on: the active term's year when there is one,
 * otherwise the newest year that has data, otherwise the current year.
 */
export function defaultYear(
  activeYear: number | null | undefined,
  years: number[],
): string {
  if (activeYear != null) {
    return String(activeYear)
  }
  if (years.length > 0) {
    return String(years[0])
  }
  return String(new Date().getFullYear())
}
