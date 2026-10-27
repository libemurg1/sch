import { describe, expect, it } from "vitest"
import {
  formatCents,
  parseMoneyToCents,
  parsePercentToBasisPoints,
} from "./money"

describe("money helpers", () => {
  it("formats cents without floating point rounding", () => {
    expect(formatCents(125050)).toBe("1,250.50")
    expect(formatCents(-5)).toBe("-0.05")
  })

  it("parses common currency formats and rounds half up", () => {
    expect(parseMoneyToCents("KSh 1,250.50")).toBe(125050)
    expect(parseMoneyToCents("0.999")).toBe(100)
    expect(parseMoneyToCents("1.234")).toBe(123)
  })

  it("rejects malformed or unsafe monetary values", () => {
    expect(parseMoneyToCents("1.2.3")).toBeNull()
    expect(parseMoneyToCents("9007199254740992")).toBeNull()
  })

  it("parses percentages into integer basis points", () => {
    expect(parsePercentToBasisPoints("10.5")).toBe(1050)
    expect(parsePercentToBasisPoints("1.999")).toBe(200)
    expect(parsePercentToBasisPoints("nope")).toBeNull()
  })
})
