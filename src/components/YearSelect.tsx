import { ALL_YEARS } from "../lib/years"
import { Select } from "./Input"

export interface YearSelectProps {
  id: string
  value: string
  onChange: (value: string) => void
  years: number[]
  className?: string
}

/** A year filter with an "All years" option. */
export function YearSelect({
  id,
  value,
  onChange,
  years,
  className = "w-44",
}: YearSelectProps) {
  const selectedYear =
    value === ALL_YEARS || value === "" ? null : Number(value)
  const options =
    selectedYear !== null &&
    Number.isInteger(selectedYear) &&
    !years.includes(selectedYear)
      ? [...years, selectedYear].sort((left, right) => right - left)
      : years

  return (
    <div className={className}>
      <label
        htmlFor={id}
        className="mb-1 block text-sm font-medium text-slate-700"
      >
        Year
      </label>
      <Select
        id={id}
        value={value || ALL_YEARS}
        onChange={(event) => onChange(event.target.value)}
      >
        <option value={ALL_YEARS}>All years</option>
        {options.map((year) => (
          <option key={year} value={year}>
            {year}
          </option>
        ))}
      </Select>
    </div>
  )
}
