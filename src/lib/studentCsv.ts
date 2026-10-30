import type { Student } from "../types/schema/Student"
import type { RegisterStudentInput } from "../types/schema/RegisterStudentInput"

export const STUDENT_IMPORT_HEADERS = [
  "firstName",
  "lastName",
  "dob",
  "gender",
  "admissionDate",
  "guardianName",
  "guardianPhone",
  "guardianEmail",
]

const EXPORT_HEADERS = [
  "studentNumber",
  ...STUDENT_IMPORT_HEADERS,
  "status",
]

export interface StudentCsvPreview {
  students: RegisterStudentInput[]
  errors: string[]
  skippedExisting: number
  rowCount: number
}

export function parseStudentCsv(
  input: string,
  existingStudentNumbers: string[],
): StudentCsvPreview {
  const rows = parseRows(input.replace(/^\uFEFF/, ""))
  if (rows.length === 0) {
    throw new Error("The CSV file is empty.")
  }

  const headers = rows[0].map(normalizeHeader)
  const headerIndexes = new Map<string, number>()
  headers.forEach((header, index) => {
    if (headerIndexes.has(header)) {
      throw new Error(`The CSV has a duplicate "${rows[0][index]}" column.`)
    }
    headerIndexes.set(header, index)
  })

  const requiredHeaders = ["firstname", "lastname", "dob", "admissiondate"]
  const missing = requiredHeaders.filter((header) => !headerIndexes.has(header))
  if (missing.length > 0) {
    throw new Error(
      `Missing required columns: ${missing.map(displayHeader).join(", ")}.`,
    )
  }

  const existing = new Set(
    existingStudentNumbers.map((number) => number.trim().toLocaleLowerCase()),
  )
  const seenNumbers = new Set<string>()
  const seenRecords = new Set<string>()
  const errors: string[] = []
  const students: RegisterStudentInput[] = []
  let skippedExisting = 0
  let rowCount = 0

  for (const [index, row] of rows.slice(1).entries()) {
    if (row.every((value) => value.trim() === "")) {
      continue
    }
    rowCount += 1
    const rowNumber = index + 2
    const value = (header: string) =>
      (row[headerIndexes.get(header) ?? -1] ?? "").trim()

    const genderValue = value("gender")
    const genderByLowercase: Record<string, string> = {
      female: "Female",
      male: "Male",
      other: "Other",
    }
    const gender = genderByLowercase[genderValue.toLocaleLowerCase()] ?? null
    const student: RegisterStudentInput = {
      firstName: value("firstname"),
      lastName: value("lastname"),
      dob: value("dob"),
      gender,
      admissionDate: value("admissiondate"),
      guardianName: value("guardianname") || null,
      guardianPhone: value("guardianphone") || null,
      guardianEmail: value("guardianemail") || null,
    }
    const rowErrors: string[] = []
    if (!student.firstName) rowErrors.push("first name is required")
    if (!student.lastName) rowErrors.push("last name is required")
    if (!isIsoDate(student.dob)) rowErrors.push("date of birth must be YYYY-MM-DD")
    if (!isIsoDate(student.admissionDate)) {
      rowErrors.push("admission date must be YYYY-MM-DD")
    }
    if (genderValue && !student.gender) {
      rowErrors.push("gender must be Female, Male, Other, or blank")
    }
    if (
      student.guardianEmail &&
      !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(student.guardianEmail)
    ) {
      rowErrors.push("guardian email must be a valid email address")
    }
    if (rowErrors.length > 0) {
      errors.push(`Row ${rowNumber}: ${rowErrors.join("; ")}.`)
      continue
    }

    const importedNumber = value("studentnumber").toLocaleLowerCase()
    if (importedNumber) {
      if (seenNumbers.has(importedNumber)) {
        errors.push(`Row ${rowNumber}: student number appears more than once.`)
        continue
      }
      seenNumbers.add(importedNumber)
      if (existing.has(importedNumber)) {
        skippedExisting += 1
        continue
      }
    }

    const fingerprint = [
      student.firstName,
      student.lastName,
      student.dob,
      student.admissionDate,
    ]
      .join("|")
      .toLocaleLowerCase()
    if (seenRecords.has(fingerprint)) {
      errors.push(`Row ${rowNumber}: this student appears more than once.`)
      continue
    }
    seenRecords.add(fingerprint)
    students.push(student)
  }

  if (rowCount === 0) {
    errors.push("The CSV does not contain any student rows.")
  }

  return { students, errors, skippedExisting, rowCount }
}

export function studentImportTemplateCsv(): string {
  return `${STUDENT_IMPORT_HEADERS.join(",")}\r\n`
}

export function exportStudentsCsv(students: Student[]): string {
  const records = [
    EXPORT_HEADERS,
    ...students.map((student) => [
      student.studentNumber,
      student.firstName,
      student.lastName,
      student.dob,
      student.gender ?? "",
      student.admissionDate,
      student.guardianName ?? "",
      student.guardianPhone ?? "",
      student.guardianEmail ?? "",
      student.status,
    ]),
  ]
  return `\uFEFF${records.map((record) => record.map(escapeCell).join(",")).join("\r\n")}\r\n`
}

function parseRows(input: string): string[][] {
  const rows: string[][] = []
  let row: string[] = []
  let cell = ""
  let quoted = false

  for (let index = 0; index < input.length; index += 1) {
    const character = input[index]
    if (quoted) {
      if (character === '"') {
        if (input[index + 1] === '"') {
          cell += '"'
          index += 1
        } else {
          quoted = false
        }
      } else {
        cell += character
      }
      continue
    }

    if (character === '"' && cell.length === 0) {
      quoted = true
    } else if (character === '"') {
      throw new Error("The CSV contains a quote in an unquoted field.")
    } else if (character === ",") {
      row.push(cell)
      cell = ""
    } else if (character === "\n" || character === "\r") {
      row.push(cell)
      rows.push(row)
      row = []
      cell = ""
      if (character === "\r" && input[index + 1] === "\n") {
        index += 1
      }
    } else {
      cell += character
    }
  }
  if (quoted) {
    throw new Error("The CSV has an unclosed quoted field.")
  }
  if (cell.length > 0 || row.length > 0) {
    row.push(cell)
    rows.push(row)
  }
  return rows
}

function normalizeHeader(value: string): string {
  return value.trim().toLocaleLowerCase().replace(/[\s_-]/g, "")
}

function displayHeader(value: string): string {
  return value.replace(/([a-z])([A-Z])/g, "$1 $2")
}

function isIsoDate(value: string): boolean {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    return false
  }
  const date = new Date(`${value}T00:00:00.000Z`)
  return !Number.isNaN(date.getTime()) && date.toISOString().slice(0, 10) === value
}

function escapeCell(value: string): string {
  const safeValue = /^[\s]*[=+\-@]/.test(value) ? `'${value}` : value
  return /[",\r\n]/.test(safeValue)
    ? `"${safeValue.replace(/"/g, '""')}"`
    : safeValue
}
