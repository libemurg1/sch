import { call } from "./client"
import type { RegisterStudentInput } from "../types/schema/RegisterStudentInput"
import type { Student } from "../types/schema/Student"
import type { StudentAccount } from "../types/schema/StudentAccount"
import type { UpdateStudentInput } from "../types/schema/UpdateStudentInput"
import type { UpdateStudentStatusInput } from "../types/schema/UpdateStudentStatusInput"

/** Every student, whatever their status. */
export function listStudents(): Promise<Student[]> {
  return call<Student[]>("list_students")
}

/** Students with one specific status. */
export function listStudentsByStatus(status: string): Promise<Student[]> {
  return call<Student[]>("list_students_by_status", { status })
}

export function registerStudent(input: RegisterStudentInput): Promise<Student> {
  return call<Student>("register_student", { input })
}

export function getStudent(id: string): Promise<Student | null> {
  return call<Student | null>("get_student", { id })
}

/** Changes a student's details; the student number and status are immutable. */
export function updateStudent(input: UpdateStudentInput): Promise<Student> {
  return call<Student>("update_student", { input })
}

/** Students are never deleted; they move between soft states instead. */
export function updateStudentStatus(input: UpdateStudentStatusInput): Promise<void> {
  return call<void>("update_student_status", { input })
}

/** What the student owes, what is held as credit, and the net balance. */
export function getStudentAccount(studentId: string): Promise<StudentAccount> {
  return call<StudentAccount>("get_student_account", { studentId })
}
