use crate::domain::entities::StudentFee;
use crate::domain::errors::AppResult;
use crate::domain::ports::StudentFeeRepository;
use rusqlite::{params, Connection};

pub struct SqliteStudentFeeRepository<'c> {
    conn: &'c Connection,
}

impl<'c> SqliteStudentFeeRepository<'c> {
    pub fn new(conn: &'c Connection) -> Self {
        Self { conn }
    }
}

const STUDENT_FEE_COLUMNS: &str = "id, student_id, term_id, description, amount_cents, \
     discountable, status, invoice_id";

fn map_student_fee(row: &rusqlite::Row<'_>) -> rusqlite::Result<StudentFee> {
    Ok(StudentFee {
        id: row.get(0)?,
        student_id: row.get(1)?,
        term_id: row.get(2)?,
        description: row.get(3)?,
        amount_cents: row.get(4)?,
        discountable: row.get::<_, i32>(5)? != 0,
        status: row.get(6)?,
        invoice_id: row.get(7)?,
    })
}

impl StudentFeeRepository for SqliteStudentFeeRepository<'_> {
    fn create(&self, fee: &StudentFee) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO student_fees (id, student_id, term_id, description, amount_cents, \
             discountable, status) VALUES (?,?,?,?,?,?,?)",
            params![
                fee.id,
                fee.student_id,
                fee.term_id,
                fee.description,
                fee.amount_cents,
                i32::from(fee.discountable),
                fee.status,
            ],
        )?;
        Ok(())
    }

    fn list_pending(&self, student_id: &str, term_id: &str) -> AppResult<Vec<StudentFee>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM student_fees WHERE student_id = ? AND term_id = ? AND status = 'PENDING' \
             ORDER BY created_at",
            STUDENT_FEE_COLUMNS
        ))?;
        let rows = stmt.query_map(params![student_id, term_id], map_student_fee)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn mark_billed(&self, fee_id: &str, invoice_id: &str) -> AppResult<()> {
        self.conn.execute(
            "UPDATE student_fees SET status = 'BILLED', invoice_id = ? WHERE id = ?",
            params![invoice_id, fee_id],
        )?;
        Ok(())
    }
}
