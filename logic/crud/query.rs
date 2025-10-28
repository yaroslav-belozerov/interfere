use rusqlite::{Connection, Result as RusqliteResult};
pub fn create_query_param(
    conn: &Connection,
    parent_endpoint_id: u64,
    key: &str,
    value: &str,
    on: bool,
) -> RusqliteResult<u64> {
    conn.execute(
        "INSERT INTO query_param (parent_endpoint_id, key, value, on) VALUES (?, ?, ?, ?)",
        rusqlite::params![parent_endpoint_id, key, value, on],
    )?;
    Ok(conn.last_insert_rowid() as u64)
}

pub fn create_query_param_with_tx(
    tx: &rusqlite::Transaction,
    parent_endpoint_id: u64,
    key: &str,
    value: &str,
    on: bool,
) -> RusqliteResult<u64> {
    tx.execute(
        "INSERT INTO query_param (parent_endpoint_id, key, value, on) VALUES (?, ?, ?, ?)",
        rusqlite::params![parent_endpoint_id, key, value, on],
    )?;
    Ok(tx.last_insert_rowid() as u64)
}

pub fn delete_query_param(conn: &Connection, id: u64) -> RusqliteResult<()> {
    conn.execute("DELETE FROM query_param WHERE id = ?", [id])?;
    Ok(())
}

pub fn delete_query_params_by_endpoint(
    conn: &Connection,
    parent_endpoint_id: u64,
) -> RusqliteResult<()> {
    conn.execute(
        "DELETE FROM query_param WHERE parent_endpoint_id = ?",
        [parent_endpoint_id],
    )?;
    Ok(())
}

pub fn update_query_param_on(conn: &Connection, id: u64, on: bool) -> RusqliteResult<()> {
    conn.execute(
        "UPDATE query_param SET on = ? WHERE id = ?",
        rusqlite::params![on, id],
    )?;
    Ok(())
}

pub fn update_query_param(
    conn: &Connection,
    id: u64,
    key: &str,
    value: &str,
    on: bool,
) -> RusqliteResult<()> {
    conn.execute(
        "UPDATE query_param SET key = ?, value = ?, on = ? WHERE id = ?",
        rusqlite::params![key, value, on, id],
    )?;
    Ok(())
}
