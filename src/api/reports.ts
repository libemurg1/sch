import { call } from "./client"

export interface PaymentMethodSummary {
  paymentMethod: string
  paymentCount: number
  amountCents: number
}

export interface TermBalanceSummary {
  termId: string
  label: string
  invoiceCount: number
  billedCents: number
  paidCents: number
  outstandingCents: number
  overdueCents: number
}

export interface FinanceReport {
  startDate: string
  endDate: string
  paymentCount: number
  paymentsReceivedCents: number
  outstandingCents: number
  overdueCents: number
  byMethod: PaymentMethodSummary[]
  byTerm: TermBalanceSummary[]
}

export interface AuditEntry {
  id: number
  actorId: string | null
  actorName: string | null
  entity: string
  entityId: string
  action: string
  detailJson: string | null
  at: string
}

export function getFinanceReport(startDate: string, endDate: string) {
  return call<FinanceReport>("get_finance_report", { startDate, endDate })
}

export function listAuditEntries(limit = 200) {
  return call<AuditEntry[]>("list_audit_entries", { limit })
}
