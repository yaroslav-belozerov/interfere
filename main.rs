mod logic;

use crate::AppTheme;
use chrono::{Local, NaiveDateTime, TimeZone};
use iced::Alignment::{self, Center};
use iced::Length::{Fill, Shrink};
use iced::font::Weight;
use iced::keyboard::Modifiers;
use iced::theme::Palette;
use iced::widget::scrollable::Scrollbar;
use iced::widget::text_input::focus;
use iced::widget::{
    Button, Column, Container, Row, column, container, horizontal_space, row, scrollable, svg, text,
};
use iced::{Border, Color, Element, Font, Renderer, Subscription, Task, Theme, keyboard};
use logic::common::*;
use logic::crud::endpoint::{create_endpoint_full, delete_endpoint};
use logic::crud::header::create_header_with_tx;
use logic::crud::query::create_query_param_with_tx;
use logic::crud::response::{
    create_response, delete_response, response_count_by_endpoint_id, update_response,
};
use logic::db::{get_db, init, load_endpoints};
use logic::message_handlers::{message_header, message_query_param};
use logic::ui::*;
use markup_fmt::{Language, format_text};
use reqwest::StatusCode;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;
use std::cmp::max;
use std::str::FromStr;

impl State {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                draft: Default::default(),
                copy_request: None,
                draft_request: Request {
                    query_params: vec![],
                    headers: vec![],
                },
                draft_response: None,
                draft_method: HttpMethod::GET,
                endpoints: load_endpoints(&get_db().lock().unwrap(), None).unwrap(),
                endp_search: "".to_string(),
                can_send: true,
                selected_endpoint: None,
                selected_response_index: 0,
                formatted_response: None,
                error_message: None,
                ctrl_pressed: false,
                theme: AppTheme {
                    palette: Palette {
                        background: Color::parse("#1A1B26").unwrap(),
                        danger: Color::parse("#F7768E").unwrap(),
                        primary: Color::parse("#BB9AF7").unwrap(),
                        success: Color::parse("#A6CD70").unwrap(),
                        text: Color::parse("#C0CAF5").unwrap(),
                    },
                },
            },
            Task::perform(async {}, |_| Message::Start),
        )
    }
}

