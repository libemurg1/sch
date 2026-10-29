import { useMemo, useRef, useState } from "react"
import { matchesSearch } from "../lib/search"
import { Input } from "./Input"

export interface SearchableSelectOption {
  id: string
  label: string
  searchText?: string
}

export function SearchableSelect({
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
  options: SearchableSelectOption[]
  placeholder: string
  searchPlaceholder: string
}) {
  const [open, setOpen] = useState(false)
  const [search, setSearch] = useState("")
  const [activeIndex, setActiveIndex] = useState(0)
  const containerRef = useRef<HTMLDivElement>(null)

  const filteredOptions = useMemo(
    () =>
      options.filter((option) =>
        matchesSearch(search, option.searchText ?? option.label),
      ),
    [options, search],
  )
  const visibleOptions = search.trim()
    ? filteredOptions
    : [{ id: "", label: placeholder }, ...options]
  const selectedOption = options.find((option) => option.id === value)
  const activeOption = visibleOptions[activeIndex]
  const resultsId = `${id}-results`
  const listboxId = `${id}-listbox`

  function openDropdown() {
    setOpen(true)
    setSearch("")
    const selectedIndex = options.findIndex((option) => option.id === value)
    setActiveIndex(selectedIndex === -1 ? 0 : selectedIndex + 1)
  }

  function chooseOption(option: SearchableSelectOption) {
    onChange(option.id)
    setSearch("")
    setOpen(false)
  }

  function handleKeyDown(event: React.KeyboardEvent<HTMLInputElement>) {
    if (event.key === "ArrowDown") {
      event.preventDefault()
      if (!open) {
        openDropdown()
        return
      }
      setActiveIndex((index) => Math.min(index + 1, visibleOptions.length - 1))
    } else if (event.key === "ArrowUp") {
      event.preventDefault()
      if (!open) {
        openDropdown()
        return
      }
      setActiveIndex((index) => Math.max(index - 1, 0))
    } else if (event.key === "Enter" && open && activeOption) {
      event.preventDefault()
      chooseOption(activeOption)
    } else if (event.key === "Escape" && open) {
      event.preventDefault()
      setOpen(false)
      setSearch("")
    }
  }

  return (
    <div
      ref={containerRef}
      className="relative"
      onBlur={(event) => {
        if (!containerRef.current?.contains(event.relatedTarget as Node | null)) {
          setOpen(false)
          setSearch("")
        }
      }}
    >
      <Input
        id={id}
        role="combobox"
        autoComplete="off"
        value={open ? search : selectedOption?.label ?? ""}
        placeholder={open ? searchPlaceholder : placeholder}
        aria-label={searchPlaceholder}
        aria-autocomplete="list"
        aria-expanded={open}
        aria-controls={listboxId}
        aria-activedescendant={open && activeOption ? `${id}-option-${activeIndex}` : undefined}
        aria-describedby={open && search.trim() ? resultsId : undefined}
        className="pr-9"
        onFocus={openDropdown}
        onChange={(event) => {
          setSearch(event.target.value)
          setActiveIndex(0)
          setOpen(true)
        }}
        onKeyDown={handleKeyDown}
      />
      <svg
        aria-hidden="true"
        viewBox="0 0 20 20"
        fill="none"
        className="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500"
      >
        <path
          d="m6 8 4 4 4-4"
          stroke="currentColor"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="1.5"
        />
      </svg>
      {open ? (
        <div className="absolute z-30 mt-1 w-full overflow-hidden rounded-md bg-white shadow-lg ring-1 ring-slate-300">
          {search.trim() ? (
            <p
              id={resultsId}
              className="px-3 py-2 text-xs text-slate-500"
              role="status"
              aria-live="polite"
            >
              {filteredOptions.length === 0
                ? "No matches. Try another search."
                : `${filteredOptions.length} matching ${filteredOptions.length === 1 ? "option" : "options"}`}
            </p>
          ) : null}
          {visibleOptions.length > 0 ? (
            <ul
              id={listboxId}
              role="listbox"
              aria-label={searchPlaceholder}
              className="max-h-60 overflow-y-auto py-1"
            >
              {visibleOptions.map((option, index) => (
                <li
                  id={`${id}-option-${index}`}
                  key={option.id || "empty"}
                  role="option"
                  aria-selected={option.id === value}
                  className={`cursor-pointer px-3 py-2 text-sm ${
                    index === activeIndex
                      ? "bg-indigo-50 text-indigo-900"
                      : "text-slate-900 hover:bg-slate-50"
                  }`}
                  onMouseDown={(event) => event.preventDefault()}
                  onMouseEnter={() => setActiveIndex(index)}
                  onClick={() => chooseOption(option)}
                >
                  {option.label}
                </li>
              ))}
            </ul>
          ) : null}
        </div>
      ) : null}
    </div>
  )
}
