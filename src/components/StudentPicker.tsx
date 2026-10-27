import { useMemo, useState } from "react"
import { studentName } from "../lib/dates"
import { Input, Select } from "./Input"

export interface StudentLike {
  id: string
  studentNumber: string
  firstName?: string | null
  lastName?: string | null
  otherNames?: string | null
  middleName?: string | null
}

function displayStudentName(student: StudentLike): string {
  return studentName({
    firstName: student.firstName ?? "",
    lastName: student.lastName ?? "",
  })
}

export function StudentPicker<T extends StudentLike>({
  id,
  value,
  onChange,
  students,
  placeholder = "Choose a student",
}: {
  id: string
  value: string
  onChange: (value: string) => void
  students: T[]
  placeholder?: string
}) {
  const [search, setSearch] = useState("")

  const filteredStudents = useMemo(() => {
    const term = search.trim().toLowerCase()
    if (!term) {
      return students
    }

    return students.filter((student) => {
      const label = `${student.studentNumber} ${displayStudentName(student)}`.toLowerCase()
      return label.includes(term)
    })
  }, [search, students])

  return (
    <div className="space-y-2">
      <Input
        id={`${id}-search`}
        value={search}
        onChange={(event) => setSearch(event.target.value)}
        placeholder="Search by name or student number"
      />
      <Select
        id={id}
        value={value}
        onChange={(event) => onChange(event.target.value)}
      >
        <option value="">{placeholder}</option>
        {filteredStudents.map((student) => (
          <option key={student.id} value={student.id}>
            {student.studentNumber} - {displayStudentName(student)}
          </option>
        ))}
      </Select>
    </div>
  )
}