fn create_new_endpoint(state: &mut State, parent_id: u64, text: &str, code: StatusCode) {
    let resp_id = create_response(&get_db().lock().unwrap(), parent_id, &text, code).unwrap();
    match &state.copy_request {
        Some(d) => {
            let mut db = get_db().lock().unwrap();
            let tx = db.transaction().unwrap();
            for q in &d.query_params {
                create_query_param_with_tx(&tx, resp_id, &q.key, &q.value).unwrap();
            }
            for h in &d.headers {
                create_header_with_tx(&tx, resp_id, &h.key, &h.value).unwrap();
            }
            tx.commit().unwrap();
            state.draft_response = None;
            state.copy_request = None;
        }
        None => {}
    }
    state.selected_response_index = max(
        response_count_by_endpoint_id(&get_db().lock().unwrap(), parent_id).unwrap() as usize - 1,
        0,
    );
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::SetCtrlPressed(pressed) => {
            state.ctrl_pressed = pressed;
            Task::none()
        }
        Message::SetDraftQuery(copy) => {
            if copy {
                match current_response(state) {
                    Some(resp) => {
                        state.copy_request = Some(resp.request.clone());
                    }
                    None => {}
                }
            } else {
                state.copy_request = Some(Request {
                    query_params: Vec::new(),
                    headers: Vec::new(),
                });
            }
            Task::none()
        }
        Message::Duplicate(s) => {
            state.draft_response = None;
            Task::batch([
                update(state, Message::Back),
                update(state, Message::SetDraft(s)),
            ])
        }
        Message::Start => focus("main_urlbar"),
        Message::RefetchDb => {
            state.endpoints =
                load_endpoints(&get_db().lock().unwrap(), Some(&state.endp_search)).unwrap();
            Task::none()
        }
        Message::SetSelectedResponseIndex(index) => {
            state.formatted_response = None;
            state.selected_response_index = index;
            state.draft_response = None;
            state.copy_request = None;
            Task::none()
        }
        Message::SetDraft(string) => {
            state.draft = string;
            Task::none()
        }
        Message::Send => {
            state.can_send = false;
            let url = format_url_from_state(state);
            let headers = headers_from_state(state);
            let method = method_from_state(state);
            Task::perform(send_request(url, headers, method), |res| match res {
                Ok((text, code)) => Message::GotResponse(text, code, false),
                Err(err) => Message::GotError(err),
            })
        }
        Message::SendDraft => {
            state.can_send = false;
            let url = format_url_from_state(state);
            let headers = headers_from_state(state);
            let method = method_from_state(state);
            Task::perform(send_request(url, headers, method), |res| match res {
                Ok((text, code)) => Message::GotResponse(text, code, true),
                Err(err) => Message::GotError(err),
            })
        }
        Message::Back => {
            state.error_message = None;
            if state.draft_response.is_none() {
                state.draft_request.headers = vec![];
                state.draft_request.query_params = vec![];
                state.selected_endpoint = None;
                focus("main_urlbar")
            } else {
                update(state, Message::DiscardDraftResponse)
            }
        }
        Message::GotResponse(text, code, is_draft) => {
            state.can_send = true;
            if is_draft {
                match &mut state.draft_response {
                    Some(d) => {
                        d.0 = code;
                        d.1 = text;
                        d.2 = Local::now().naive_local();
                    }
                    None => state.draft_response = Some((code, text, Local::now().naive_local())),
                }
            } else {
                match state.selected_endpoint {
                    Some(id) => match current_response(state) {
                        Some(current_response) => {
                            if state.copy_request.is_none() {
                                update_response(
                                    &get_db().lock().unwrap(),
                                    current_response.id,
                                    &text,
                                    code,
                                    Local::now().naive_utc(),
                                )
                                .unwrap();
                            } else {
                                create_new_endpoint(state, id, &text, code);
                            }
                        }
                        None => {
                            create_new_endpoint(state, id, &text, code);
                        }
                    },
                    None => {
                        let id = create_endpoint_full(
                            &get_db().lock().unwrap(),
                            &EndpointDb {
                                id: 0,
                                url: state.draft.clone(),
                                responses: [Response {
                                    id: 0,
                                    parent_endpoint_id: 0,
                                    text: text.to_string(),
                                    code,
                                    received_time: NaiveDateTime::default(),
                                    request: Request {
                                        query_params: state.draft_request.query_params.to_vec(),
                                        headers: state.draft_request.headers.to_vec(),
                                    },
                                }]
                                .to_vec(),
                                method: HttpMethod::GET,
                            },
                        )
                        .unwrap();
                        return Task::batch([
                            update(state, Message::ClickEndpoint(id)),
                            update(state, Message::RefetchDb),
                        ]);
                    }
                }
            }
            state.draft = "".to_string();
            update(state, Message::RefetchDb)
        }
        Message::GotError(err) => {
            state.can_send = true;
            state.error_message = Some(err.to_string());
            Task::none()
        }
        Message::ClearErrorMessage => {
            state.error_message = None;
            Task::none()
        }
        Message::QueryParam(message) => message_query_param(state, message),
        Message::Header(message) => message_header(state, message),
        Message::ClickEndpoint(id) => {
            state.formatted_response = None;
            state.copy_request = None;
            state.draft_response = None;
            state.selected_endpoint = Some(id);
            state.error_message = None;
            let count =
                response_count_by_endpoint_id(&get_db().lock().unwrap(), id).unwrap() as usize;
            state.selected_response_index = max(count, 1) - 1;
            Task::none()
        }
        Message::ClickMethod => {
            state.draft_method = match state.draft_method {
                HttpMethod::GET => HttpMethod::POST,
                HttpMethod::POST => HttpMethod::GET,
            };
            Task::none()
        }
        Message::ClickDeleteEndpoint(id) => {
            delete_endpoint(&get_db().lock().unwrap(), id).unwrap();
            match state.selected_endpoint {
                Some(selected_id) => {
                    if id == selected_id {
                        state.selected_endpoint = None;
                    }
                }
                None => {}
            }
            update(state, Message::RefetchDb)
        }
        Message::ClickDeleteResponse(id) => match state.draft_response {
            Some(_) => {
                state.draft_response = None;
                Task::none()
            }
            None => {
                match state.selected_endpoint {
                    Some(endpoint_id) => {
                        match state.endpoints.iter().find(|&it| it.id == endpoint_id) {
                            Some(endpoint) => {
                                if state.selected_response_index + 1
                                    == endpoint.responses.iter().len()
                                {
                                    state.selected_response_index =
                                        max(state.selected_response_index, 1) - 1;
                                }
                            }
                            None => return Task::none(),
                        }
                    }
                    None => return Task::none(),
                };

                delete_response(&get_db().lock().unwrap(), id).unwrap();
                update(state, Message::RefetchDb)
            }
        },
        Message::SetSearch(query) => {
            state.endp_search = query;
            update(state, Message::RefetchDb)
        }
        Message::FormatResponse => {
            let current = state
                .endpoints
                .iter()
                .find(|it| Some(it.id) == state.selected_endpoint);
            match current {
                Some(e) => {
                    let current_response = e.responses.get(state.selected_response_index);
                    match current_response {
                        Some(resp) => {
                            state.formatted_response = Some(format_response(&resp.text));
                        }
                        None => match &state.draft_response {
                            Some(draft_resp) => {
                                state.formatted_response = Some(format_response(&draft_resp.1))
                            }
                            None => {}
                        },
                    }
                }
                None => {}
            };
            Task::none()
        }
        Message::Focus(it) => focus(it),
        Message::DecrementSelectedResponseIndex => match state.copy_request {
            Some(_) => {
                state.draft_response = None;
                state.copy_request = None;
                Task::none()
            }
            None => update(
                state,
                Message::SetSelectedResponseIndex(usize::max(state.selected_response_index, 1) - 1),
            ),
        },
        Message::IncrementSelectedResponseIndex => match current_endpoint(state) {
            Some(endpoint) => update(
                state,
                if state.selected_response_index < endpoint.responses.len() - 1 {
                    Message::SetSelectedResponseIndex(usize::min(
                        state.selected_response_index + 1,
                        endpoint.responses.len() - 1,
                    ))
                } else {
                    Message::SetDraftQuery(false)
                },
            ),
            None => Task::none(),
        },
        Message::DiscardDraftResponse => {
            state.draft_response = None;
            state.formatted_response = None;
            Task::none()
        }
        Message::IncrementSelectedEndpoint => update(
            state,
            Message::ClickEndpoint(match current_endpoint(state) {
                Some(endpoint) => {
                    let curr_index = state
                        .endpoints
                        .iter()
                        .position(|it| it.id == endpoint.id)
                        .unwrap();
                    match state
                        .endpoints
                        .get(usize::min(curr_index, state.endpoints.len() - 1) + 1)
                    {
                        Some(n) => n.id,
                        None => {
                            return Task::none();
                        }
                    }
                }
                None => match state.endpoints.get(0) {
                    Some(n) => n.id,
                    None => return Task::none(),
                },
            }),
        ),
        Message::DecrementSelectedEndpoint => update(
            state,
            Message::ClickEndpoint(match current_endpoint(state) {
                Some(endpoint) => {
                    let curr_index = state
                        .endpoints
                        .iter()
                        .position(|it| it.id == endpoint.id)
                        .unwrap();
                    match state.endpoints.get(usize::max(curr_index, 1) - 1) {
                        Some(n) => n.id,
                        None => {
                            return Task::none();
                        }
                    }
                }
                None => match state.endpoints.get(if state.endpoints.len() > 0 {
                    state.endpoints.len() - 1
                } else {
                    state.endpoints.len()
                }) {
                    Some(n) => n.id,
                    None => return Task::none(),
                },
            }),
        ),
    }
}

