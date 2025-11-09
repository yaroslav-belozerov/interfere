use crate::AppTheme;
use chrono::NaiveDateTime;
use reqwest::StatusCode;
use std::fmt::Display;

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
    GotResponse(String, StatusCode, bool),
    DiscardDraftResponse,
    GotError(MyErr),
    Duplicate(String),
    AddQueryParam(),
    SetQueryParamKey(u64, String),
    SetQueryParamContent(u64, String),
    DeleteQueryParam(u64),
    ToggleQueryParamIsOn(u64),
    SetDraftQuery(bool),
    SetSearch(String),
    FormatResponse,
    Start,
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
    pub draft_request: Option<Request>,
    pub draft_response: Option<(StatusCode, String, NaiveDateTime)>,
    pub endp_search: String,
    pub selected_response_index: usize,
    pub formatted_response: Option<String>,
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
