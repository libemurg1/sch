import { call } from "./client"
import type { AssignStudentFeeInput } from "../types/schema/AssignStudentFeeInput"
import type { GenerateBatchInput } from "../types/schema/GenerateBatchInput"
import type { GenerateSupplementaryInput } from "../types/schema/GenerateSupplementaryInput"
import type { Invoice } from "../types/schema/Invoice"
import type { InvoiceItem } from "../types/schema/InvoiceItem"
import type { StudentFee } from "../types/schema/StudentFee"

/**
 * Raises one TERM invoice per enrolled student in a class.
 *
 * Students who already have a live invoice for the term are skipped, so the
 * batch can be run again safely.
 */
export function generateInvoiceBatch(
  input: GenerateBatchInput,
): Promise<Invoice[]> {
  return call<Invoice[]>("generate_invoice_batch", { input })
}

/**
 * Raises a SUPPLEMENTARY invoice for a student's pending ad-hoc fees.
 *
 * Returns `null` when the student has nothing pending.
 */
export function generateSupplementaryInvoice(
  input: GenerateSupplementaryInput,
): Promise<Invoice | null> {
  return call<Invoice | null>("generate_supplementary_invoice", { input })
}

/** Records an ad-hoc charge that stays pending until it is billed. */
export function assignStudentFee(
  input: AssignStudentFeeInput,
): Promise<StudentFee> {
  return call<StudentFee>("assign_student_fee", { input })
}

export function listInvoices(): Promise<Invoice[]> {
  return call<Invoice[]>("list_invoices")
}

export function listStudentInvoices(studentId: string): Promise<Invoice[]> {
  return call<Invoice[]>("list_student_invoices", { studentId })
}

export function listClassInvoices(
  classId: string,
  termId: string,
): Promise<Invoice[]> {
  return call<Invoice[]>("list_class_invoices", { classId, termId })
}

export function getInvoice(id: string): Promise<Invoice | null> {
  return call<Invoice | null>("get_invoice", { id })
}

export function getInvoiceItems(invoiceId: string): Promise<InvoiceItem[]> {
  return call<InvoiceItem[]>("get_invoice_items", { invoiceId })
}
