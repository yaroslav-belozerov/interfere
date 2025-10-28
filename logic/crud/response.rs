use reqwest::StatusCode;
use rusqlite::{Connection, Result as RusqliteResult};

use crate::Response;
pub fn create_response(
    conn: &Connection,
    parent_endpoint_id: u64,
    text: &str,
    code: StatusCode,
) -> RusqliteResult<u64> {
    conn.execute(
        "INSERT INTO response (parent_endpoint_id, text, code) VALUES (?, ?, ?)",
        rusqlite::params![parent_endpoint_id, text, code.as_u16() as i64],
    )?;
    Ok(conn.last_insert_rowid() as u64)
}

pub fn responses_by_endpoint_id(
    conn: &rusqlite::Connection,
    id: u64,
) -> Result<Vec<Response>, rusqlite::Error> {
    let mut resp_stmt = conn.prepare(
        "SELECT id, parent_endpoint_id, text, code, received_time 
             FROM response 
             WHERE parent_endpoint_id = ?",
    )?;
    let responses: Vec<Response> = resp_stmt
        .query_map([id], |row| {
            Ok(Response {
                id: row.get(0)?,
                parent_endpoint_id: row.get(1)?,
                text: row.get(2)?,
                code: StatusCode::from_u16(row.get(3).unwrap()).unwrap(),
                received_time: row.get(4)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(responses)
}

// Helper for transactions
pub fn create_response_with_tx(
    tx: &rusqlite::Transaction,
    parent_endpoint_id: u64,
    text: &str,
    code: StatusCode,
) -> RusqliteResult<u64> {
    tx.execute(
        "INSERT INTO response (parent_endpoint_id, text, code) VALUES (?, ?, ?)",
        rusqlite::params![parent_endpoint_id, text, code.as_u16() as i64],
    )?;
    Ok(tx.last_insert_rowid() as u64)
}

pub fn delete_response(conn: &Connection, id: u64) -> RusqliteResult<()> {
    conn.execute("DELETE FROM response WHERE id = ?", [id])?;
    Ok(())
}

pub fn delete_responses_by_endpoint(
    conn: &Connection,
    parent_endpoint_id: u64,
) -> RusqliteResult<()> {
    conn.execute(
        "DELETE FROM response WHERE parent_endpoint_id = ?",
        [parent_endpoint_id],
    )?;
    Ok(())
}

pub fn update_response(
    conn: &Connection,
    id: u64,
    text: &str,
    code: StatusCode,
) -> RusqliteResult<()> {
    conn.execute(
        "UPDATE response SET text = ?, code = ? WHERE id = ?",
        rusqlite::params![text, code.as_u16() as i64, id],
    )?;
    Ok(())
}
