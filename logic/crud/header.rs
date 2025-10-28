use rusqlite::{Connection, Result as RusqliteResult};

pub fn create_header(
    conn: &Connection,
    parent_endpoint_id: u64,
    key: &str,
    value: &str,
    on: bool,
) -> RusqliteResult<u64> {
    conn.execute(
        "INSERT INTO header (parent_endpoint_id, key, value, on) VALUES (?, ?, ?, ?)",
        rusqlite::params![parent_endpoint_id, key, value, on],
    )?;
    Ok(conn.last_insert_rowid() as u64)
}

pub fn create_header_with_tx(
    tx: &rusqlite::Transaction,
    parent_endpoint_id: u64,
    key: &str,
    value: &str,
    on: bool,
) -> RusqliteResult<u64> {
    tx.execute(
        "INSERT INTO header (parent_endpoint_id, key, value, on) VALUES (?, ?, ?, ?)",
        rusqlite::params![parent_endpoint_id, key, value, on],
    )?;
    Ok(tx.last_insert_rowid() as u64)
}

pub fn delete_header(conn: &Connection, id: u64) -> RusqliteResult<()> {
    conn.execute("DELETE FROM header WHERE id = ?", [id])?;
    Ok(())
}

pub fn delete_headers_by_endpoint(
    conn: &Connection,
    parent_endpoint_id: u64,
) -> RusqliteResult<()> {
    conn.execute(
        "DELETE FROM header WHERE parent_endpoint_id = ?",
        [parent_endpoint_id],
    )?;
    Ok(())
}

pub fn update_header_on(conn: &Connection, id: u64, on: bool) -> RusqliteResult<()> {
    conn.execute(
        "UPDATE header SET on = ? WHERE id = ?",
        rusqlite::params![on, id],
    )?;
    Ok(())
}

pub fn update_header(
    conn: &Connection,
    id: u64,
    key: &str,
    value: &str,
    on: bool,
) -> RusqliteResult<()> {
    conn.execute(
        "UPDATE header SET key = ?, value = ?, on = ? WHERE id = ?",
        rusqlite::params![key, value, on, id],
    )?;
    Ok(())
}
