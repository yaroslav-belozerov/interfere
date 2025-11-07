mod logic;

use crate::AppTheme;
use chrono::{Local, NaiveDateTime, TimeZone};
use iced::font::Weight;
use iced::theme::Palette;
use iced::widget::scrollable::Scrollbar;
use iced::widget::text_input::focus;
use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, svg, text, Button, Column, Row,
};
use iced::Alignment::Center;
use iced::Length::{Fill, Shrink};
use iced::{keyboard, Border, Color, Element, Font, Subscription, Task, Theme};
use logic::common::*;
use logic::crud::endpoint::{self, create_endpoint_full, delete_endpoint};
use logic::crud::query::{
    create_query_param, create_query_param_with_tx, delete_query_param, update_query_param_key,
    update_query_param_on, update_query_param_value,
};
use logic::crud::response::{create_response, delete_response, response_count_by_endpoint_id};
use logic::db::{get_db, init, load_endpoints};
use logic::ui::*;
use markup_fmt::{format_text, Language};
use reqwest::StatusCode;
use serde_json::Value;
use std::cmp::max;

impl State {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                draft: Default::default(),
                draft_query: None,
                endpoints: load_endpoints(&get_db().lock().unwrap(), None).unwrap(),
                endp_search: "".to_string(),
                can_send: true,
                selected_endpoint: None,
                selected_response_index: 0,
                formatted_response: None,
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

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::SetDraftQuery(copy) => {
            if copy {
                match current_response(state) {
                    Some(resp) => {
                        state.draft_query = Some(resp.query_params.clone());
                    }
                    None => {}
                }
            } else {
                state.draft_query = Some(Vec::new());
            }
            Task::none()
        }
        Message::Duplicate(s) => Task::batch([
            update(state, Message::Back),
            update(state, Message::SetDraft(s)),
        ]),
        Message::Start => focus("main_urlbar"),
        Message::RefetchDb => {
            state.endpoints =
                load_endpoints(&get_db().lock().unwrap(), Some(&state.endp_search)).unwrap();
            Task::none()
        }
        Message::SetSelectedResponseIndex(index) => {
            state.formatted_response = None;
            state.selected_response_index = index;
            state.draft_query = None;
            Task::none()
        }
        Message::SetDraft(string) => {
            state.draft = string;
            Task::none()
        }
        Message::Send => {
            state.can_send = false;
            Task::perform(
                send_get_request(format_url_from_state(state)),
                |res| match res {
                    Ok((text, code)) => Message::GotResponse(text, code),
                    Err(err) => Message::GotError(err),
                },
            )
        }
        Message::Back => {
            state.selected_endpoint = None;
            focus("main_urlbar")
        }
        Message::GotResponse(text, code) => {
            state.can_send = true;
            match state.selected_endpoint {
                Some(id) => {
                    let resp_id =
                        create_response(&get_db().lock().unwrap(), id, &text, code).unwrap();
                    match &state.draft_query {
                        Some(d) => {
                            let mut db = get_db().lock().unwrap();
                            let tx = db.transaction().unwrap();
                            for q in d {
                                create_query_param_with_tx(&tx, resp_id, &q.key, &q.value, q.on)
                                    .unwrap();
                            }
                            tx.commit().unwrap();
                            state.draft_query = None;
                        }
                        None => {}
                    }
                    state.selected_response_index = max(
                        response_count_by_endpoint_id(&get_db().lock().unwrap(), id).unwrap()
                            as usize
                            - 1,
                        0,
                    );
                }
                None => {
                    let id = create_endpoint_full(
                        &get_db().lock().unwrap(),
                        &EndpointDb {
                            id: 0,
                            url: state.draft.clone(),
                            responses: [Response {
                                id: 0,
                                parent_endpoint_id: 0,
                                text,
                                code,
                                received_time: NaiveDateTime::default(),
                                query_params: [].to_vec(),
                                headers: [].to_vec(),
                            }]
                            .to_vec(),
                        },
                    )
                    .unwrap();
                    return Task::batch([
                        update(state, Message::ClickEndpoint(id)),
                        update(state, Message::RefetchDb),
                    ]);
                }
            }
            state.draft = "".to_string();
            update(state, Message::RefetchDb)
        }
        Message::GotError(_err) => {
            state.can_send = true;
            Task::none()
        }
        Message::AddQueryParam() => match &mut state.draft_query {
            Some(q) => {
                q.push(EndpointKvPair {
                    id: q.len() as u64,
                    parent_response_id: 0,
                    key: "".to_string(),
                    value: "".to_string(),
                    on: true,
                });
                Task::none()
            }
            None => Task::none(),
        },
        Message::SetQueryParamContent(id, content) => match &mut state.draft_query {
            Some(q) => {
                if let Some(elem) = q.iter_mut().find(|it| it.id == id) {
                    elem.value = content.clone();
                };
                Task::none()
            }
            None => {
                update_query_param_value(&get_db().lock().unwrap(), id, &content).unwrap();
                update(state, Message::RefetchDb)
            }
        },
        Message::SetQueryParamKey(id, content) => match &mut state.draft_query {
            Some(q) => {
                if let Some(elem) = q.iter_mut().find(|it| it.id == id) {
                    elem.key = content.clone();
                };
                Task::none()
            }
            None => {
                update_query_param_key(&get_db().lock().unwrap(), id, &content).unwrap();
                update(state, Message::RefetchDb)
            }
        },

