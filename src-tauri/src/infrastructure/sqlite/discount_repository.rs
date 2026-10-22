use crate::domain::entities::{Discount, StudentDiscount};
use crate::domain::errors::AppResult;
use crate::domain::ports::DiscountRepository;
use rusqlite::{params, Connection};

pub struct SqliteDiscountRepository<'c> {
    conn: &'c Connection,
}

impl<'c> SqliteDiscountRepository<'c> {
    pub fn new(conn: &'c Connection) -> Self {
        Self { conn }
    }
}

fn map_discount(row: &rusqlite::Row<'_>) -> rusqlite::Result<Discount> {
    Ok(Discount {
        id: row.get(0)?,
        name: row.get(1)?,
        kind: row.get(2)?,
        value: row.get(3)?,
        is_active: row.get::<_, i32>(4)? != 0,
    })
}

fn map_student_discount(row: &rusqlite::Row<'_>) -> rusqlite::Result<StudentDiscount> {
    Ok(StudentDiscount {
        id: row.get(0)?,
        student_id: row.get(1)?,
        discount_id: row.get(2)?,
        term_id: row.get(3)?,
        target_fee_item_id: row.get(4)?,
        target_student_fee_id: row.get(5)?,
        reason: row.get(6)?,
    })
}

impl DiscountRepository for SqliteDiscountRepository<'_> {
    fn list_active(&self) -> AppResult<Vec<Discount>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, kind, value, is_active FROM discounts WHERE is_active = 1 ORDER BY name",
        )?;
        let rows = stmt.query_map([], map_discount)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn list_for_student(&self, student_id: &str, term_id: &str) -> AppResult<Vec<StudentDiscount>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, student_id, discount_id, term_id, target_fee_item_id, \
             target_student_fee_id, reason FROM student_discounts \
             WHERE student_id = ? AND (term_id IS NULL OR term_id = ?) ORDER BY created_at",
        )?;
        let rows = stmt.query_map(params![student_id, term_id], map_student_discount)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn create(&self, discount: &Discount) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO discounts (id, name, kind, value, is_active) VALUES (?,?,?,?,?)",
            params![
                discount.id,
                discount.name,
                discount.kind,
                discount.value,
                i32::from(discount.is_active),
            ],
        )?;
        Ok(())
    }

    fn assign(&self, student_discount: &StudentDiscount) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO student_discounts (id, student_id, discount_id, term_id, \
             target_fee_item_id, target_student_fee_id, reason) VALUES (?,?,?,?,?,?,?)",
            params![
                student_discount.id,
                student_discount.student_id,
                student_discount.discount_id,
                student_discount.term_id,
                student_discount.target_fee_item_id,
                student_discount.target_student_fee_id,
                student_discount.reason,
            ],
        )?;
        Ok(())
    }
}
