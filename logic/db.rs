use std::sync::{Mutex, OnceLock};

use reqwest::StatusCode;
use rusqlite::{Connection, Result};

use crate::{EndpointDb, EndpointKvPair, Request, Response};

static DB: OnceLock<Mutex<Connection>> = OnceLock::new();

pub fn get_db() -> &'static Mutex<Connection> {
    DB.get_or_init(|| {
        let conn = Connection::open("interfere.db").expect("Failed to open database");
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        Mutex::new(conn)
    })
}

pub fn init() -> Result<()> {
    let mut db = get_db().lock().unwrap();
    let tx = db.transaction().unwrap();

    tx.execute(
        "create table if not exists endpoint (
             id integer primary key,
             url varchar(512)
         )",
        (),
    )?;
    tx.execute(
        "create table if not exists response (
             id integer primary key,
             parent_endpoint_id integer not null,
             text varchar(512),
             code integer,
             received_time DATETIME DEFAULT CURRENT_TIMESTAMP,
             FOREIGN KEY (parent_endpoint_id) references endpoint(id) ON DELETE CASCADE
         )",
        (),
    )?;
    tx.execute(
        "create table if not exists query_param (
             id integer primary key,
             parent_response_id integer not null,
             key varchar(512),
             value varchar(512),
             is_on boolean,
             FOREIGN KEY (parent_response_id) references response(id) ON DELETE CASCADE
         )",
        (),
    )?;
    tx.execute(
        "create table if not exists header (
             id integer primary key,
             parent_response_id integer not null,
             key varchar(512),
             value varchar(512),
             is_on boolean,
             FOREIGN KEY (parent_response_id) references response(id) ON DELETE CASCADE
         )",
        (),
    )?;

    tx.commit()
}

pub fn load_endpoints(
    conn: &rusqlite::Connection,
    query: Option<&str>,
) -> Result<Vec<EndpointDb>, rusqlite::Error> {
    // First, get all endpoints
    let mut stmt =
        conn.prepare("SELECT id, url FROM endpoint WHERE url LIKE (?) ORDER BY id DESC")?;
    let endpoint_rows = stmt.query_map(
        [match query {
            Some(it) => format!("%{}%", it),
            None => "%".to_string(),
        }],
        |row| Ok((row.get::<_, u64>(0)?, row.get::<_, String>(1)?)),
    )?;

    let mut endpoints = Vec::new();

    for endpoint_result in endpoint_rows {
        let (id, url) = endpoint_result?;

        // Load responses for this endpoint
        let mut resp_stmt = conn.prepare(
            "SELECT id, parent_endpoint_id, text, code, received_time 
             FROM response 
             WHERE parent_endpoint_id = ?",
        )?;
        let responses: Vec<Response> = resp_stmt
            .query_map([id], |row| {
                let resp_id = row.get(0)?;

                // Load query params for this response
                let mut qp_stmt = conn.prepare(
                    "SELECT id, parent_response_id, key, value, is_on 
             FROM query_param 
             WHERE parent_response_id = ?",
                )?;
                let query_params: Vec<EndpointKvPair> = qp_stmt
                    .query_map([resp_id], |row| {
                        Ok(EndpointKvPair {
                            id: row.get(0)?,
                            parent_response_id: row.get(1)?,
                            key: row.get(2)?,
                            value: row.get(3)?,
                            on: row.get(4)?,
                        })
                    })?
                    .collect::<Result<_, _>>()?;

                // Load headers for this response
                let mut hdr_stmt = conn.prepare(
                    "SELECT id, parent_response_id, key, value, is_on 
             FROM header 
             WHERE parent_response_id = ?",
                )?;
                let headers: Vec<EndpointKvPair> = hdr_stmt
                    .query_map([resp_id], |row| {
                        Ok(EndpointKvPair {
                            id: row.get(0)?,
                            parent_response_id: row.get(1)?,
                            key: row.get(2)?,
                            value: row.get(3)?,
                            on: row.get(4)?,
                        })
                    })?
                    .collect::<Result<_, _>>()?;

                Ok(Response {
                    id: resp_id,
                    parent_endpoint_id: row.get(1)?,
                    text: row.get(2)?,
                    code: StatusCode::from_u16(row.get(3).unwrap()).unwrap(),
                    received_time: row.get(4)?,
                    request: Request {
                        query_params,
                        headers,
                    },
                })
            })?
            .collect::<Result<_, _>>()?;

        endpoints.push(EndpointDb { id, url, responses });
    }

    Ok(endpoints)
}
