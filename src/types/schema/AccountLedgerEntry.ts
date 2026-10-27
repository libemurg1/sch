export type AccountLedgerEntry = {
  id: string
  date: string
  entryType: string
  reference: string
  description: string
  debitCents: number
  creditCents: number
  balanceCents: number
  invoiceId: string | null
  paymentId: string | null
}
