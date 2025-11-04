mod logic;

use html5ever::tendril::TendrilSink;
use std::cmp::max;

use crate::AppTheme;
use chrono::{Local, NaiveDateTime, TimeZone};
use html5ever::parse_document;
use iced::font::Weight;
use iced::theme::Palette;
use iced::widget::scrollable::Scrollbar;
use iced::widget::text_input::focus;
use iced::widget::{
    column, container, horizontal_space, row, scrollable, svg, text, Button, Column,
};
use iced::Alignment::Center;
use iced::Length::{Fill, Shrink};
use iced::{keyboard, Border, Color, Element, Font, Subscription, Task, Theme};
use logic::common::*;
use logic::crud::endpoint::{create_endpoint_full, delete_endpoint};
use logic::crud::query::{
    create_query_param, delete_query_param, update_query_param_key, update_query_param_on,
    update_query_param_value,
};
use logic::crud::response::{create_response, delete_response, responses_by_endpoint_id};
use logic::db::{get_db, init, load_endpoints};
use logic::ui::*;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use reqwest::StatusCode;
use serde_json::Value;

impl State {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                draft: Default::default(),
                endpoints: load_endpoints(&get_db().lock().unwrap()).unwrap(),
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
        Message::Duplicate(s) => Task::batch([
            update(state, Message::Back),
            update(state, Message::SetDraft(s)),
        ]),
        Message::Start => focus("main_urlbar"),
        Message::RefetchDb => {
            state.endpoints = load_endpoints(&get_db().lock().unwrap()).unwrap();
            Task::none()
        }
        Message::SetSelectedResponseIndex(index) => {
            state.formatted_response = None;
            state.selected_response_index = index;
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
                    create_response(&get_db().lock().unwrap(), id, &text, code).unwrap();
                    state.selected_response_index = max(
                        responses_by_endpoint_id(&get_db().lock().unwrap(), id)
                            .unwrap()
                            .len()
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
                            }]
                            .to_vec(),
                            query_params: [].to_vec(),
                            headers: [].to_vec(),
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
        Message::AddQueryParam() => match state.selected_endpoint {
            Some(id) => {
                create_query_param(&get_db().lock().unwrap(), id, "", "", true).unwrap();
                update(state, Message::RefetchDb)
            }
            None => Task::none(),
        },
        Message::SetQueryParamContent(id, content) => {
            update_query_param_value(&get_db().lock().unwrap(), id, &content).unwrap();
            update(state, Message::RefetchDb)
        }
        Message::SetQueryParamKey(id, content) => {
            update_query_param_key(&get_db().lock().unwrap(), id, &content).unwrap();
            update(state, Message::RefetchDb)
        }
        Message::ClickEndpoint(id) => {
            state.formatted_response = None;
            state.selected_endpoint = Some(id);
            state.selected_response_index = max(
                responses_by_endpoint_id(&get_db().lock().unwrap(), id)
                    .unwrap()
                    .len(),
                1,
            ) - 1;
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
        Message::ToggleQueryParamIsOn(id) => {
            update_query_param_on(&get_db().lock().unwrap(), id).unwrap();
            update(state, Message::RefetchDb)
        }
        Message::ClickDeleteResponse(id) => {
            match state.selected_endpoint {
                Some(endpoint_id) => {
                    match state.endpoints.iter().find(|&it| it.id == endpoint_id) {
                        Some(endpoint) => {
                            if state.selected_response_index + 1 == endpoint.responses.iter().len()
                            {
                                state.selected_response_index =
                                    max(state.selected_response_index, 1) - 1;
                            }
                        }
                        None => return Task::none(),
                    }
                }
                None => return Task::none(),
            }
            delete_response(&get_db().lock().unwrap(), id).unwrap();
            update(state, Message::RefetchDb)
        }
        Message::DeleteQueryParam(id) => {
            delete_query_param(&get_db().lock().unwrap(), id).unwrap();
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
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .unwrap();

    let mut output = String::new();

    // Check for DOCTYPE
    if html.trim_start().to_lowercase().starts_with("<!doctype") {
        output.push_str("<!DOCTYPE html>\n");
    }

    for child in dom.document.children.borrow().iter() {
        if let NodeData::Doctype { .. } = child.data {
            continue;
        }
        serialize_node(child, &mut output, 0);
    }

    output
}
fn serialize_node(node: &Handle, output: &mut String, indent: usize) {
    let indent_str = "  ".repeat(indent);

    match &node.data {
        NodeData::Element { name, attrs, .. } => {
            // Opening tag
            output.push_str(&indent_str);
            output.push('<');
            output.push_str(&name.local.to_string());

            for attr in attrs.borrow().iter() {
                output.push(' ');
                output.push_str(&attr.name.local.to_string());
                output.push_str("=\"");
                output.push_str(&attr.value);
                output.push('"');
            }
            output.push('>');

            // Children
            let children = node.children.borrow();
            let has_element_children = children
                .iter()
                .any(|child| matches!(child.data, NodeData::Element { .. }));

            if has_element_children {
                output.push('\n');
                for child in children.iter() {
                    serialize_node(child, output, indent + 1);
                }
                output.push_str(&indent_str);
            } else {
                // Inline text content
                for child in children.iter() {
                    if let NodeData::Text { ref contents } = child.data {
                        output.push_str(&contents.borrow());
                    }
                }
            }

            // Closing tag
            output.push_str("</");
            output.push_str(&name.local.to_string());
            output.push_str(">\n");
        }
        NodeData::Text { contents } => {
            let text = contents.borrow();
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                output.push_str(&indent_str);
                output.push_str(trimmed);
                output.push('\n');
            }
        }
        _ => {}
    }
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
                        .height(32)
                ]
                .align_y(Center)
                .spacing(8)
                .into(),
                16.0
            ),
            Column::from_iter(state.endpoints.iter().map(|el| {
                row![
                    bt(
                        el.url.strip_prefix("https://").unwrap(),
                        Some(Message::ClickEndpoint(el.id)),
                        ButtonType::Text
                    )
                    .width(256),
                    bi(
                        Icons::Delete,
                        Some(Message::ClickDeleteEndpoint(el.id)),
                        ButtonType::Danger
                    )
                    .width(52)
                ]
                .spacing(8)
                .into()
            }))
            .spacing(8)
        ]
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
    let current = state
        .endpoints
        .iter()
        .find(|it| Some(it.id) == state.selected_endpoint);
    match current {
        Some(result) => {
            let query_params = Column::from_iter(result.query_params.iter().map(|it| {
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
            }))
            .spacing(8);
            let urlbar = card_clickable(
                row![
                    text(format_url_from_state(state)),
                    horizontal_space(),
                    svg(svg::Handle::from_memory(match_icon(Icons::Duplicate)))
                        .width(20)
                        .height(20)
                ]
                .align_y(Center),
                Some(Message::Duplicate(result.url.clone())),
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
                row![
                    container(
                        column![
                            row![
                                text("Query params"),
                                horizontal_space(),
                                bt("Add", Some(Message::AddQueryParam()), ButtonType::Primary)
                            ]
                            .padding([0, 8])
                            .align_y(Center),
                            query_params
                        ]
                        .spacing(16)
                    )
                    .style(|t| container::Style {
                        border: Border::default().rounded(16),
                        background: Some(iced::Background::Color(t.palette().background)),
                        ..container::Style::default()
                    }),
                    response_panels(state)
                ]
            ]
        }
        None => column![draft_urlbar(state)],
    }
}