fn current_response(state: &State) -> Option<&Response> {
    match current_endpoint(state) {
        Some(endpoint) => endpoint.responses.get(state.selected_response_index),
        None => None,
    }
}

fn current_endpoint(state: &State) -> Option<&EndpointDb> {
    match state.selected_endpoint {
        Some(endpoint_id) => state.endpoints.iter().find(|it| it.id == endpoint_id),
        None => None,
    }
}

fn detect_mime_type(content: &str) -> &'static str {
    let trimmed = content.trim();

    if trimmed.is_empty() {
        return "text/plain";
    }

    // JSON - validate properly
    if let Ok(_) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return "application/json";
    }

    // XML
    if trimmed.starts_with("<?xml") || trimmed.starts_with("<") && trimmed.contains("</") {
        if trimmed.contains("<!DOCTYPE html") || trimmed.contains("<html") {
            return "text/html";
        }
        if trimmed.contains("<svg") {
            return "image/svg+xml";
        }
        return "application/xml";
    }

    // YAML
    if trimmed.starts_with("---") || (trimmed.contains(":\n") && !trimmed.contains('<')) {
        return "text/yaml";
    }

    "text/plain"
}

fn format_html(html: &str) -> String {
    format_text(html, Language::Html, &Default::default(), |s, _| {
        Ok::<_, std::convert::Infallible>(s.into())
    })
    .unwrap_or_else(|_| html.to_string())
}

fn format_response(s: &str) -> String {
    let t = detect_mime_type(s);
    match t {
        "text/html" => format_html(s),
        "application/json" => {
            let v: Result<Value, String> = serde_json::from_str(s).map_err(|it| it.to_string());
            match v {
                Ok(value) => {
                    let res = serde_json::to_string_pretty(&value).map_err(|it| it.to_string());
                    match res {
                        Ok(value) => value,
                        Err(error) => error,
                    }
                }
                Err(error) => error,
            }
        }
        _ => s.to_string(),
    }
}

fn view<'a>(state: &'a State) -> Element<'a, Message> {
    column![row![endpoint_list(state), column![content(state)]]]
        .padding(16)
        .into()
}

