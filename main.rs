mod logic;

use std::cmp::max;
use std::time::{SystemTime, SystemTimeError};

use crate::AppTheme;
use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, TimeZone};
use iced::font::Weight;
use iced::theme::Palette;
use iced::widget::scrollable::Scrollbar;
use iced::widget::text_input::{self, focus};
use iced::widget::{
    column, container, horizontal_space, row, scrollable, svg, text, Button, Column,
};
use iced::Alignment::Center;
use iced::Length::{Fill, Shrink};
use iced::{keyboard, Color, Element, Font, Subscription, Task, Theme};
use logic::common::*;
use logic::crud::endpoint::{create_endpoint_full, delete_endpoint, update_endpoint_url};
use logic::crud::response::{create_response, delete_response, responses_by_endpoint_id};
use logic::db::{get_db, init, load_endpoints};
use logic::ui::*;
use reqwest::StatusCode;

impl State {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                draft: Default::default(),
                endpoints: load_endpoints(&get_db().lock().unwrap()).unwrap(),
                can_send: true,
                selected_endpoint: None,
                selected_response_index: 0,
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
                send_get_request(match state.selected_endpoint {
                    Some(id) => state
                        .endpoints
                        .iter()
                        .find(|it| it.id == id)
                        .unwrap()
                        .url
                        .clone(),
                    None => state.draft.clone(),
                }),
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
                    create_endpoint_full(
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
                }
            }
            state.draft = "".to_string();
            update(state, Message::RefetchDb)
        }
        Message::GotError(_err) => {
            state.can_send = true;
            Task::none()
        }
        Message::ClickEndpoint(id) => {
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
                Some(endpoint_id) => {
                    match state.endpoints.iter().find(|&it| it.id == endpoint_id) {
                        Some(endpoint) => {
                            if state.selected_response_index + 1 > endpoint.responses.iter().len() {
                                state.selected_response_index =
                                    max(state.selected_response_index, 1) - 1;
                            }
                        }
                        None => return Task::none(),
                    }
                }
                None => return Task::none(),
            }
            update(state, Message::RefetchDb)
        }
        Message::ClickDeleteResponse(id) => {
            delete_response(&get_db().lock().unwrap(), id).unwrap();
            update(state, Message::RefetchDb)
        }
        Message::Empty => Task::none(),
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
                    icon_outlined_b(Icons::Plus, Some(Message::Back))
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
                    text_b(
                        text(el.url.strip_prefix("https://").unwrap()),
                        Some(Message::ClickEndpoint(el.id))
                    )
                    .width(256),
                    danger_b("Del", Some(Message::ClickDeleteEndpoint(el.id))).width(52)
                ]
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
    primary_b(
        "Send",
        if state.can_send {
            Some(Message::Send)
        } else {
            None
        },
        Some(Icons::Enter),
    )
    .height(Shrink)
    .padding([14, 16])
}
fn draft_urlbar(state: &State) -> Element<Message> {
    let urlbar = mytext_input(
        "Input URL...",
        &state.draft,
        &Message::SetDraft,
        Message::Send,
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
            let urlbar = text_b(
                row![
                    text(&result.url),
                    horizontal_space(),
                    svg(svg::Handle::from_memory(match_icon(Icons::Duplicate)))
                        .width(20)
                        .height(20)
                ]
                .align_y(Center),
                Some(Message::Duplicate(result.url.clone())),
            )
            .width(Fill)
            .padding(16);
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
                    card(column![row![
                        text("Query params"),
                        horizontal_space(),
                        primary_b("Add", None, None)
                    ]
                    .align_y(Center)]),
                    response_panels(state)
                ]
            ]
        }
        None => column![draft_urlbar(state)],
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
            column![
                mb(
                    ml(
                        row![
                            if state.selected_response_index > 0 {
                                icon_outlined_b(
                                    Icons::Left,
                                    Some(Message::SetSelectedResponseIndex(
                                        state.selected_response_index - 1,
                                    )),
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
                                icon_outlined_b(
                                    Icons::Right,
                                    Some(Message::SetSelectedResponseIndex(
                                        state.selected_response_index + 1,
                                    )),
                                )
                            } else {
                                empty_b()
                            }
                            .width(32)
                            .height(32),
                        ]
                        .into(),
                        16.0
                    )
                    .into(),
                    16.0
                ),
                match current_response {
                    Some(resp) => {
                        let time = Local::now().offset().from_utc_datetime(&resp.received_time);
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
                                    icon_outlined_b(
                                        Icons::Delete,
                                        Some(Message::ClickDeleteResponse(resp.id))
                                    )
                                ]
                                .align_y(Center)
                                .spacing(8),
                                mb(
                                    scrollable(text(&resp.text))
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
                    }
                    None => container(column![]),
                }
                .width(Fill)
            ]
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
