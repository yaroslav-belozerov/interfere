use rusqlite::{Connection, Result as RusqliteResult};

pub fn create_header(
    conn: &Connection,
    parent_response_id: u64,
    key: &str,
    value: &str,
    on: bool,
) -> RusqliteResult<u64> {
    conn.execute(
        "INSERT INTO header (parent_response_id, key, value, is_on) VALUES (?, ?, ?, ?)",
        rusqlite::params![parent_response_id, key, value, on],
    )?;
    Ok(conn.last_insert_rowid() as u64)
}

pub fn create_header_with_tx(
    tx: &rusqlite::Transaction,
    parent_response_id: u64,
    key: &str,
    value: &str,
    on: bool,
) -> RusqliteResult<u64> {
    tx.execute(
        "INSERT INTO header (parent_response_id, key, value, is_on) VALUES (?, ?, ?, ?)",
        rusqlite::params![parent_response_id, key, value, on],
    )?;
    Ok(tx.last_insert_rowid() as u64)
}

pub fn delete_header(conn: &Connection, id: u64) -> RusqliteResult<()> {
    conn.execute("DELETE FROM header WHERE id = ?", [id])?;
    Ok(())
}

pub fn delete_headers_by_response(
    conn: &Connection,
    parent_response_id: u64,
) -> RusqliteResult<()> {
    conn.execute(
        "DELETE FROM header WHERE parent_response_id = ?",
        [parent_response_id],
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
        "UPDATE header SET key = ?, value = ?, is_on = ? WHERE id = ?",
        rusqlite::params![key, value, on, id],
    )?;
    Ok(())
}

pub fn update_header_on(conn: &Connection, id: u64) -> RusqliteResult<()> {
    conn.execute(
        "UPDATE header SET is_on = NOT is_on WHERE id = ?",
        rusqlite::params![id],
    )?;
    Ok(())
}

pub fn update_header_key(conn: &Connection, id: u64, key: &str) -> RusqliteResult<()> {
    conn.execute(
        "UPDATE header SET key = ? WHERE id = ?",
        rusqlite::params![key, id],
    )?;
    Ok(())
}

pub fn update_header_value(conn: &Connection, id: u64, value: &str) -> RusqliteResult<()> {
    conn.execute(
        "UPDATE header SET value = ? WHERE id = ?",
        rusqlite::params![value, id],
    )?;
    Ok(())
}