pub fn main() -> iced::Result {
    init().unwrap();
    iced::application("Interfere", update, view)
        .subscription(subscription)
        .theme(theme)
        .font(include_bytes!("./res/font/Geist-Regular.ttf").as_slice())
        .font(include_bytes!("./res/font/Geist-Light.ttf").as_slice())
        .font(include_bytes!("./res/font/Geist-Medium.ttf").as_slice())
        .default_font(GEIST_FONT)
        .run_with(State::new)
}

fn endpoint_list(state: &State) -> Element<'_, Message> {
    mr(
        column![
            column![
                mb(
                    row![
                        text("Interfere")
                            .size(32)
                            .font(Font {
                                weight: Weight::Bold,
                                ..GEIST_FONT
                            })
                            .color(state.theme.palette.primary),
                        bi(Icons::Plus, Some(Message::Back), ButtonType::Outlined)
                            .width(32)
                            .height(32),
                    ]
                    .align_y(Center)
                    .spacing(8)
                    .into(),
                    match &state.error_message {
                        Some(_) => 8.0,
                        None => 0.0,
                    }
                ),
                match &state.error_message {
                    Some(m) => {
                        bti(
                            m.clone(),
                            Icons::Close,
                            Some(Message::ClearErrorMessage),
                            ButtonType::Danger,
                        )
                    }
                    None => empty_b(),
                }
            ],
            mytext_input("Search...", &state.endp_search, &Message::SetSearch, None)
                .width(348)
                .id("searchbar"),
            Column::from_iter(state.endpoints.iter().map(|el| {
                row![
                    bt(
                        strip_url(&el.url),
                        Some(Message::ClickEndpoint(el.id)),
                        if state.selected_endpoint == Some(el.id) {
                            ButtonType::PrimaryInline
                        } else {
                            ButtonType::Inline
                        }
                    )
                    .width(Fill),
                    row![
                        container(
                            row![text(el.method.to_string()).style(|_| {
                                text::Style {
                                    color: Some(Color::BLACK),
                                    ..text::Style::default()
                                }
                            })]
                            .padding([6, 8])
                        )
                        .style(|_| {
                            container::Style {
                                background: Some(iced::Background::Color(color_for_method(
                                    el.method,
                                ))),
                                ..container::Style::default()
                            }
                        })
                    ],
                    bi(
                        Icons::Delete,
                        Some(Message::ClickDeleteEndpoint(el.id)),
                        ButtonType::Inline
                    )
                    .width(48)
                ]
                .align_y(Alignment::Center)
                .width(348)
                .into()
            }))
            .spacing(2)
        ]
        .spacing(16.0)
        .into(),
        16.0,
    )
    .into()
}

fn send_button(state: &State) -> Button<'_, Message> {
    bti(
        if state.copy_request.is_none() && current_response(state).is_some() {
            "Rerun"
        } else {
            "Send"
        }
        .to_string(),
        Icons::Enter,
        if state.can_send {
            Some(Message::Send)
        } else {
            None
        },
        ButtonType::Primary,
    )
    .height(Shrink)
    .padding([14, 16])
}

fn draft_urlbar<'a>(state: &'a State) -> Element<'a, Message> {
    let urlbar = mytext_input(
        "Input URL...",
        &state.draft,
        &Message::SetDraft,
        Some(Message::Send),
    )
    .id("main_urlbar");
    mb(
        column![
            row![urlbar, method_button(state), send_button(state)]
                .spacing(8)
                .align_y(Center)
        ]
        .into(),
        16.0,
    )
    .into()
}

fn method_button<'a>(state: &'a State) -> Button<'a, Message> {
    bt(
        state.draft_method.to_string(),
        Some(Message::ClickMethod),
        ButtonType::Outlined,
    )
}

fn content<'a>(state: &'a State) -> Column<'a, Message> {
    match current_endpoint(state) {
        Some(endpoint) => {
            let urlbar = card_clickable(
                row![
                    text(format_url_from_state(state)),
                    horizontal_space(),
                    svg(svg::Handle::from_memory(match_icon(Icons::Duplicate)))
                        .width(20)
                        .height(20)
                ]
                .align_y(Center),
                Some(Message::Duplicate(endpoint.url.clone())),
            )
            .width(Fill);
            column![
                mb(
                    column![
                        row![urlbar, send_button(state)]
                            .width(Fill)
                            .spacing(8)
                            .align_y(Center)
                    ]
                    .into(),
                    16.0,
                ),
                row![
                    column![
                        query_param_panel(state, endpoint),
                        header_panel(state, endpoint)
                    ]
                    .spacing(16),
                    match state.draft_response {
                        Some(_) => {
                            draft_response_panel(state)
                        }
                        None => {
                            container(
                                match &endpoint.responses.get(state.selected_response_index) {
                                    Some(resp) => {
                                        response_panels(resp, state, endpoint.responses.len())
                                    }
                                    None => {
                                        column![]
                                    }
                                },
                            )
                        }
                    },
                ]
            ]
        }
        None => column![
            draft_urlbar(state),
            row![
                column![draft_query_param_panel(state), draft_header_panel(state)].spacing(16),
                match state.draft_response {
                    Some(_) => {
                        draft_response_panel(state)
                    }
                    None => container(column![]),
                },
            ],
        ],
    }
}

