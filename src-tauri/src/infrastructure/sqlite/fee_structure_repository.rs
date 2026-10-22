use crate::domain::entities::{FeeItem, FeeStructure};
use crate::domain::errors::AppResult;
use crate::domain::ports::FeeStructureRepository;
use crate::infrastructure::db::with_optional_transaction;
use rusqlite::{params, Connection};

pub struct SqliteFeeStructureRepository<'c> {
    conn: &'c Connection,
}

impl<'c> SqliteFeeStructureRepository<'c> {
    pub fn new(conn: &'c Connection) -> Self {
        Self { conn }
    }
}

fn map_fee_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<FeeItem> {
    Ok(FeeItem {
        id: row.get(0)?,
        fee_structure_id: row.get(1)?,
        description: row.get(2)?,
        amount_cents: row.get(3)?,
        discountable: row.get::<_, i32>(4)? != 0,
        sort_order: row.get(5)?,
    })
}

impl FeeStructureRepository for SqliteFeeStructureRepository<'_> {
    fn find_by_class_term(&self, class_id: &str, term_id: &str) -> AppResult<Option<FeeStructure>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, class_id, term_id, total_cents FROM fee_structures \
             WHERE class_id = ? AND term_id = ?",
        )?;
        let mut rows = stmt.query(params![class_id, term_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(FeeStructure {
                id: row.get(0)?,
                class_id: row.get(1)?,
                term_id: row.get(2)?,
                total_cents: row.get(3)?,
            })),
            None => Ok(None),
        }
    }

    fn list_items(&self, fee_structure_id: &str) -> AppResult<Vec<FeeItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, fee_structure_id, description, amount_cents, discountable, sort_order \
             FROM fee_items WHERE fee_structure_id = ? ORDER BY sort_order, description",
        )?;
        let rows = stmt.query_map(params![fee_structure_id], map_fee_item)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn save(&self, structure: &FeeStructure, items: &[FeeItem]) -> AppResult<()> {
        with_optional_transaction(self.conn, || {
            self.conn.execute(
                "INSERT INTO fee_structures (id, class_id, term_id, total_cents) VALUES (?,?,?,?) \
                 ON CONFLICT (class_id, term_id) DO UPDATE SET total_cents = excluded.total_cents",
                params![
                    structure.id,
                    structure.class_id,
                    structure.term_id,
                    structure.total_cents,
                ],
            )?;

            let existing_id: Option<String> = self
                .conn
                .query_row(
                    "SELECT id FROM fee_structures WHERE class_id = ? AND term_id = ?",
                    params![structure.class_id, structure.term_id],
                    |row| row.get(0),
                )
                .ok();

            let effective_id = existing_id.unwrap_or_else(|| structure.id.clone());

            self.conn.execute(
                "DELETE FROM fee_items WHERE fee_structure_id = ?",
                params![effective_id],
            )?;

            for item in items {
                self.conn.execute(
                    "INSERT INTO fee_items (id, fee_structure_id, description, amount_cents, \
                     discountable, sort_order) VALUES (?,?,?,?,?,?)",
                    params![
                        item.id,
                        effective_id,
                        item.description,
                        item.amount_cents,
                        i32::from(item.discountable),
                        item.sort_order,
                    ],
                )?;
            }

            Ok(())
        })
    }
}
