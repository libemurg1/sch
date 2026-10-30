import { studentName } from "../lib/dates"
import { SearchableSelect } from "./SearchableSelect"

export interface StudentLike {
  id: string
  studentNumber: string
  firstName?: string | null
  lastName?: string | null
  otherNames?: string | null
  middleName?: string | null
  guardianName?: string | null
  guardianPhone?: string | null
  guardianEmail?: string | null
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
  return (
    <SearchableSelect
      id={id}
      value={value}
      onChange={onChange}
      options={students.map((student) => ({
        id: student.id,
        label: `${student.studentNumber} - ${displayStudentName(student)}`,
        searchText: [
          student.studentNumber,
          student.firstName,
          student.lastName,
          student.otherNames,
          student.middleName,
          student.guardianName,
          student.guardianPhone,
          student.guardianEmail,
        ]
          .filter(Boolean)
          .join(" "),
      }))}
      placeholder={placeholder}
      searchPlaceholder="Search name, student number, or guardian"
    />
  )
}