fn draft_query_param_panel<'a>(state: &'a State) -> Container<'a, Message> {
    container(
        column![
            row![
                text("Query params"),
                bt(
                    "Add",
                    Some(Message::QueryParam(MQueryParam::AddQueryParam())),
                    ButtonType::Primary,
                )
            ]
            .spacing(16)
            .padding([0, 8])
            .align_y(Center)
            .width(Fill),
            Column::from_iter(
                state
                    .draft_request
                    .query_params
                    .iter()
                    .map(|it| query_row(it, state, false).into()),
            )
            .spacing(8)
        ]
        .spacing(16),
    )
    .style(|t| container::Style {
        border: Border::default().rounded(16),
        background: Some(iced::Background::Color(t.palette().background)),
        ..container::Style::default()
    })
}

fn draft_response_panel(state: &State) -> container::Container<'_, Message> {
    match &state.draft_response {
        Some(draft) => card(column![
            mb(text("Temporary response").into(), 8.0),
            row![
                bi(
                    Icons::Left,
                    Some(Message::DiscardDraftResponse),
                    ButtonType::Text
                ),
                row![
                    container(text!("{}", draft.0).style(|_| {
                        text::Style {
                            color: Some(Color::BLACK),
                        }
                    }))
                    .padding([2, 4])
                    .style(|_| {
                        container::Style {
                            background: Some(iced::Background::Color(color_for_status(draft.0))),
                            ..container::Style::default()
                        }
                    }),
                    column![
                        text(draft.2.format("%H:%M:%S").to_string())
                            .size(14)
                            .line_height(1.0),
                        text(draft.2.format("%d-%m-%Y").to_string())
                            .size(10)
                            .line_height(0.9)
                    ]
                ]
                .align_y(Center)
                .spacing(8)
                .width(Fill),
                bi(
                    Icons::Format,
                    Some(Message::FormatResponse),
                    ButtonType::Text
                )
            ]
            .align_y(Center)
            .spacing(8),
            mb(
                scrollable(match &state.formatted_response {
                    Some(fmt) => text(fmt),
                    None => text(&draft.1),
                })
                .direction(iced::widget::scrollable::Direction::Both {
                    vertical: Scrollbar::default(),
                    horizontal: Scrollbar::default()
                })
                .height(Fill)
                .width(Fill)
                .into(),
                16.0
            )
            .padding([16, 0])
        ])
        .width(Fill),
        None => container(column![]),
    }
}

