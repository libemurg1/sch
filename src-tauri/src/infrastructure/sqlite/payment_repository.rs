use crate::domain::entities::{Payment, PaymentAllocation};
use crate::domain::errors::{AppError, AppResult};
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
     payment_method, reference_number, created_by, created_at, reversed_at, reversed_by, \
     reversal_reason";

const ALLOCATION_COLUMNS: &str =
    "id, payment_id, invoice_id, amount_cents, reversed_at, reversal_reason";

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
        reversed_at: row.get(9)?,
        reversed_by: row.get(10)?,
        reversal_reason: row.get(11)?,
    })
}

fn map_allocation(row: &rusqlite::Row<'_>) -> rusqlite::Result<PaymentAllocation> {
    Ok(PaymentAllocation {
        id: row.get(0)?,
        payment_id: row.get(1)?,
        invoice_id: row.get(2)?,
        amount_cents: row.get(3)?,
        reversed_at: row.get(4)?,
        reversal_reason: row.get(5)?,
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
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM payments WHERE id = ?",
            PAYMENT_COLUMNS
        ))?;
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
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM payment_allocations WHERE payment_id = ? ORDER BY id",
            ALLOCATION_COLUMNS
        ))?;
        let rows = stmt.query_map(params![payment_id], map_allocation)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn list_allocations_for_student(&self, student_id: &str) -> AppResult<Vec<PaymentAllocation>> {
        let mut stmt = self.conn.prepare(
            "SELECT a.id, a.payment_id, a.invoice_id, a.amount_cents, a.reversed_at, \
             a.reversal_reason \
             FROM payment_allocations a \
             JOIN payments p ON p.id = a.payment_id \
             WHERE p.student_id = ? ORDER BY a.id",
        )?;
        let rows = stmt.query_map(params![student_id], map_allocation)?;
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

    fn reverse_allocation(&self, allocation_id: &str, reason: &str, at: &str) -> AppResult<()> {
        let changed = self.conn.execute(
            "UPDATE payment_allocations SET reversed_at = ?, reversal_reason = ? \
             WHERE id = ? AND reversed_at IS NULL",
            params![at, reason, allocation_id],
        )?;
        if changed == 0 {
            return Err(AppError::conflict("Payment allocation is already reversed"));
        }
        Ok(())
    }

    fn mark_reversed(
        &self,
        payment_id: &str,
        actor_id: &str,
        reason: &str,
        at: &str,
    ) -> AppResult<Payment> {
        with_optional_transaction(self.conn, || {
            let payment = self
                .find_by_id(payment_id)?
                .ok_or_else(|| AppError::not_found("Payment", payment_id))?;
            if payment.reversed_at.is_some() {
                return Err(AppError::conflict(format!(
                    "Receipt {} is already reversed",
                    payment.receipt_number
                )));
            }
            self.conn.execute(
                "UPDATE payments SET reversed_at = ?, reversed_by = ?, reversal_reason = ? \
                 WHERE id = ?",
                params![at, actor_id, reason, payment_id],
            )?;
            self.find_by_id(payment_id)?
                .ok_or_else(|| AppError::not_found("Payment", payment_id))
        })
    }

    fn credit_sources(&self, student_id: &str) -> AppResult<Vec<CreditSource>> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.amount_cents - COALESCE(SUM(\
               CASE WHEN a.reversed_at IS NULL THEN a.amount_cents ELSE 0 END), 0) AS remaining \
             FROM payments p \
             LEFT JOIN payment_allocations a ON a.payment_id = p.id \
             WHERE p.student_id = ? AND p.reversed_at IS NULL \
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
