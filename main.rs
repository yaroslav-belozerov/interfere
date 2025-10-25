mod lib;

use iced::theme::Palette;
use iced::widget::{
    column, container, horizontal_space, row, scrollable, text, Column, Container, Row,
};
use iced::Alignment::Center;
use iced::Length::{Fill, FillPortion, Shrink};
use iced::{border, keyboard, Color, Element, Padding, Subscription, Task, Theme};
use lib::common::*;
use lib::ui::*;
use reqwest::StatusCode;

pub fn main() -> iced::Result {
    iced::application("Interfere", update, view)
        .subscription(subscription)
        .theme(theme)
        .run()
}

#[derive(Default)]
struct Response {
    text: String,
    code: StatusCode,
}

struct State {
    url: String,
    can_send: bool,
    response: Option<Result<Response, MyErr>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            url: Default::default(),
            response: None,
            can_send: true,
        }
    }
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::SetUrl(string) => {
            state.url = string;
            Task::none()
        }
        Message::Send => {
            state.can_send = false;
            Task::perform(send_get_request(state.url.clone()), |res| match res {
                Ok((text, code)) => Message::GotResponse(text, code),
                Err(err) => Message::GotError(err),
            })
        }
        Message::GotResponse(text, code) => {
            state.response = Some(Ok(Response { text, code }));
            state.can_send = true;
            Task::none()
        }
        Message::GotError(err) => {
            state.can_send = true;
            state.response = Some(Err(err));
            Task::none()
        }
        Message::Empty => Task::none(),
    }
}

fn view(state: &State) -> Element<Message> {
    column![header(state), content(state)].padding(16).into()
}

fn header(state: &State) -> Container<Message> {
    let urlbar = mytext_input("Input URL...", &state.url, &Message::SetUrl, Message::Send);
    let button = mybutton(
        "Send",
        if state.can_send {
            Some(Message::Send)
        } else {
            None
        },
    )
    .height(Shrink)
    .padding([8, 16]);
    mb(
        column![row![urlbar, button].spacing(16).align_y(Center)].into(),
        16.0,
    )
}

fn content(state: &State) -> Row<Message> {
    row![
        card(column![row![
            text("Query params"),
            horizontal_space(),
            mybutton("Add", None)
        ]
        .align_y(Center)]),
        match &state.response {
            Some(result) => {
                ml(
                    card(match result {
                        Ok(response) => {
                            column![
                                container(text!("{}", response.code).style(|_| text::Style {
                                    color: Some(Color::BLACK)
                                }))
                                .padding([2, 4])
                                .style(|_| container::Style {
                                    background: Some(iced::Background::Color(color_for_status(
                                        response.code
                                    ))),
                                    ..container::Style::default()
                                }),
                                mb(
                                    scrollable(row![text(&response.text)].padding([16, 0]))
                                        .height(Fill)
                                        .width(Fill)
                                        .into(),
                                    16.0
                                )
                            ]
                        }
                        Err(err) => {
                            column![text!("{}", err.to_string())]
                        }
                    })
                    .width(Fill)
                    .into(),
                    16.0,
                )
                .width(Fill)
            }
            None => container(column![]),
        }
        .width(Fill)
    ]
}

impl From<reqwest::Error> for MyErr {
    fn from(err: reqwest::Error) -> Self {
        Self::Unknown(err.to_string())
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
        _ => None,
    })
}

fn theme(_state: &State) -> Theme {
    Theme::custom(
        "Abobolik".into(),
        Palette {
            background: Color::parse("#1A1B26").unwrap(),
            danger: Color::parse("#F7768E").unwrap(),
            primary: Color::parse("#BB9AF7").unwrap(),
            success: Color::parse("#A6CD70").unwrap(),
            text: Color::parse("#C0CAF5").unwrap(),
        },
    )
}

fn color_for_status(code: StatusCode) -> Color {
    match code {
        StatusCode::OK => Color::parse("#9ECE6A"),
        StatusCode::FOUND => Color::parse("#414868"),
        _ => Color::parse("#F7768E"),
    }
    .unwrap()
}
