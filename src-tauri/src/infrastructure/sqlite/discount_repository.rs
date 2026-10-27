use crate::domain::entities::{Discount, StudentDiscount};
use crate::domain::errors::{AppError, AppResult};
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
        revoked_at: row.get(7)?,
        revoked_by: row.get(8)?,
        revoke_reason: row.get(9)?,
    })
}

const GRANT_COLUMNS: &str = "id, student_id, discount_id, term_id, target_fee_item_id, \
     target_student_fee_id, reason, revoked_at, revoked_by, revoke_reason";

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
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM student_discounts \
             WHERE student_id = ? AND (term_id IS NULL OR term_id = ?) ORDER BY created_at",
            GRANT_COLUMNS
        ))?;
        let rows = stmt.query_map(params![student_id, term_id], map_student_discount)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn list_all_grants(&self) -> AppResult<Vec<StudentDiscount>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM student_discounts ORDER BY created_at DESC, id DESC",
            GRANT_COLUMNS
        ))?;
        let rows = stmt.query_map([], map_student_discount)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn find_grant_by_id(&self, grant_id: &str) -> AppResult<Option<StudentDiscount>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM student_discounts WHERE id = ?",
            GRANT_COLUMNS
        ))?;
        let mut rows = stmt.query(params![grant_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_student_discount(row)?)),
            None => Ok(None),
        }
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

    fn revoke_grant(
        &self,
        grant_id: &str,
        actor_id: &str,
        reason: &str,
        at: &str,
    ) -> AppResult<StudentDiscount> {
        let grant = self
            .find_grant_by_id(grant_id)?
            .ok_or_else(|| AppError::not_found("Student discount", grant_id))?;
        if grant.revoked_at.is_some() {
            return Err(AppError::conflict("Discount has already been revoked"));
        }
        self.conn.execute(
            "UPDATE student_discounts SET revoked_at = ?, revoked_by = ?, revoke_reason = ? \
             WHERE id = ?",
            params![at, actor_id, reason, grant_id],
        )?;
        self.find_grant_by_id(grant_id)?
            .ok_or_else(|| AppError::not_found("Student discount", grant_id))
    }
}
