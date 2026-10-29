import { describe, expect, it } from "vitest"
import { matchesSearch } from "./search"

describe("matchesSearch", () => {
  it("matches case-insensitively and ignores extra whitespace", () => {
    expect(matchesSearch("  TERM   2 ", "Term 2, 2026")).toBe(true)
  })

  it("matches every word in any order across the searchable label", () => {
    expect(matchesSearch("2026 term", "Term 2, 2026")).toBe(true)
    expect(matchesSearch("grade 2", "Grade 2 Blue")).toBe(true)
  })

  it("rejects a result if any query word is missing", () => {
    expect(matchesSearch("grade 3", "Grade 2 Blue")).toBe(false)
    expect(matchesSearch("   ", "Any value")).toBe(true)
  })
})