        Message::ClickEndpoint(id) => {
            state.formatted_response = None;
            state.draft_query = None;
            state.selected_endpoint = Some(id);
            let count =
                response_count_by_endpoint_id(&get_db().lock().unwrap(), id).unwrap() as usize;
            state.selected_response_index = max(count, 1) - 1;
            Task::none()
        }
        Message::ClickDeleteEndpoint(id) => {
            match state.selected_endpoint {
                Some(selected_id) => {
                    if id == selected_id {
                        delete_endpoint(&get_db().lock().unwrap(), selected_id).unwrap();
                        if state.draft_query.is_some() {
                            state.draft_query = None;
                        }
                        state.selected_endpoint = None;
                    }
                }
                None => {}
            }
            update(state, Message::RefetchDb)
        }
        Message::ToggleQueryParamIsOn(id) => match &mut state.draft_query {
            Some(q) => {
                if let Some(elem) = q.iter_mut().find(|it| it.id == id) {
                    elem.on = !elem.on;
                };
                Task::none()
            }
            None => {
                update_query_param_on(&get_db().lock().unwrap(), id).unwrap();
                update(state, Message::RefetchDb)
            }
        },
        Message::ClickDeleteResponse(id) => match state.draft_query {
            Some(_) => {
                state.draft_query = None;
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
        Message::DeleteQueryParam(id) => {
            match &mut state.draft_query {
                Some(draft) => match draft.iter().position(|it| it.id == id) {
                    Some(found) => {
                        draft.remove(found);
                    }
                    None => {}
                },
                None => {
                    delete_query_param(&get_db().lock().unwrap(), id).unwrap();
                }
            }
            update(state, Message::RefetchDb)
        }
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
                        None => {}
                    }
                }
                None => {}
            };
            Task::none()
        }
        Message::Empty => Task::none(),
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
        _ => todo!(),
    }
}

fn view(state: &State) -> Element<Message> {
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

fn endpoint_list(state: &State) -> Element<Message> {
    mr(
        column![
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
                    .height(32)
            ]
            .align_y(Center)
            .spacing(8),
            mytext_input("Search...", &state.endp_search, &Message::SetSearch, None).width(312),
            Column::from_iter(state.endpoints.iter().map(|el| {
                row![
                    bt(
                        el.url.strip_prefix("https://").unwrap(),
                        Some(Message::ClickEndpoint(el.id)),
                        ButtonType::Text
                    )
                    .width(Fill),
                    bi(
                        Icons::Delete,
                        Some(Message::ClickDeleteEndpoint(el.id)),
                        ButtonType::Danger
                    )
                    .width(48)
                ]
                .width(312)
                .spacing(8)
                .into()
            }))
            .spacing(8)
        ]
        .spacing(16.0)
        .into(),
        16.0,
    )
    .into()
}

fn send_button(state: &State) -> Button<Message> {
    bti(
        "Send",
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

fn draft_urlbar(state: &State) -> Element<Message> {
    let urlbar = mytext_input(
        "Input URL...",
        &state.draft,
        &Message::SetDraft,
        Some(Message::Send),
    )
    .id("main_urlbar");
    mb(
        column![row![urlbar, send_button(state)].spacing(8).align_y(Center)].into(),
        16.0,
    )
    .into()
}

fn content(state: &State) -> Column<Message> {
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
                    column![row![urlbar, send_button(state)]
                        .width(Fill)
                        .spacing(8)
                        .align_y(Center)]
                    .into(),
                    16.0,
                ),
                row![query_param_panel(state, endpoint), response_panels(state)]
            ]
        }
        None => column![draft_urlbar(state)],
    }
}

