# School Fees System (SFS)

Offline-first desktop app for registering students, setting term fees, batch billing a class, recording payments, printing receipts/statements. Built with Tauri v2 + React + TypeScript + Rust + SQLite.

## Tech Stack
- **Frontend**: React 19 + TypeScript + Vite + TailwindCSS + Zod + TanStack Query
- **Backend**: Rust + Tauri v2 + rusqlite (bundled) with WAL
- **Database**: SQLite (single file `school.db`), all business rules in Rust transactions
- **Printing/PDF**: Webview HTML print + @media print (A4/Letter for invoices/statements, 80mm receipt)

## Core Principles
- **Money as integer cents** (i64) — no floating point for totals/balances
- **All access via Tauri commands only** — no SQL plugin from React
- **Snapshot invoices** — invoice items copied at billing time; fee changes never rewrite issued invoices
- **Soft state only** — no hard delete of financial rows (use CANCELLED/INACTIVE)
- **UUIDs for rows, human numbers** — STU-YYYY-0001, INV-YYYY-0003, REC-YYYY-0004 from counters
- **Business rules in transactions** — batch billing and payments are atomic
- **One invoice per class batch** — student in two classes gets two TERM invoices in same term
- **Multi-invoice payments** — one payment can cover multiple invoices via PAYMENT_ALLOCATIONS
- **Discounts** — both FIXED (cents) and PERCENT (basis points, 1-10000), stackable, targeted to specific fee items or whole invoice, each saved as negative line
- **Supplementary invoices** — late fees create SUPPLEMENTARY invoices; fixed discounts not re-applied in same term to late fees
- **Real accounts, attribution only** — login required, full access for all, stamps audit trail and receipts

## Project Structure
```
school-fees/
├─ src/                          # React + TypeScript frontend
│  ├─ app/                       # router, providers, layout shell
│  ├─ features/                  # students, enrollment, fee-structures, billing, payments, statements
│  ├─ components/                # shared UI only
│  ├─ api/                       # invoke() client, error mapping
│  ├─ reports/                   # StatementDocument, InvoiceDocument, ReceiptDocument
│  ├─ lib/                       # money.ts, dates.ts, validation (zod)
│  └─ types/schema.ts            # generated from Rust
├─ src-tauri/
│  ├─ migrations/                # 0001_init.sql, ...
│  └─ src/
│     ├─ main.rs                 # composition root, DI wiring
│     ├─ commands/               # Tauri command handlers
│     ├─ application/            # services (use cases)
│     ├─ domain/                 # entities, Money, errors, ports
│     └─ infrastructure/         # sqlite repos, unit_of_work, backup, number_generator
└─ docs/                         # design docs, ADRs
```

## Discount Rules
- PERCENT value in basis points (e.g., 1000 = 10.00%), max 10000 (100%)
- FIXED value in cents
- Apply to discountable base only (items with discountable=1)
- Multiple percentages add together (not compounded), each rounded half up, capped at 100% of discountable base
- Fixed discounts come off whole invoice
- All discounts stack, no per-discount cap (only final clamp so invoice never below zero) — total discount capped at subtotal
- Each discount saved as separate negative INVOICE_ITEM (kind=DISCOUNT)
- Targeted: STUDENT_DISCOUNTS.target_student_fee_id or target_fee_item_id (nullable = whole invoice)
- Late discount: add negative line to existing invoice if new total >= paid_cents
- Fixed discounts not re-applied to supplementary invoices for same term (only percentage discounts apply to discountable lines of supplementary)

## Payment Rules
- One payment can cover multiple invoices via PAYMENT_ALLOCATIONS
- Payment belongs to student; allocations split amount across invoices
- Overpayment impossible (CHECK paid_cents <= total_cents)
- Invoice status derived: UNPAID (paid=0), PARTIAL (0<payed<total), PAID (paid==total), CANCELLED
- Same reference not reused for same method (except CASH)

## Billing Rules
- Batch billing: for each enrolled student in class+term, create TERM invoice from FEE_STRUCTURE (snapshot items)
- One live TERM invoice per student per term (partial unique index)
- Student in two classes → two TERM invoices
- Late student fee added after billing → create SUPPLEMENTARY invoice with pending fees
- Supplementary invoices have fee_structure_id=NULL, kind=SUPPLEMENTARY
- Each discount as negative line

## Numbering
- Students: STU-{year}-{seq:04}
- Invoices: INV-{year}-{seq:03}
- Receipts: REC-{year}-{seq:04}
- Generated inside transaction via NUMBER_COUNTERS

## Printing
- Invoice/Statement: A4/Letter via webview print, CSS @media print
- Receipt: 80mm thermal slip via webview print with 80mm stylesheet
- PDF export via "Save as PDF" in webview

## Database
All migrations in `src-tauri/migrations/` applied at startup with PRAGMA user_version. Backup created before any migration. App refuses to start if DB is newer than understood.
