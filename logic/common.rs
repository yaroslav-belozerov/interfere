use crate::AppTheme;
use chrono::{NaiveDate, NaiveDateTime};
use iced::theme::Palette;
use reqwest::StatusCode;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Message {
    SetDraft(String),
    Send,
    Back,
    RefetchDb,
    SetSelectedResponseIndex(usize),
    ClickEndpoint(u64),
    ClickDeleteEndpoint(u64),
    ClickDeleteResponse(u64),
    GotResponse(String, StatusCode),
    GotError(MyErr),
    Duplicate(String),
    Start,
    Empty,
}

#[derive(Debug, Clone)]
pub enum MyErr {
    Unknown(String),
    Client(String),
}

#[derive(Debug, Clone)]
pub struct EndpointDb {
    pub id: u64,
    pub url: String,
    pub responses: Vec<Response>,
    pub query_params: Vec<EndpointKvPair>,
    pub headers: Vec<EndpointKvPair>,
}

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub id: u64,
    pub url: String,
}

#[derive(Default, Debug, Clone)]
pub struct Response {
    pub id: u64,
    pub parent_endpoint_id: u64,
    pub text: String,
    pub code: StatusCode,
    pub received_time: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct EndpointKvPair {
    pub id: u64,
    pub parent_endpoint_id: u64,
    pub key: String,
    pub value: String,
    pub on: bool,
}

pub struct State {
    pub can_send: bool,
    pub endpoints: Vec<EndpointDb>,
    pub selected_endpoint: Option<u64>,
    pub draft: String,
    pub selected_response_index: usize,
    pub theme: AppTheme,
}

impl Display for MyErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown(s) => write!(f, "Unknown error: {s}"),
            Self::Client(s) => write!(f, "Interfere error: {s}"),
        }
    }
}