fn query_param_panel(
    state: &State,
    endpoint: &EndpointDb,
) -> container::Container<'static, Message> {
    let query_params = match endpoint.responses.get(state.selected_response_index) {
        Some(resp) => Column::from_iter(
            (match &state.draft_query {
                Some(drafts) => drafts,
                None => &resp.query_params,
            })
            .iter()
            .map(|it| {
                container(
                    row![
                        mytext_input(
                            "Name",
                            &it.key,
                            {
                                let it = it.clone();
                                move |new_content| Message::SetQueryParamKey(it.id, new_content)
                            },
                            None
                        )
                        .id(format!("query_param_{}", it.id)),
                        mytext_input(
                            "Value",
                            &it.value,
                            {
                                let it = it.clone();
                                move |new_content| Message::SetQueryParamContent(it.id, new_content)
                            },
                            None
                        ),
                        bi(
                            Icons::Check,
                            Some(Message::ToggleQueryParamIsOn(it.id)),
                            if it.on {
                                ButtonType::Primary
                            } else {
                                ButtonType::Text
                            }
                        ),
                        bi(
                            Icons::Delete,
                            Some(Message::DeleteQueryParam(it.id)),
                            ButtonType::Text
                        )
                    ]
                    .align_y(Center)
                    .spacing(8),
                )
                .style(|t| container::Style {
                    border: Border::default().rounded(16),
                    background: Some(iced::Background::Color(t.palette().background)),
                    ..container::Style::default()
                })
                .into()
            }),
        )
        .spacing(8),
        None => column![],
    };
    container(
        column![
            row![
                text("Query params"),
                horizontal_space(),
                if state.draft_query.is_none() {
                    bt(
                        "Copy",
                        Some(Message::SetDraftQuery(true)),
                        ButtonType::Outlined,
                    )
                } else {
                    bt("Add", Some(Message::AddQueryParam()), ButtonType::Primary)
                }
            ]
            .padding([0, 8])
            .align_y(Center),
            query_params
        ]
        .spacing(16),
    )
    .style(|t| container::Style {
        border: Border::default().rounded(16),
        background: Some(iced::Background::Color(t.palette().background)),
        ..container::Style::default()
    })
}

fn format_url_from_state(state: &State) -> String {
    match current_endpoint(state) {
        Some(endpoint) => match endpoint.responses.get(state.selected_response_index) {
            Some(resp) => {
                let query_param_vec: Vec<String> = (match &state.draft_query {
                    Some(drafts) => drafts,
                    None => &resp.query_params,
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
            None => endpoint.url.clone(),
        },
        None => state.draft.clone(),
    }
}

fn response_panels(state: &State) -> Element<Message> {
    match current_endpoint(state) {
        Some(e) => {
            let current_response = e.responses.get(state.selected_response_index);
            match current_response {
                Some(resp) => {
                    let time = Local::now().offset().from_utc_datetime(&resp.received_time);
                    column![
                        mb(
                            ml(
                                row![
                                    bi(
                                        Icons::Plus,
                                        Some(Message::SetDraftQuery(false)),
                                        if state.draft_query.is_none() {
                                            ButtonType::Outlined
                                        } else {
                                            ButtonType::Primary
                                        }
                                    ),
                                    scrollable(
                                        Row::from_iter((1..e.responses.len() + 1).rev().map(
                                            |index| {
                                                bt(
                                                    index,
                                                    Some(Message::SetSelectedResponseIndex(
                                                        index - 1,
                                                    )),
                                                    if index == state.selected_response_index + 1
                                                        && state.draft_query.is_none()
                                                    {
                                                        ButtonType::Primary
                                                    } else {
                                                        ButtonType::Text
                                                    },
                                                )
                                                .into()
                                            }
                                        ))
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
                                                background: Some(iced::Background::Color(
                                                    color_for_status(resp.code),
                                                )),
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
                                        Icons::Check,
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
                None => column![],
            }
            .width(Fill)
        }
        None => column![],
    }
    .into()
}

impl From<reqwest::Error> for MyErr {
    fn from(err: reqwest::Error) -> Self {
        if err.is_builder() {
            Self::Client("Invalid URL scheme.".to_string())
        } else {
            Self::Unknown(err.to_string())
        }
    }
}

async fn send_get_request(url: String) -> Result<(String, StatusCode), MyErr> {
    let resp = reqwest::get(url).await?;
    let status = resp.status();
    let text = resp.text().await?;
    Ok((text, status))
}

fn subscription(_state: &State) -> Subscription<Message> {
    keyboard::on_key_press(|key, _mods| match key {
        keyboard::key::Key::Named(keyboard::key::Named::Enter) => Some(Message::Send),
        keyboard::key::Key::Named(keyboard::key::Named::Escape) => Some(Message::Back),
        _ => None,
    })
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
