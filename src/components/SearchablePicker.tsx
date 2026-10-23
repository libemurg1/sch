import { useMemo, useState } from "react"
import { Input, Select } from "./Input"

export interface SearchableOption {
  id: string
  label: string
}

export function SearchablePicker({
  id,
  value,
  onChange,
  options,
  placeholder,
  searchPlaceholder,
}: {
  id: string
  value: string
  onChange: (value: string) => void
  options: SearchableOption[]
  placeholder: string
  searchPlaceholder: string
}) {
  const [search, setSearch] = useState("")

  const filteredOptions = useMemo(() => {
    const query = search.trim().toLowerCase()
    return query
      ? options.filter((option) => option.label.toLowerCase().includes(query))
      : options
  }, [options, search])

  return (
    <div className="space-y-2">
      <Input
        id={`${id}-search`}
        value={search}
        onChange={(event) => setSearch(event.target.value)}
        placeholder={searchPlaceholder}
        aria-label={searchPlaceholder}
      />
      <Select
        id={id}
        value={value}
        onChange={(event) => {
          onChange(event.target.value)
          setSearch("")
        }}
      >
        <option value="">{placeholder}</option>
        {filteredOptions.map((option) => (
          <option key={option.id} value={option.id}>
            {option.label}
          </option>
        ))}
      </Select>
    </div>
  )
}