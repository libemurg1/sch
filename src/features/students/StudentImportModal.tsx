import { useEffect, useRef, useState } from "react"
import { useImportStudents } from "../../api/queries"
import {
  Button,
  ErrorBanner,
  Modal,
} from "../../components"
import { publishActionNotice } from "../../lib/actionNotice"
import {
  parseStudentCsv,
  studentImportTemplateCsv,
  type StudentCsvPreview,
} from "../../lib/studentCsv"
import type { Student } from "../../types/schema/Student"

export function StudentImportModal({
  open,
  onClose,
  students,
}: {
  open: boolean
  onClose: () => void
  students: Student[]
}) {
  const fileInput = useRef<HTMLInputElement>(null)
  const [fileName, setFileName] = useState("")
  const [preview, setPreview] = useState<StudentCsvPreview | null>(null)
  const [parseError, setParseError] = useState<string | null>(null)
  const mutation = useImportStudents()

  useEffect(() => {
    if (open) {
      setFileName("")
      setPreview(null)
      setParseError(null)
      mutation.reset()
      if (fileInput.current) {
        fileInput.current.value = ""
      }
    }
    // Reset only when opening; mutation is stable but intentionally excluded.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open])

  const chooseFile = async (file?: File) => {
    setPreview(null)
    setParseError(null)
    setFileName(file?.name ?? "")
    if (!file) {
      return
    }
    try {
      const contents = await file.text()
      setPreview(
        parseStudentCsv(
          contents,
          students.map((student) => student.studentNumber),
        ),
      )
    } catch (error) {
      setParseError(
        error instanceof Error ? error.message : "The CSV file could not be read.",
      )
    }
  }

  const downloadTemplate = () => {
    downloadCsv(studentImportTemplateCsv(), "student-import-template.csv")
    publishActionNotice("Student import template downloaded.")
  }

  return (
    <Modal
      open={open}
      onClose={onClose}
      title="Import students from CSV"
      description="Imports student details only. Class and term enrollment must be managed separately."
      size="lg"
      footer={
        <>
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button
            onClick={() => {
              if (preview && preview.errors.length === 0) {
                mutation.mutate(preview.students, { onSuccess: onClose })
              }
            }}
            loading={mutation.isPending}
            disabled={
              !preview ||
              preview.errors.length > 0 ||
              preview.students.length === 0
            }
          >
            Import {preview?.students.length ?? 0} students
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        <p className="text-sm text-slate-600">
          Use the template headers. Required columns are first name, last name,
          date of birth, and admission date. Dates must use YYYY-MM-DD. Existing
          student numbers from an exported file are skipped to avoid importing
          those students twice.
        </p>
        <div className="flex flex-wrap items-center gap-3">
          <Button variant="secondary" onClick={downloadTemplate}>
            Download CSV template
          </Button>
          <Button
            variant="secondary"
            onClick={() => fileInput.current?.click()}
          >
            Choose CSV file
          </Button>
          <input
            ref={fileInput}
            type="file"
            accept=".csv,text/csv"
            className="sr-only"
            onChange={(event) => {
              void chooseFile(event.target.files?.[0])
            }}
          />
          {fileName ? (
            <span className="text-sm text-slate-600">{fileName}</span>
          ) : null}
        </div>

        {parseError ? (
          <div
            className="rounded-md bg-rose-50 px-4 py-3 text-sm text-rose-800"
            role="alert"
          >
            {parseError}
          </div>
        ) : null}
        {preview ? (
          <div className="space-y-3 rounded-md border border-slate-200 p-4">
            <p className="text-sm font-medium text-slate-800">
              {preview.students.length} new student
              {preview.students.length === 1 ? "" : "s"} ready to import
              {preview.skippedExisting > 0
                ? ` · ${preview.skippedExisting} already in this school and will be skipped`
                : ""}
              .
            </p>
            {preview.students.length > 0 ? (
              <ul className="space-y-1 text-sm text-slate-700">
                {preview.students.slice(0, 5).map((student, index) => (
                  <li key={`${student.firstName}-${student.lastName}-${index}`}>
                    {student.firstName} {student.lastName} · born {student.dob} ·
                    admitted {student.admissionDate}
                  </li>
                ))}
                {preview.students.length > 5 ? (
                  <li className="text-slate-500">
                    And {preview.students.length - 5} more student
                    {preview.students.length - 5 === 1 ? "" : "s"}.
                  </li>
                ) : null}
              </ul>
            ) : null}
            {preview.errors.length > 0 ? (
              <div
                className="max-h-48 overflow-y-auto rounded bg-rose-50 p-3 text-sm text-rose-800"
                role="alert"
              >
                <p className="mb-1 font-semibold">
                  Correct these rows before importing:
                </p>
                <ul className="list-inside list-disc">
                  {preview.errors.map((error) => (
                    <li key={error}>{error}</li>
                  ))}
                </ul>
              </div>
            ) : null}
            {preview.students.length === 0 && preview.skippedExisting > 0 ? (
              <p className="text-sm text-slate-600">
                All valid rows already exist. No records will be added.
              </p>
            ) : null}
          </div>
        ) : null}
        {mutation.isError ? <ErrorBanner error={mutation.error} /> : null}
      </div>
    </Modal>
  )
}

function downloadCsv(contents: string, fileName: string) {
  const url = URL.createObjectURL(
    new Blob([contents], { type: "text/csv;charset=utf-8" }),
  )
  const link = document.createElement("a")
  link.href = url
  link.download = fileName
  link.click()
  window.setTimeout(() => URL.revokeObjectURL(url), 0)
}
