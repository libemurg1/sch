use crate::domain::dto::AuditEntry;
use crate::domain::errors::AppResult;
use rusqlite::{params, Connection};
use serde_json::Value;

pub struct AuditService;

impl AuditService {
    pub fn record(
        conn: &Connection,
        actor_id: Option<&str>,
        entity: &str,
        entity_id: &str,
        action: &str,
        detail: Value,
        at: &str,
    ) -> AppResult<()> {
        conn.execute(
            "INSERT INTO audit_log (actor_id, entity, entity_id, action, detail_json, at) \
             VALUES (?,?,?,?,?,?)",
            params![actor_id, entity, entity_id, action, detail.to_string(), at,],
        )?;
        Ok(())
    }

    pub fn list(conn: &Connection, limit: i64) -> AppResult<Vec<AuditEntry>> {
        let mut statement = conn.prepare(
            "SELECT a.id, a.actor_id, u.display_name, a.entity, a.entity_id, a.action, \
             a.detail_json, a.at \
             FROM audit_log a LEFT JOIN users u ON u.id = a.actor_id \
             ORDER BY a.id DESC LIMIT ?",
        )?;
        let rows = statement.query_map([limit], |row| {
            Ok(AuditEntry {
                id: row.get(0)?,
                actor_id: row.get(1)?,
                actor_name: row.get(2)?,
                entity: row.get(3)?,
                entity_id: row.get(4)?,
                action: row.get(5)?,
                detail_json: row.get(6)?,
                at: row.get(7)?,
            })
        })?;
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }
}
