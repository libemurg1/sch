import { call } from "./client"
import type { Payment } from "../types/schema/Payment"
import type { PaymentAllocation } from "../types/schema/PaymentAllocation"
import type { RecordPaymentInput } from "../types/schema/RecordPaymentInput"

export function recordPayment(input: RecordPaymentInput): Promise<Payment> {
  return call<Payment>("record_payment", { input })
}

export function listPayments(): Promise<Payment[]> {
  return call<Payment[]>("list_payments")
}

export function listStudentPayments(studentId: string): Promise<Payment[]> {
  return call<Payment[]>("list_student_payments", { studentId })
}

export function getPayment(id: string): Promise<Payment | null> {
  return call<Payment | null>("get_payment", { id })
}

export function getPaymentAllocations(
  paymentId: string,
): Promise<PaymentAllocation[]> {
  return call<PaymentAllocation[]>("get_payment_allocations", { paymentId })
}
