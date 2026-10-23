use crate::domain::entities::{Payment, PaymentAllocation};
use crate::domain::errors::AppResult;
use crate::domain::ports::{CreditSource, PaymentRepository};
use crate::infrastructure::db::with_optional_transaction;
use rusqlite::{params, Connection};

pub struct SqlitePaymentRepository<'c> {
    conn: &'c Connection,
}

impl<'c> SqlitePaymentRepository<'c> {
    pub fn new(conn: &'c Connection) -> Self {
        Self { conn }
    }
}

const PAYMENT_COLUMNS: &str = "id, receipt_number, student_id, amount_cents, payment_date, \
     payment_method, reference_number, created_by, created_at";

fn map_payment(row: &rusqlite::Row<'_>) -> rusqlite::Result<Payment> {
    Ok(Payment {
        id: row.get(0)?,
        receipt_number: row.get(1)?,
        student_id: row.get(2)?,
        amount_cents: row.get(3)?,
        payment_date: row.get(4)?,
        payment_method: row.get(5)?,
        reference_number: row.get(6)?,
        created_by: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn map_allocation(row: &rusqlite::Row<'_>) -> rusqlite::Result<PaymentAllocation> {
    Ok(PaymentAllocation {
        id: row.get(0)?,
        payment_id: row.get(1)?,
        invoice_id: row.get(2)?,
        amount_cents: row.get(3)?,
    })
}

impl PaymentRepository for SqlitePaymentRepository<'_> {
    fn create(&self, payment: &Payment, allocations: &[PaymentAllocation]) -> AppResult<()> {
        with_optional_transaction(self.conn, || {
            self.conn.execute(
                "INSERT INTO payments (id, receipt_number, student_id, amount_cents, payment_date, \
                 payment_method, reference_number, created_by) VALUES (?,?,?,?,?,?,?,?)",
                params![
                    payment.id,
                    payment.receipt_number,
                    payment.student_id,
                    payment.amount_cents,
                    payment.payment_date,
                    payment.payment_method,
                    payment.reference_number,
                    payment.created_by,
                ],
            )?;

            for allocation in allocations {
                self.conn.execute(
                    "INSERT INTO payment_allocations (id, payment_id, invoice_id, amount_cents) \
                     VALUES (?,?,?,?)",
                    params![
                        allocation.id,
                        allocation.payment_id,
                        allocation.invoice_id,
                        allocation.amount_cents,
                    ],
                )?;
            }

            Ok(())
        })
    }

    fn find_by_id(&self, id: &str) -> AppResult<Option<Payment>> {
        let mut stmt = self
            .conn
            .prepare(&format!("SELECT {} FROM payments WHERE id = ?", PAYMENT_COLUMNS))?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_payment(row)?)),
            None => Ok(None),
        }
    }

    fn list(&self) -> AppResult<Vec<Payment>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM payments ORDER BY payment_date DESC, created_at DESC",
            PAYMENT_COLUMNS
        ))?;
        let rows = stmt.query_map([], map_payment)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn list_by_student(&self, student_id: &str) -> AppResult<Vec<Payment>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM payments WHERE student_id = ? ORDER BY payment_date DESC, created_at DESC",
            PAYMENT_COLUMNS
        ))?;
        let rows = stmt.query_map(params![student_id], map_payment)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn list_allocations(&self, payment_id: &str) -> AppResult<Vec<PaymentAllocation>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, payment_id, invoice_id, amount_cents FROM payment_allocations \
             WHERE payment_id = ? ORDER BY id",
        )?;
        let rows = stmt.query_map(params![payment_id], map_allocation)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn add_allocation(&self, allocation: &PaymentAllocation) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO payment_allocations (id, payment_id, invoice_id, amount_cents) \
             VALUES (?,?,?,?)",
            params![
                allocation.id,
                allocation.payment_id,
                allocation.invoice_id,
                allocation.amount_cents,
            ],
        )?;
        Ok(())
    }

    fn credit_sources(&self, student_id: &str) -> AppResult<Vec<CreditSource>> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.amount_cents - COALESCE(SUM(a.amount_cents), 0) AS remaining \
             FROM payments p \
             LEFT JOIN payment_allocations a ON a.payment_id = p.id \
             WHERE p.student_id = ? \
             GROUP BY p.id \
             HAVING remaining > 0 \
             ORDER BY p.payment_date ASC, p.created_at ASC, p.id ASC",
        )?;
        let rows = stmt.query_map(params![student_id], |row| {
            Ok(CreditSource {
                payment_id: row.get(0)?,
                remaining_cents: row.get(1)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
}
