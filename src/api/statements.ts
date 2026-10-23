import { call } from "./client"
import type { InvoiceDetail } from "../types/schema/InvoiceDetail"
import type { StudentStatement } from "../types/schema/StudentStatement"

/**
 * The full billing picture for one student: their invoices, payments and the
 * resulting balance. `termId` is optional and can be `null` to cover every
 * term.
 */
export function getStudentStatement(
  studentId: string,
  termId: string | null,
): Promise<StudentStatement> {
  return call<StudentStatement>("get_student_statement", { studentId, termId })
}

/** An invoice together with its snapshotted line items. */
export function getInvoiceDetail(invoiceId: string): Promise<InvoiceDetail> {
  return call<InvoiceDetail>("get_invoice_detail", { invoiceId })
}
