import { call } from "./client"
import type { Class } from "../types/schema/Class"
import type { CreateClassInput } from "../types/schema/CreateClassInput"
import type { CreateTermInput } from "../types/schema/CreateTermInput"
import type { Enrollment } from "../types/schema/Enrollment"
import type { EnrollStudentInput } from "../types/schema/EnrollStudentInput"
import type { Grade } from "../types/schema/Grade"
import type { MoveStudentInput } from "../types/schema/MoveStudentInput"
import type { Term } from "../types/schema/Term"
import type { UpdateClassInput } from "../types/schema/UpdateClassInput"
import type { UpdateTermInput } from "../types/schema/UpdateTermInput"

/** All classes, or only the classes that belong to `year`. */
export function listClasses(year?: number): Promise<Class[]> {
  return call<Class[]>("list_classes", { year: year ?? null })
}

export function listGrades(): Promise<Grade[]> {
  return call<Grade[]>("list_grades")
}

export function createClass(input: CreateClassInput): Promise<Class> {
  return call<Class>("create_class", { input })
}

export function updateClass(input: UpdateClassInput): Promise<Class> {
  return call<Class>("update_class", { input })
}

export function deleteClass(classId: string): Promise<void> {
  return call("delete_class", { classId })
}

export function listTerms(): Promise<Term[]> {
  return call<Term[]>("list_terms")
}

/** The single term marked active, if there is one. */
export function getActiveTerm(): Promise<Term | null> {
  return call<Term | null>("get_active_term")
}

/** Creating an active term deactivates the previous one. */
export function createTerm(input: CreateTermInput): Promise<Term> {
  return call<Term>("create_term", { input })
}

export function updateTerm(input: UpdateTermInput): Promise<Term> {
  return call<Term>("update_term", { input })
}

export function deleteTerm(termId: string): Promise<void> {
  return call("delete_term", { termId })
}

export function enrollStudent(input: EnrollStudentInput): Promise<Enrollment> {
  return call<Enrollment>("enroll_student", { input })
}

export function listEnrollments(
  classId: string,
  termId: string,
): Promise<Enrollment[]> {
  return call<Enrollment[]>("list_enrollments", { classId, termId })
}

export function listEnrollmentsByTerm(termId: string): Promise<Enrollment[]> {
  return call<Enrollment[]>("list_enrollments_by_term", { termId })
}

export function listStudentEnrollments(studentId: string): Promise<Enrollment[]> {
  return call<Enrollment[]>("list_student_enrollments", { studentId })
}

/** Moves a student to another class (stream) within the same term. */
export function moveStudent(input: MoveStudentInput): Promise<Enrollment> {
  return call<Enrollment>("move_student", { input })
}
