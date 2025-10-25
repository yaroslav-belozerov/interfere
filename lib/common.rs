use reqwest::StatusCode;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Message {
    SetUrl(String),
    Send,
    GotResponse(String, StatusCode),
    GotError(MyErr),
    Empty,
}

#[derive(Debug, Clone)]
pub enum MyErr {
    Unknown(String),
}

impl Display for MyErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown(s) => write!(f, "Unknown error: {s}"),
        }
    }
}