fn query_row<'a>(
    it: &'a EndpointKvPair,
    state: &'a State,
    is_header: bool,
) -> container::Container<'a, Message> {
    container(
        match state.selected_endpoint {
            Some(_) => match &state.copy_request {
                Some(_) => {
                    row![
                        mytext_input(
                            "Name",
                            &it.key,
                            {
                                let it = it.clone();
                                move |new_content| {
                                    if is_header {
                                        Message::Header(MHeader::SetHeaderKey(it.id, new_content))
                                    } else {
                                        Message::QueryParam(MQueryParam::SetQueryParamKey(
                                            it.id,
                                            new_content,
                                        ))
                                    }
                                }
                            },
                            None
                        )
                        .id(format!("query_param_{}", it.id)),
                        mytext_input(
                            "Value",
                            &it.value,
                            {
                                let it = it.clone();
                                move |new_content| {
                                    if is_header {
                                        Message::Header(MHeader::SetHeaderContent(
                                            it.id,
                                            new_content,
                                        ))
                                    } else {
                                        Message::QueryParam(MQueryParam::SetQueryParamContent(
                                            it.id,
                                            new_content,
                                        ))
                                    }
                                }
                            },
                            None,
                        ),
                        bi(
                            Icons::Check,
                            Some(if is_header {
                                Message::Header(MHeader::ToggleHeaderIsOn(it.id))
                            } else {
                                Message::QueryParam(MQueryParam::ToggleQueryParamIsOn(it.id))
                            }),
                            if it.on {
                                ButtonType::Primary
                            } else {
                                ButtonType::Text
                            },
                        ),
                        bi(
                            Icons::Delete,
                            Some(if is_header {
                                Message::Header(MHeader::DeleteHeader(it.id))
                            } else {
                                Message::QueryParam(MQueryParam::DeleteQueryParam(it.id))
                            }),
                            ButtonType::Text,
                        )
                    ]
                }
                None => {
                    row![
                        Element::from(card(row![text(&it.key).width(Fill)])),
                        Element::from(card(row![text(&it.value)].width(Fill))),
                    ]
                }
            },
            None => {
                row![
                    mytext_input(
                        "Name",
                        &it.key,
                        {
                            let it = it.clone();
                            move |new_content| {
                                if is_header {
                                    Message::Header(MHeader::SetHeaderKey(it.id, new_content))
                                } else {
                                    Message::QueryParam(MQueryParam::SetQueryParamKey(
                                        it.id,
                                        new_content,
                                    ))
                                }
                            }
                        },
                        None
                    )
                    .id(format!("query_param_{}", it.id)),
                    mytext_input(
                        "Value",
                        &it.value,
                        {
                            let it = it.clone();
                            move |new_content| {
                                if is_header {
                                    Message::Header(MHeader::SetHeaderContent(it.id, new_content))
                                } else {
                                    Message::QueryParam(MQueryParam::SetQueryParamContent(
                                        it.id,
                                        new_content,
                                    ))
                                }
                            }
                        },
                        None,
                    ),
                    bi(
                        Icons::Check,
                        Some(if is_header {
                            Message::Header(MHeader::ToggleHeaderIsOn(it.id))
                        } else {
                            Message::QueryParam(MQueryParam::ToggleQueryParamIsOn(it.id))
                        }),
                        if it.on {
                            ButtonType::Primary
                        } else {
                            ButtonType::Text
                        },
                    ),
                    bi(
                        Icons::Delete,
                        Some(if is_header {
                            Message::Header(MHeader::DeleteHeader(it.id))
                        } else {
                            Message::QueryParam(MQueryParam::DeleteQueryParam(it.id))
                        }),
                        ButtonType::Text,
                    )
                ]
            }
        }
        .align_y(Center)
        .spacing(8),
    )
    .style(|t| container::Style {
        border: Border::default().rounded(16),
        background: Some(iced::Background::Color(t.palette().background)),
        ..container::Style::default()
    })
}

fn query_param_panel<'a>(
    state: &'a State,
    endpoint: &'a EndpointDb,
) -> container::Container<'a, Message> {
    container(
        column![
            if state.copy_request.is_none() && current_response(state).is_some() {
                row![
                    column![
                        bt(
                            "Copy",
                            Some(Message::SetDraftQuery(true)),
                            ButtonType::Outlined,
                        ),
                        text("Query params"),
                    ]
                    .spacing(16)
                ]
            } else {
                row![
                    text("Query params"),
                    bt(
                        "Add",
                        Some(Message::QueryParam(MQueryParam::AddQueryParam())),
                        ButtonType::Primary,
                    )
                ]
                .spacing(16)
                .align_y(Center)
            }
            .padding([0, 8])
            .width(Fill),
            match &state.copy_request {
                Some(drafts) => {
                    Column::from_iter(
                        drafts
                            .query_params
                            .iter()
                            .map(|it| query_row(it, state, false).into()),
                    )
                }
                None => {
                    match endpoint.responses.get(state.selected_response_index) {
                        Some(resp) => Column::from_iter(
                            resp.request
                                .query_params
                                .iter()
                                .map(|it| query_row(it, state, false).into()),
                        ),
                        None => column![],
                    }
                }
            }
            .spacing(8)
        ]
        .spacing(16),
    )
    .style(|t| container::Style {
        border: Border::default().rounded(16),
        background: Some(iced::Background::Color(t.palette().background)),
        ..container::Style::default()
    })
}

fn draft_header_panel<'a>(state: &'a State) -> Container<'a, Message> {
    container(
        column![
            row![
                text("Headers"),
                bt(
                    "Add",
                    Some(Message::Header(MHeader::AddHeader())),
                    ButtonType::Primary,
                )
            ]
            .spacing(16)
            .padding([0, 8])
            .align_y(Center)
            .width(Fill),
            Column::from_iter(
                state
                    .draft_request
                    .headers
                    .iter()
                    .map(|it| query_row(it, state, true).into()),
            )
            .spacing(8)
        ]
        .spacing(16),
    )
    .style(|t| container::Style {
        border: Border::default().rounded(16),
        background: Some(iced::Background::Color(t.palette().background)),
        ..container::Style::default()
    })
}

