use crate::domain::entities::Term;
use crate::domain::errors::AppResult;
use crate::domain::ports::TermRepository;
use rusqlite::{params, Connection};

pub struct SqliteTermRepository<'c> {
    conn: &'c Connection,
}

impl<'c> SqliteTermRepository<'c> {
    pub fn new(conn: &'c Connection) -> Self {
        Self { conn }
    }
}

fn map_term(row: &rusqlite::Row<'_>) -> rusqlite::Result<Term> {
    Ok(Term {
        id: row.get(0)?,
        year: row.get(1)?,
        term_number: row.get(2)?,
        start_date: row.get(3)?,
        end_date: row.get(4)?,
        is_active: row.get::<_, i32>(5)? != 0,
    })
}

const TERM_COLUMNS: &str = "id, year, term_number, start_date, end_date, is_active";

impl TermRepository for SqliteTermRepository<'_> {
    fn list(&self) -> AppResult<Vec<Term>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM terms ORDER BY year DESC, term_number DESC",
            TERM_COLUMNS
        ))?;
        let rows = stmt.query_map([], map_term)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn find_by_id(&self, id: &str) -> AppResult<Option<Term>> {
        let mut stmt = self
            .conn
            .prepare(&format!("SELECT {} FROM terms WHERE id = ?", TERM_COLUMNS))?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_term(row)?)),
            None => Ok(None),
        }
    }

    fn find_by_year_term(&self, year: i32, term_number: i32) -> AppResult<Option<Term>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM terms WHERE year = ? AND term_number = ?",
            TERM_COLUMNS
        ))?;
        let mut rows = stmt.query(params![year, term_number])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_term(row)?)),
            None => Ok(None),
        }
    }

    fn get_active(&self) -> AppResult<Option<Term>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM terms WHERE is_active = 1",
            TERM_COLUMNS
        ))?;
        let mut rows = stmt.query([])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_term(row)?)),
            None => Ok(None),
        }
    }

    fn create(&self, term: &Term) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO terms (id, year, term_number, start_date, end_date, is_active) \
             VALUES (?,?,?,?,?,?)",
            params![
                term.id,
                term.year,
                term.term_number,
                term.start_date,
                term.end_date,
                i32::from(term.is_active),
            ],
        )?;
        Ok(())
    }

    fn update(&self, term: &Term) -> AppResult<()> {
        let changed = self.conn.execute(
            "UPDATE terms SET year = ?, term_number = ?, start_date = ?, \
             end_date = ?, is_active = ? WHERE id = ?",
            params![
                term.year,
                term.term_number,
                term.start_date,
                term.end_date,
                i32::from(term.is_active),
                term.id,
            ],
        )?;
        if changed == 0 {
            return Err(crate::domain::errors::AppError::not_found("Term", &term.id));
        }
        Ok(())
    }

    fn delete(&self, id: &str) -> AppResult<()> {
        let changed = self
            .conn
            .execute("DELETE FROM terms WHERE id = ?", params![id])?;
        if changed == 0 {
            return Err(crate::domain::errors::AppError::not_found("Term", id));
        }
        Ok(())
    }

    fn deactivate_others(&self, id: &str) -> AppResult<()> {
        self.conn.execute(
            "UPDATE terms SET is_active = 0 WHERE is_active = 1 AND id <> ?",
            params![id],
        )?;
        Ok(())
    }

    fn reference_count(&self, id: &str) -> AppResult<i64> {
        Ok(self.conn.query_row(
            "SELECT \
               (SELECT COUNT(*) FROM enrollments WHERE term_id = ?1) \
             + (SELECT COUNT(*) FROM fee_structures WHERE term_id = ?1) \
             + (SELECT COUNT(*) FROM invoices WHERE term_id = ?1) \
             + (SELECT COUNT(*) FROM student_fees WHERE term_id = ?1) \
             + (SELECT COUNT(*) FROM student_discounts WHERE term_id = ?1)",
            params![id],
            |row| row.get(0),
        )?)
    }
}
