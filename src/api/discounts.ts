import { call } from "./client"
import type { AssignDiscountInput } from "../types/schema/AssignDiscountInput"
import type { CreateDiscountInput } from "../types/schema/CreateDiscountInput"
import type { Discount } from "../types/schema/Discount"
import type { StudentDiscount } from "../types/schema/StudentDiscount"

/** The active discount definitions. */
export function listDiscounts(): Promise<Discount[]> {
  return call<Discount[]>("list_discounts")
}

/**
 * Creates a discount definition.
 *
 * `PERCENT` values are basis points (`1000` = 10.00%), `FIXED` values are
 * cents.
 */
export function createDiscount(input: CreateDiscountInput): Promise<Discount> {
  return call<Discount>("create_discount", { input })
}

/** The discounts granted to one student, optionally within a term. */
export function listStudentDiscounts(
  studentId: string,
  termId: string,
): Promise<StudentDiscount[]> {
  return call<StudentDiscount[]>("list_student_discounts", { studentId, termId })
}

export function listAllStudentDiscounts(): Promise<StudentDiscount[]> {
  return call<StudentDiscount[]>("list_all_student_discounts")
}

export function revokeDiscount(grantId: string, reason: string): Promise<StudentDiscount> {
  return call<StudentDiscount>("revoke_discount", { grantId, reason })
}

/** Grants a discount to a student, optionally scoped to a term. */
export function assignDiscount(
  input: AssignDiscountInput,
): Promise<StudentDiscount> {
  return call<StudentDiscount>("assign_discount", { input })
}