fn header_panel<'a>(
    state: &'a State,
    endpoint: &'a EndpointDb,
) -> container::Container<'a, Message> {
    container(
        column![
            row![
                text("Headers"),
                if state.copy_request.is_some() || current_response(state).is_none() {
                    bt(
                        "Add",
                        Some(Message::Header(MHeader::AddHeader())),
                        ButtonType::Primary,
                    )
                } else {
                    empty_b()
                }
            ]
            .spacing(16)
            .padding([0, 8])
            .align_y(Center)
            .width(Fill),
            match &state.copy_request {
                Some(drafts) => {
                    Column::from_iter(
                        drafts
                            .headers
                            .iter()
                            .map(|it| query_row(it, state, true).into()),
                    )
                }
                None => {
                    match endpoint.responses.get(state.selected_response_index) {
                        Some(resp) => Column::from_iter(
                            resp.request
                                .headers
                                .iter()
                                .map(|it| query_row(it, state, true).into()),
                        ),
                        None => column![],
                    }
                }
            }
            .spacing(8)
        ]
        .spacing(16),
    )
    .style(|t| container::Style {
        border: Border::default().rounded(16),
        background: Some(iced::Background::Color(t.palette().background)),
        ..container::Style::default()
    })
}

fn strip_url(url: &str) -> String {
    let prefixes = ["https://", "http://"];
    for p in &prefixes {
        match url.strip_prefix(p) {
            Some(it) => {
                return it.to_string();
            }
            None => {}
        }
    }
    url.to_string()
}

fn method_from_state(state: &State) -> HttpMethod {
    match current_endpoint(state) {
        Some(endpoint) => endpoint.method,
        None => state.draft_method,
    }
}

fn format_url_from_state(state: &State) -> String {
    match current_endpoint(state) {
        Some(endpoint) => {
            let empty = [].to_vec().clone();
            let query_param_vec: Vec<String> = (match &state.copy_request {
                Some(drafts) => &drafts.query_params,
                None => match endpoint.responses.get(state.selected_response_index) {
                    Some(resp) => &resp.request.query_params,
                    None => &empty,
                },
            })
            .iter()
            .filter(|it| !it.key.is_empty() && it.on)
            .map(|it| {
                format!(
                    "{}={}",
                    urlencoding::encode(&it.key),
                    urlencoding::encode(&it.value)
                )
            })
            .collect();
            format!(
                "{}{}{}",
                endpoint.url,
                if query_param_vec.is_empty() { "" } else { "?" },
                query_param_vec.join("&")
            )
        }
        None => {
            let query_param_vec: Vec<String> = state
                .draft_request
                .query_params
                .iter()
                .filter(|it| !it.key.is_empty() && it.on)
                .map(|it| {
                    format!(
                        "{}={}",
                        urlencoding::encode(&it.key),
                        urlencoding::encode(&it.value)
                    )
                })
                .collect();
            format!(
                "{}{}{}",
                state.draft,
                if query_param_vec.is_empty() { "" } else { "?" },
                query_param_vec.join("&")
            )
        }
    }
}

fn headers_from_state(state: &State) -> HeaderMap {
    let empty: Vec<EndpointKvPair> = [].to_vec().clone();
    match current_endpoint(state) {
        Some(endpoint) => HeaderMap::from_iter(
            (match &state.copy_request {
                Some(drafts) => &drafts.headers,
                None => match endpoint.responses.get(state.selected_response_index) {
                    Some(resp) => &resp.request.headers,
                    None => &empty,
                },
            })
            .iter()
            .filter(|it| !it.key.is_empty() && it.on)
            .map(|it| {
                (
                    HeaderName::from_str(&it.key).unwrap(),
                    HeaderValue::from_str(&it.value).unwrap(),
                )
            }),
        ),
        None => HeaderMap::from_iter(
            state
                .draft_request
                .headers
                .iter()
                .filter(|it| !it.key.is_empty() && it.on)
                .map(|it| {
                    (
                        HeaderName::from_str(&it.key).unwrap(),
                        HeaderValue::from_str(&it.value).unwrap(),
                    )
                }),
        ),
    }
}