fn format_url_from_state(state: &State) -> String {
    let current = state
        .endpoints
        .iter()
        .find(|it| Some(it.id) == state.selected_endpoint);
    match current {
        Some(endpoint) => {
            let query_param_vec: Vec<String> = endpoint
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
            return format!(
                "{}{}{}",
                endpoint.url,
                if query_param_vec.is_empty() { "" } else { "?" },
                query_param_vec.join("&")
            );
        }
        None => {
            return state.draft.clone();
        }
    }
}

fn response_panels(state: &State) -> Element<Message> {
    let current = state
        .endpoints
        .iter()
        .find(|it| Some(it.id) == state.selected_endpoint);
    match current {
        Some(e) => {
            let current_response = e.responses.get(state.selected_response_index);
            match current_response {
                Some(resp) => {
                    let time = Local::now().offset().from_utc_datetime(&resp.received_time);
                    column![
                        mb(
                            ml(
                                row![
                                    if state.selected_response_index > 0 {
                                        bi(
                                            Icons::Left,
                                            Some(Message::SetSelectedResponseIndex(
                                                state.selected_response_index - 1,
                                            )),
                                            ButtonType::Outlined,
                                        )
                                    } else {
                                        empty_b()
                                    }
                                    .width(32)
                                    .height(32),
                                    text(state.selected_response_index + 1)
                                        .width(Fill)
                                        .align_x(Center),
                                    if state.selected_response_index + 1 < e.responses.len() {
                                        bi(
                                            Icons::Right,
                                            Some(Message::SetSelectedResponseIndex(
                                                state.selected_response_index + 1,
                                            )),
                                            ButtonType::Outlined,
                                        )
                                    } else {
                                        empty_b()
                                    }
                                    .width(32)
                                    .height(32),
                                ]
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
