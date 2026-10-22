use crate::domain::entities::*;
use crate::domain::errors::AppResult;
use crate::domain::ports::InvoiceRepository;
use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};

pub struct SqliteInvoiceRepository {
    conn: Arc<Mutex<Connection>>,
}
impl SqliteInvoiceRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self { Self { conn } }
}
impl InvoiceRepository for SqliteInvoiceRepository {
    fn create(&self, invoice: &Invoice) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO invoices (id, invoice_number, kind, student_id, term_id, fee_structure_id, issue_date, due_date, status, subtotal_cents, discount_cents, total_cents, paid_cents) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)",
            params![invoice.id, invoice.invoice_number, invoice.kind, invoice.student_id, invoice.term_id, invoice.fee_structure_id, invoice.issue_date, invoice.due_date, invoice.status, invoice.subtotal_cents, invoice.discount_cents, invoice.total_cents, invoice.paid_cents],
        )?;
        Ok(())
    }
    fn create_items(&self, invoice_id: &str, items: &[InvoiceItem]) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        for item in items {
            conn.execute(
                "INSERT INTO invoice_items (id, invoice_id, kind, description, amount_cents, discountable, discount_id, sort_order) VALUES (?,?,?,?,?,?,?,?)",
                params![item.id, invoice_id, item.kind, item.description, item.amount_cents, if item.discountable { 1i32 } else { 0i32 }, item.discount_id, item.sort_order],
            )?;
        }
        Ok(())
    }
    fn find_live_by_student_term(&self, student_id: &str, term_id: &str) -> AppResult<Option<Invoice>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, invoice_number, kind, student_id, term_id, fee_structure_id, issue_date, due_date, status, subtotal_cents, discount_cents, total_cents, paid_cents FROM invoices WHERE student_id = ? AND term_id = ? AND kind = 'TERM' AND status <> 'CANCELLED'")?;
        let mut rows = stmt.query(params![student_id, term_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Invoice {
                id: row.get(0)?, invoice_number: row.get(1)?, kind: row.get(2)?, student_id: row.get(3)?, term_id: row.get(4)?, fee_structure_id: row.get(5)?, issue_date: row.get(6)?, due_date: row.get(7)?, status: row.get(8)?, subtotal_cents: row.get(9)?, discount_cents: row.get(10)?, total_cents: row.get(11)?, paid_cents: row.get(12)?,
            }))
        } else { Ok(None) }
    }
    fn list(&self) -> AppResult<Vec<Invoice>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, invoice_number, kind, student_id, term_id, fee_structure_id, issue_date, due_date, status, subtotal_cents, discount_cents, total_cents, paid_cents FROM invoices ORDER BY issue_date DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Invoice {
                id: row.get(0)?, invoice_number: row.get(1)?, kind: row.get(2)?, student_id: row.get(3)?, term_id: row.get(4)?, fee_structure_id: row.get(5)?, issue_date: row.get(6)?, due_date: row.get(7)?, status: row.get(8)?, subtotal_cents: row.get(9)?, discount_cents: row.get(10)?, total_cents: row.get(11)?, paid_cents: row.get(12)?,
            })
        })?;
        let mut res = Vec::new();
        for r in rows { res.push(r?); }
        Ok(res)
    }
    fn find_by_id(&self, id: &str) -> AppResult<Option<Invoice>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, invoice_number, kind, student_id, term_id, fee_structure_id, issue_date, due_date, status, subtotal_cents, discount_cents, total_cents, paid_cents FROM invoices WHERE id = ?")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Invoice {
                id: row.get(0)?, invoice_number: row.get(1)?, kind: row.get(2)?, student_id: row.get(3)?, term_id: row.get(4)?, fee_structure_id: row.get(5)?, issue_date: row.get(6)?, due_date: row.get(7)?, status: row.get(8)?, subtotal_cents: row.get(9)?, discount_cents: row.get(10)?, total_cents: row.get(11)?, paid_cents: row.get(12)?,
            }))
        } else { Ok(None) }
    }
}
