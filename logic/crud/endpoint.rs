use crate::EndpointDb as Endpoint;
use crate::logic::crud::header::create_header_with_tx;
use crate::logic::crud::query::create_query_param_with_tx;
use crate::logic::crud::response::create_response_with_tx;
use rusqlite::{Connection, Result as RusqliteResult};

// ============================================================================
// ENDPOINT OPERATIONS
// ============================================================================

pub fn create_endpoint(conn: &Connection, url: &str) -> RusqliteResult<u64> {
    conn.execute("INSERT INTO endpoint (url) VALUES (?)", [url])?;
    Ok(conn.last_insert_rowid() as u64)
}

pub fn delete_endpoint(conn: &Connection, id: u64) -> RusqliteResult<()> {
    // Delete all related records first (if you don't have CASCADE setup)
    conn.execute("DELETE FROM response WHERE parent_endpoint_id = ?", [id])?;
    // conn.execute("DELETE FROM query_param WHERE parent_endpoint_id = ?", [id])?;
    // conn.execute("DELETE FROM header WHERE parent_endpoint_id = ?", [id])?;

    // Delete the endpoint itself
    conn.execute("DELETE FROM endpoint WHERE id = ?", [id])?;
    Ok(())
}

// Create endpoint with all related data in a transaction
pub fn create_endpoint_full(conn: &Connection, endpoint: &Endpoint) -> RusqliteResult<u64> {
    let tx = conn.unchecked_transaction()?;

    // Insert endpoint
    tx.execute(
        "INSERT INTO endpoint (url, method) VALUES (?, ?)",
        [&endpoint.url, &endpoint.method.to_string()],
    )?;
    let endpoint_id = tx.last_insert_rowid() as u64;

    // Insert responses
    for response in &endpoint.responses {
        create_response_with_tx(&tx, endpoint_id, &response.text, response.code)?;

        // Insert query params
        for qp in &response.request.query_params {
            create_query_param_with_tx(&tx, response.id, &qp.key, &qp.value, qp.on)?;
        }

        // Insert headers
        for header in &response.request.headers {
            create_header_with_tx(&tx, response.id, &header.key, &header.value, header.on)?;
        }
    }

    tx.commit()?;
    Ok(endpoint_id)
}

pub fn update_endpoint_url(conn: &Connection, id: u64, url: &str) -> RusqliteResult<()> {
    conn.execute(
        "UPDATE endpoint SET url = ? WHERE id = ?",
        rusqlite::params![url, id],
    )?;
    Ok(())
}
