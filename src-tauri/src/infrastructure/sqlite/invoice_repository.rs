use crate::domain::entities::{Invoice, InvoiceItem};
use crate::domain::errors::{AppError, AppResult};
use crate::domain::ports::InvoiceRepository;
use crate::infrastructure::db::with_optional_transaction;
use rusqlite::{params, Connection};

pub struct SqliteInvoiceRepository<'c> {
    conn: &'c Connection,
}

impl<'c> SqliteInvoiceRepository<'c> {
    pub fn new(conn: &'c Connection) -> Self {
        Self { conn }
    }
}

const INVOICE_COLUMNS: &str = "id, invoice_number, kind, student_id, term_id, fee_structure_id, \
     issue_date, due_date, status, subtotal_cents, discount_cents, total_cents, paid_cents";

fn map_invoice(row: &rusqlite::Row<'_>) -> rusqlite::Result<Invoice> {
    Ok(Invoice {
        id: row.get(0)?,
        invoice_number: row.get(1)?,
        kind: row.get(2)?,
        student_id: row.get(3)?,
        term_id: row.get(4)?,
        fee_structure_id: row.get(5)?,
        issue_date: row.get(6)?,
        due_date: row.get(7)?,
        status: row.get(8)?,
        subtotal_cents: row.get(9)?,
        discount_cents: row.get(10)?,
        total_cents: row.get(11)?,
        paid_cents: row.get(12)?,
    })
}

fn map_invoice_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<InvoiceItem> {
    Ok(InvoiceItem {
        id: row.get(0)?,
        invoice_id: row.get(1)?,
        kind: row.get(2)?,
        description: row.get(3)?,
        amount_cents: row.get(4)?,
        discountable: row.get::<_, i32>(5)? != 0,
        discount_id: row.get(6)?,
        sort_order: row.get(7)?,
    })
}

impl InvoiceRepository for SqliteInvoiceRepository<'_> {
    fn create(&self, invoice: &Invoice) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO invoices (id, invoice_number, kind, student_id, term_id, fee_structure_id, \
             issue_date, due_date, status, subtotal_cents, discount_cents, total_cents, paid_cents) \
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)",
            params![
                invoice.id,
                invoice.invoice_number,
                invoice.kind,
                invoice.student_id,
                invoice.term_id,
                invoice.fee_structure_id,
                invoice.issue_date,
                invoice.due_date,
                invoice.status,
                invoice.subtotal_cents,
                invoice.discount_cents,
                invoice.total_cents,
                invoice.paid_cents,
            ],
        )?;
        Ok(())
    }

    fn create_items(&self, invoice_id: &str, items: &[InvoiceItem]) -> AppResult<()> {
        with_optional_transaction(self.conn, || {
            for item in items {
                self.conn.execute(
                    "INSERT INTO invoice_items (id, invoice_id, kind, description, amount_cents, \
                     discountable, discount_id, sort_order) VALUES (?,?,?,?,?,?,?,?)",
                    params![
                        item.id,
                        invoice_id,
                        item.kind,
                        item.description,
                        item.amount_cents,
                        i32::from(item.discountable),
                        item.discount_id,
                        item.sort_order,
                    ],
                )?;
            }
            Ok(())
        })
    }

    fn list_items(&self, invoice_id: &str) -> AppResult<Vec<InvoiceItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, invoice_id, kind, description, amount_cents, discountable, discount_id, \
             sort_order FROM invoice_items WHERE invoice_id = ? ORDER BY sort_order, id",
        )?;
        let rows = stmt.query_map(params![invoice_id], map_invoice_item)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn find_live_by_student_term(
        &self,
        student_id: &str,
        term_id: &str,
    ) -> AppResult<Option<Invoice>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM invoices WHERE student_id = ? AND term_id = ? AND kind = 'TERM' \
             AND status <> 'CANCELLED'",
            INVOICE_COLUMNS
        ))?;
        let mut rows = stmt.query(params![student_id, term_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_invoice(row)?)),
            None => Ok(None),
        }
    }

    fn list(&self) -> AppResult<Vec<Invoice>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM invoices ORDER BY issue_date DESC, invoice_number DESC",
            INVOICE_COLUMNS
        ))?;
        let rows = stmt.query_map([], map_invoice)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn list_by_student(&self, student_id: &str) -> AppResult<Vec<Invoice>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM invoices WHERE student_id = ? ORDER BY issue_date DESC, invoice_number DESC",
            INVOICE_COLUMNS
        ))?;
        let rows = stmt.query_map(params![student_id], map_invoice)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn list_by_class_term(&self, class_id: &str, term_id: &str) -> AppResult<Vec<Invoice>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM invoices WHERE term_id = ? AND status <> 'CANCELLED' \
             AND student_id IN (SELECT student_id FROM enrollments WHERE class_id = ? AND term_id = ?) \
             ORDER BY invoice_number",
            INVOICE_COLUMNS
        ))?;
        let rows = stmt.query_map(params![term_id, class_id, term_id], map_invoice)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn find_by_id(&self, id: &str) -> AppResult<Option<Invoice>> {
        let mut stmt = self
            .conn
            .prepare(&format!("SELECT {} FROM invoices WHERE id = ?", INVOICE_COLUMNS))?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_invoice(row)?)),
            None => Ok(None),
        }
    }

    fn apply_payment(&self, invoice_id: &str, amount_cents: i64) -> AppResult<Invoice> {
        with_optional_transaction(self.conn, || {
            let invoice = self
                .find_by_id(invoice_id)?
                .ok_or_else(|| AppError::not_found("Invoice", invoice_id))?;

            if invoice.status == "CANCELLED" {
                return Err(AppError::conflict(format!(
                    "Invoice {} is cancelled and cannot accept payments",
                    invoice.invoice_number
                )));
            }

            let paid = invoice.paid_cents + amount_cents;
            if paid > invoice.total_cents {
                return Err(AppError::conflict(format!(
                    "Payment of {} exceeds the outstanding balance of {} on invoice {}",
                    amount_cents,
                    invoice.total_cents - invoice.paid_cents,
                    invoice.invoice_number
                )));
            }

            let status = if paid == 0 {
                "UNPAID"
            } else if paid < invoice.total_cents {
                "PARTIAL"
            } else {
                "PAID"
            };

            self.conn.execute(
                "UPDATE invoices SET paid_cents = ?, status = ? WHERE id = ?",
                params![paid, status, invoice_id],
            )?;

            self.find_by_id(invoice_id)?
                .ok_or_else(|| AppError::not_found("Invoice", invoice_id))
        })
    }
}
