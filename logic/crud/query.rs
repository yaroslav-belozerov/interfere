use rusqlite::{Connection, Result as RusqliteResult};
pub fn create_query_param(
    conn: &Connection,
    parent_response_id: u64,
    key: &str,
    value: &str,
) -> RusqliteResult<u64> {
    conn.execute(
        "INSERT INTO query_param (parent_response_id, key, value) VALUES (?, ?, ?)",
        rusqlite::params![parent_response_id, key, value],
    )?;
    Ok(conn.last_insert_rowid() as u64)
}

pub fn create_query_param_with_tx(
    tx: &rusqlite::Transaction,
    parent_response_id: u64,
    key: &str,
    value: &str,
) -> RusqliteResult<u64> {
    tx.execute(
        "INSERT INTO query_param (parent_response_id, key, value) VALUES (?, ?, ?)",
        rusqlite::params![parent_response_id, key, value],
    )?;
    Ok(tx.last_insert_rowid() as u64)
}

pub fn delete_query_param(conn: &Connection, id: u64) -> RusqliteResult<()> {
    conn.execute("DELETE FROM query_param WHERE id = ?", [id])?;
    Ok(())
}

pub fn delete_query_params_by_response(
    conn: &Connection,
    parent_response_id: u64,
) -> RusqliteResult<()> {
    conn.execute(
        "DELETE FROM query_param WHERE parent_response_id = ?",
        [parent_response_id],
    )?;
    Ok(())
}

pub fn update_query_param_key(conn: &Connection, id: u64, key: &str) -> RusqliteResult<()> {
    conn.execute(
        "UPDATE query_param SET key = ? WHERE id = ?",
        rusqlite::params![key, id],
    )?;
    Ok(())
}

pub fn update_query_param_value(conn: &Connection, id: u64, value: &str) -> RusqliteResult<()> {
    conn.execute(
        "UPDATE query_param SET value = ? WHERE id = ?",
        rusqlite::params![value, id],
    )?;
    Ok(())
}
