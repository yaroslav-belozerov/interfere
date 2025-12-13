use crate::AppTheme;
use chrono::NaiveDateTime;
use core::fmt;
use reqwest::StatusCode;
use rusqlite::{
    ToSql,
    types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef},
};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone)]
pub enum Message {
    SetDraft(String),
    Send,
    SendDraft,
    Back,
    RefetchDb,
    SetSelectedResponseIndex(usize),
    DecrementSelectedResponseIndex,
    IncrementSelectedResponseIndex,
    DecrementSelectedEndpoint,
    IncrementSelectedEndpoint,
    Focus(&'static str),
    ClickEndpoint(u64),
    ClickDeleteEndpoint(u64),
    ClickDeleteResponse(u64),
    ClearErrorMessage,
    SetCtrlPressed(bool),
    GotResponse(String, StatusCode, bool),
    DiscardDraftResponse,
    GotError(MyErr),
    Duplicate(String),
    SetDraftQuery(bool),
    SetSearch(String),
    FormatResponse,
    Start,
    ClickMethod,
    QueryParam(MQueryParam),
    Header(MHeader),
}

#[derive(Debug, Clone)]
pub enum MQueryParam {
    AddQueryParam(),
    SetQueryParamKey(u64, String),
    SetQueryParamContent(u64, String),
    DeleteQueryParam(u64),
    ToggleQueryParamIsOn(u64),
}

#[derive(Debug, Clone)]
pub enum MHeader {
    AddHeader(),
    SetHeaderKey(u64, String),
    SetHeaderContent(u64, String),
    DeleteHeader(u64),
    ToggleHeaderIsOn(u64),
}

#[derive(Debug, Clone)]
pub enum MyErr {
    Unknown(String),
    Client(String),
}

#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    GET,
    POST,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for HttpMethod {
    type Err = ();

    fn from_str(input: &str) -> Result<HttpMethod, Self::Err> {
        match input {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EndpointDb {
    pub id: u64,
    pub url: String,
    pub responses: Vec<Response>,
    pub method: HttpMethod,
}

#[derive(Default, Debug, Clone)]
pub struct Response {
    pub id: u64,
    pub parent_endpoint_id: u64,
    pub request: Request,
    pub text: String,
    pub code: StatusCode,
    pub received_time: NaiveDateTime,
}

#[derive(Default, Debug, Clone)]
pub struct Request {
    pub query_params: Vec<EndpointKvPair>,
    pub headers: Vec<EndpointKvPair>,
}

#[derive(Debug, Clone)]
pub struct EndpointKvPair {
    pub id: u64,
    pub parent_response_id: u64,
    pub key: String,
    pub value: String,
    pub on: bool,
}

pub struct State {
    pub can_send: bool,
    pub endpoints: Vec<EndpointDb>,
    pub selected_endpoint: Option<u64>,
    pub draft: String,
    pub copy_request: Option<Request>,
    pub draft_request: Request,
    pub draft_response: Option<(StatusCode, String, NaiveDateTime)>,
    pub draft_method: HttpMethod,
    pub endp_search: String,
    pub selected_response_index: usize,
    pub formatted_response: Option<String>,
    pub error_message: Option<String>,
    pub ctrl_pressed: bool,
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