fn response_panels<'a>(
    resp: &'a Response,
    state: &'a State,
    resp_count: usize,
) -> Column<'a, Message, Theme, Renderer> {
    let time = Local::now().offset().from_utc_datetime(&resp.received_time);
    column![
        mb(
            ml(
                row![
                    bi(
                        Icons::Plus,
                        Some(Message::SetDraftQuery(false)),
                        if state.copy_request.is_none() {
                            ButtonType::Outlined
                        } else {
                            ButtonType::Primary
                        }
                    ),
                    scrollable(
                        Row::from_iter((1..resp_count + 1).rev().map(|index| {
                            bt(
                                index,
                                Some(Message::SetSelectedResponseIndex(index - 1)),
                                if index == state.selected_response_index + 1
                                    && state.copy_request.is_none()
                                {
                                    ButtonType::Primary
                                } else {
                                    ButtonType::Text
                                },
                            )
                            .into()
                        }))
                        .spacing(4)
                        .align_y(Center)
                    )
                    .direction(scrollable::Direction::Horizontal(
                        Scrollbar::default().width(0).scroller_width(0)
                    ))
                    .width(Fill)
                ]
                .spacing(8)
                .align_y(Center)
                .into(),
                16.0
            )
            .into(),
            16.0
        ),
        ml(
            card(column![
                row![
                    row![
                        container(text!("{}", resp.code).style(|_| text::Style {
                            color: Some(Color::BLACK)
                        }))
                        .padding([2, 4])
                        .style(|_| {
                            container::Style {
                                background: Some(iced::Background::Color(color_for_status(
                                    resp.code,
                                ))),
                                ..container::Style::default()
                            }
                        }),
                        column![
                            text(time.format("%H:%M:%S").to_string())
                                .size(14)
                                .line_height(1.0),
                            text(time.format("%d-%m-%Y").to_string())
                                .size(10)
                                .line_height(0.9)
                        ]
                    ]
                    .align_y(Center)
                    .spacing(8)
                    .width(Fill),
                    bi(
                        Icons::Format,
                        Some(Message::FormatResponse),
                        ButtonType::Text
                    ),
                    bi(
                        Icons::Delete,
                        Some(Message::ClickDeleteResponse(resp.id)),
                        ButtonType::Text
                    )
                ]
                .align_y(Center)
                .spacing(8),
                mb(
                    scrollable(match &state.formatted_response {
                        Some(fmt) => text(fmt),
                        None => text(&resp.text),
                    })
                    .direction(iced::widget::scrollable::Direction::Both {
                        vertical: Scrollbar::default(),
                        horizontal: Scrollbar::default()
                    })
                    .height(Fill)
                    .width(Fill)
                    .into(),
                    16.0
                )
                .padding([16, 0])
            ])
            .width(Fill)
            .into(),
            16.0,
        )
    ]
}

async fn send_request(
    url: String,
    headers: HeaderMap,
    method: HttpMethod,
) -> Result<(String, StatusCode), MyErr> {
    let client = reqwest::Client::new();
    let resp = match method {
        HttpMethod::GET => client.get(url),
        HttpMethod::POST => client.post(url),
    }
    .headers(headers)
    .send()
    .await?;
    let status = resp.status();
    let text = resp.text().await?;
    Ok((text, status))
}

fn subscription(_state: &State) -> Subscription<Message> {
    Subscription::batch([
        keyboard::on_key_press(|key, mods| match key {
            keyboard::key::Key::Named(keyboard::key::Named::Enter) => {
                if mods.contains(Modifiers::CTRL) {
                    Some(Message::SendDraft)
                } else {
                    Some(Message::Send)
                }
            }
            keyboard::key::Key::Named(keyboard::key::Named::Escape) => Some(Message::Back),
            keyboard::key::Key::Named(keyboard::key::Named::ArrowLeft) => {
                if mods.contains(Modifiers::CTRL) {
                    Some(Message::IncrementSelectedResponseIndex)
                } else {
                    None
                }
            }
            keyboard::key::Key::Named(keyboard::key::Named::ArrowRight) => {
                if mods.contains(Modifiers::CTRL) {
                    Some(Message::DecrementSelectedResponseIndex)
                } else {
                    None
                }
            }
            keyboard::key::Key::Named(keyboard::key::Named::ArrowDown) => {
                if mods.contains(Modifiers::CTRL) {
                    Some(Message::IncrementSelectedEndpoint)
                } else {
                    None
                }
            }
            keyboard::key::Key::Named(keyboard::key::Named::ArrowUp) => {
                if mods.contains(Modifiers::CTRL) {
                    Some(Message::DecrementSelectedEndpoint)
                } else {
                    None
                }
            }
            keyboard::key::Key::Character(key) => {
                if mods.contains(Modifiers::CTRL) && key == "f" {
                    Some(Message::Focus("searchbar"))
                } else {
                    None
                }
            }

            _ => {
                if mods.contains(Modifiers::CTRL) {
                    Some(Message::SetCtrlPressed(true))
                } else {
                    None
                }
            }
        }),
        keyboard::on_key_release(|key, _mods| match key {
            keyboard::key::Key::Named(keyboard::key::Named::Control) => {
                Some(Message::SetCtrlPressed(false))
            }
            _ => None,
        }),
    ])
}

fn theme(state: &State) -> Theme {
    Theme::custom("Abobolik".into(), state.theme.palette)
}

fn color_for_status(code: StatusCode) -> Color {
    match code {
        StatusCode::OK => Color::parse("#9ECE6A"),
        StatusCode::FOUND => Color::parse("#414868"),
        _ => Color::parse("#F7768E"),
    }
    .unwrap()
}

fn color_for_method(method: HttpMethod) -> Color {
    match method {
        HttpMethod::GET => Color::parse("#9ECE6A"),
        HttpMethod::POST => Color::parse("#e8de6d"),
        _ => Color::parse("#414868"),
    }
    .unwrap()
}
