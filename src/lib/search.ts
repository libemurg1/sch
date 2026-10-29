export function matchesSearch(query: string, text: string): boolean {
  const terms = normalizeSearch(query).split(/\s+/).filter(Boolean)
  const searchableText = normalizeSearch(text)

  return terms.every((term) => searchableText.includes(term))
}

function normalizeSearch(value: string): string {
  return value.trim().toLocaleLowerCase()
}
