import { describe, expect, it } from "vitest"
import { exportStudentsCsv, parseStudentCsv, studentImportTemplateCsv } from "./studentCsv"
import type { Student } from "../types/schema/Student"

describe("student CSV", () => {
  it("parses quoted fields and validates dates", () => {
    const result = parseStudentCsv(
      `${studentImportTemplateCsv()}"Ada, Marie",Lovelace,2015-02-28,Female,2026-01-05,"Grace\nHopper",123,grace@example.com`,
      [],
    )

    expect(result.errors).toEqual([])
    expect(result.students).toHaveLength(1)
    expect(result.students[0].firstName).toBe("Ada, Marie")
    expect(result.students[0].guardianName).toBe("Grace\nHopper")
  })

  it("rejects malformed or duplicate rows and skips already imported exports", () => {
    const csv = [
      "studentNumber,firstName,lastName,dob,gender,admissionDate,guardianName,guardianPhone,guardianEmail,status",
      "STU-2026-0001,Ada,Lovelace,2015-01-01,Female,2026-01-01,Grace,,,ACTIVE",
      "STU-2026-0002,Ada,Lovelace,2015-02-30,Female,2026-01-01,Grace,,,ACTIVE",
    ].join("\r\n")

    const result = parseStudentCsv(csv, ["STU-2026-0001"])
    expect(result.skippedExisting).toBe(1)
    expect(result.students).toHaveLength(0)
    expect(result.errors).toEqual([
      "Row 3: date of birth must be YYYY-MM-DD.",
    ])
  })

  it("escapes exported fields so exported data round-trips", () => {
    const student: Student = {
      id: "student-1",
      studentNumber: "STU-2026-0001",
      firstName: 'Ada "A"',
      lastName: "Lovelace",
      dob: "2015-01-01",
      gender: "Female",
      admissionDate: "2026-01-01",
      guardianName: "Grace, Hopper",
      guardianPhone: null,
      guardianEmail: "grace@example.com",
      status: "ACTIVE",
      createdAt: "2026-01-01T00:00:00Z",
    }
    const exported = exportStudentsCsv([student])
    const parsed = parseStudentCsv(exported, [])

    expect(parsed.errors).toEqual([])
    expect(parsed.students[0].firstName).toBe('Ada "A"')
    expect(parsed.students[0].guardianName).toBe("Grace, Hopper")
  })

  it("neutralizes spreadsheet formulas in exported text fields", () => {
    const student: Student = {
      id: "student-2",
      studentNumber: "STU-2026-0002",
      firstName: "=HYPERLINK(\"https://example.invalid\")",
      lastName: "Example",
      dob: "2015-01-01",
      gender: null,
      admissionDate: "2026-01-01",
      guardianName: null,
      guardianPhone: null,
      guardianEmail: null,
      status: "ACTIVE",
      createdAt: "2026-01-01T00:00:00Z",
    }

    expect(exportStudentsCsv([student])).toContain(
      "'=HYPERLINK(\"\"https://example.invalid\"\")",
    )
  })
})
