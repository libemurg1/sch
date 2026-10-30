import { SearchableSelect } from "./SearchableSelect"

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
  return (
    <SearchableSelect
      id={id}
      value={value}
      onChange={onChange}
      options={options}
      placeholder={placeholder}
      searchPlaceholder={searchPlaceholder}
    />
  )
}