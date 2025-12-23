use crate::logic::common::Message;
use iced::Alignment::Center;
use iced::Length::Shrink;
use iced::advanced::text::{Paragraph, paragraph};
use iced::border::Radius;
use iced::theme::Palette;
use iced::widget::button::Status;
use iced::widget::text::IntoFragment;
use iced::widget::{
    Button, Column, Container, Row, TextInput, button, container, row, svg, text, text_input,
};
use iced::{Background, Border, Color, Element, Font, Padding, Renderer, Shadow, Theme, border};

pub const GEIST_FONT: Font = Font {
    family: iced::font::Family::Name("Geist"),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

pub enum Icons {
    Enter,
    Plus,
    Escape,
    Duplicate,
    Delete,
    Left,
    Right,
    Check,
    Close,
    Format,
}

pub enum ButtonType {
    Primary,
    Text,
    Danger,
    Outlined,
    PrimaryInline,
    Inline,
    OutlinedInline,
}

pub struct AppTheme {
    pub palette: Palette,
}

pub fn match_icon(ic: Icons) -> Vec<u8> {
    match ic {
        Icons::Enter => include_bytes!("../res/icons/enter.svg").to_vec(),
        Icons::Plus => include_bytes!("../res/icons/add.svg").to_vec(),
        Icons::Escape => include_bytes!("../res/icons/escape.svg").to_vec(),
        Icons::Duplicate => include_bytes!("../res/icons/copy.svg").to_vec(),
        Icons::Delete => include_bytes!("../res/icons/delete.svg").to_vec(),
        Icons::Left => include_bytes!("../res/icons/arrow-left.svg").to_vec(),
        Icons::Right => include_bytes!("../res/icons/arrow-right.svg").to_vec(),
        Icons::Check => include_bytes!("../res/icons/check.svg").to_vec(),
        Icons::Close => include_bytes!("../res/icons/close.svg").to_vec(),
        Icons::Format => include_bytes!("../res/icons/format.svg").to_vec(),
    }
}

fn primary_b<'a>(
    string: impl IntoFragment<'a>,
    on_click: Option<Message>,
    icon_path: Option<Icons>,
) -> Button<'a, Message> {
    match icon_path {
        Some(p) => {
            let handle = svg::Handle::from_memory(match_icon(p));
            button(
                row![
                    svg(handle)
                        .style(|_, _| {
                            svg::Style {
                                color: Some(Color::BLACK),
                            }
                        })
                        .width(20)
                        .height(20),
                    text(string)
                ]
                .align_y(Center)
                .spacing(8)
                .width(Shrink),
            )
            .on_press_maybe(on_click)
            .style(primary_button_style)
        }
        None => button(text(string))
            .on_press_maybe(on_click)
            .style(primary_button_style),
    }
}

fn primary_inline_b<'a>(
    string: impl IntoFragment<'a>,
    on_click: Option<Message>,
    icon_path: Option<Icons>,
) -> Button<'a, Message> {
    match icon_path {
        Some(p) => {
            let handle = svg::Handle::from_memory(match_icon(p));
            button(
                row![
                    svg(handle)
                        .style(|_, _| {
                            svg::Style {
                                color: Some(Color::BLACK),
                            }
                        })
                        .width(20)
                        .height(20),
                    text(string)
                ]
                .align_y(Center)
                .spacing(8)
                .width(Shrink),
            )
            .padding([6, 8])
            .on_press_maybe(on_click)
            .style(primary_inline_button_style)
        }
        None => button(text(string))
            .padding([6, 8])
            .on_press_maybe(on_click)
            .style(primary_inline_button_style),
    }
}

fn danger_b<'a>(
    string: impl IntoFragment<'a>,
    on_click: Option<Message>,
    icon_path: Option<Icons>,
) -> Button<'a, Message> {
    match icon_path {
        Some(p) => {
            let handle = svg::Handle::from_memory(match_icon(p));
            button(
                row![
                    svg(handle)
                        .style(|_, _| {
                            svg::Style {
                                color: Some(Color::BLACK),
                            }
                        })
                        .width(20)
                        .height(20),
                    text(string)
                ]
                .align_y(Center)
                .spacing(8)
                .width(Shrink),
            )
            .on_press_maybe(on_click)
            .style(danger_button_style)
        }
        None => button(text(string))
            .on_press_maybe(on_click)
            .style(danger_button_style),
    }
}
fn inline_b<'a>(content: impl IntoFragment<'a>, on_click: Option<Message>) -> Button<'a, Message> {
    button(text(content))
        .padding(Padding::from([6, 8]))
        .on_press_maybe(on_click)
        .style(|theme, status| button::Style {
            background: Some(iced::Background::Color(match status {
                button::Status::Hovered => Color::parse("#242530").unwrap(),
                _ => Color::TRANSPARENT,
            })),
            ..button::text(theme, status)
        })
}

fn text_b<'a>(content: impl IntoFragment<'a>, on_click: Option<Message>) -> Button<'a, Message> {
    button(text(content))
        .on_press_maybe(on_click)
        .style(|theme, status| button::Style {
            border: border::rounded(0),
            background: Some(iced::Background::Color(match status {
                button::Status::Hovered => Color::parse("#32333d").unwrap(),
                _ => Color::parse("#242530").unwrap(),
            })),
            ..button::text(theme, status)
        })
}
fn outlined_b<'a>(string: impl IntoFragment<'a>, on_click: Option<Message>) -> Button<'a, Message> {
    button(text(string).align_x(Center))
        .on_press_maybe(on_click)
        .style(|t, s| button::Style {
            border: Border {
                color: Color::parse("#C0CAF5").unwrap(),
                radius: Radius::new(0),
                width: 1.0,
            },
            background: match s {
                button::Status::Hovered => {
                    Some(Background::Color(Color::parse("#32333D").unwrap()))
                }
                _ => None,
            },
            text_color: button::text(t, Status::Active).text_color,
            ..button::text(t, s)
        })
}

pub fn bt<'a>(
    content: impl IntoFragment<'a>,
    on_click: Option<Message>,
    button_type: ButtonType,
) -> Button<'a, Message> {
    match button_type {
        ButtonType::Primary => primary_b(content, on_click, None),
        ButtonType::Text => text_b(content, on_click),
        ButtonType::Danger => danger_b(content, on_click, None),
        ButtonType::Outlined => outlined_b(content, on_click),
        ButtonType::PrimaryInline => primary_inline_b(content, on_click, None),
        ButtonType::Inline => inline_b(content, on_click),
        ButtonType::OutlinedInline => todo!(),
    }
}

pub fn bi<'a>(
    icon: Icons,
    on_click: Option<Message>,
    button_type: ButtonType,
) -> Button<'a, Message> {
    match button_type {
        ButtonType::Primary => icon_primary_b(icon, on_click),
        ButtonType::Text => icon_text_b(icon, on_click),
        ButtonType::Danger => icon_danger_b(icon, on_click),
        ButtonType::Outlined => icon_outlined_b(icon, on_click),
        ButtonType::PrimaryInline => icon_primary_inline_b(icon, on_click),
        ButtonType::Inline => icon_inline_b(icon, on_click),
        ButtonType::OutlinedInline => icon_outlined_inline_b(icon, on_click),
    }
}

pub fn bti<'a>(
    content: String,
    icon: Icons,
    on_click: Option<Message>,
    button_type: ButtonType,
) -> Button<'a, Message> {
    match button_type {
        ButtonType::Primary => primary_b(content, on_click, Some(icon)),
        ButtonType::Text => todo!(),
        ButtonType::Danger => danger_b(content, on_click, Some(icon)),
        ButtonType::Outlined => todo!(),
        ButtonType::PrimaryInline => todo!(),
        ButtonType::Inline => todo!(),
        ButtonType::OutlinedInline => todo!(),
    }
}

fn icon_danger_b<'a>(icon: Icons, on_click: Option<Message>) -> Button<'a, Message> {
    button(
        svg(svg::Handle::from_memory(match_icon(icon)))
            .style(|t, _| svg::Style {
                color: Some(button::danger(t, Status::Active).text_color),
            })
            .width(20)
            .height(20),
    )
    .on_press_maybe(on_click)
    .style(|t, s| button::Style {
        border: Border {
            color: Color::TRANSPARENT,
            radius: Radius::new(32),
            width: 1.0,
        },
        ..button::danger(t, s)
    })
    .padding(6)
}

fn icon_inline_b<'a>(icon: Icons, on_click: Option<Message>) -> Button<'a, Message> {
    button(
        svg(svg::Handle::from_memory(match_icon(icon)))
            .style(|t, _| svg::Style {
                color: Some(button::text(t, Status::Active).text_color),
            })
            .width(20)
            .height(20),
    )
    .on_press_maybe(on_click)
    .style(|t, s| button::Style {
        background: match s {
            button::Status::Hovered => Some(Background::Color(Color::parse("#32333D").unwrap())),
            _ => None,
        },
        text_color: button::text(t, Status::Active).text_color,
        ..button::text(t, s)
    })
    .padding(7)
}

fn icon_text_b<'a>(icon: Icons, on_click: Option<Message>) -> Button<'a, Message> {
    button(
        svg(svg::Handle::from_memory(match_icon(icon)))
            .style(|t, _| svg::Style {
                color: Some(button::text(t, Status::Active).text_color),
            })
            .width(20)
            .height(20),
    )
    .on_press_maybe(on_click)
    .style(|t, s| button::Style {
        border: Border::default().rounded(32),
        background: match s {
            button::Status::Hovered => Some(Background::Color(Color::parse("#32333D").unwrap())),
            _ => None,
        },
        text_color: button::text(t, Status::Active).text_color,
        ..button::text(t, s)
    })
    .padding(6)
}

fn icon_primary_b<'a>(icon: Icons, on_click: Option<Message>) -> Button<'a, Message> {
    button(
        svg(svg::Handle::from_memory(match_icon(icon)))
            .style(|t, _| svg::Style {
                color: Some(button::primary(t, Status::Active).text_color),
            })
            .width(20)
            .height(20),
    )
    .on_press_maybe(on_click)
    .style(|t, s| button::Style {
        border: Border::default().rounded(32),
        ..button::primary(t, s)
    })
    .padding(6)
}

fn icon_primary_inline_b<'a>(icon: Icons, on_click: Option<Message>) -> Button<'a, Message> {
    button(
        svg(svg::Handle::from_memory(match_icon(icon)))
            .style(|t, _| svg::Style {
                color: Some(button::primary(t, Status::Active).text_color),
            })
            .width(16)
            .height(16),
    )
    .on_press_maybe(on_click)
    .style(|t, s| button::Style {
        border: Border::default().rounded(0),
        ..button::primary(t, s)
    })
    .padding(6)
}

fn icon_outlined_inline_b<'a>(icon: Icons, on_click: Option<Message>) -> Button<'a, Message> {
    button(
        svg(svg::Handle::from_memory(match_icon(icon)))
            .style(|t, _| svg::Style {
                color: Some(button::text(t, Status::Active).text_color),
            })
            .width(16)
            .height(16),
    )
    .on_press_maybe(on_click)
    .style(|t, s| button::Style {
        border: Border {
            color: Color::parse("#C0CAF5").unwrap(),
            radius: Radius::new(0),
            width: 1.0,
        },
        background: match s {
            button::Status::Hovered => Some(Background::Color(Color::parse("#32333D").unwrap())),
            _ => None,
        },
        text_color: button::text(t, Status::Active).text_color,
        ..button::text(t, s)
    })
    .padding(6)
}

fn icon_outlined_b<'a>(icon: Icons, on_click: Option<Message>) -> Button<'a, Message> {
    button(
        svg(svg::Handle::from_memory(match_icon(icon)))
            .style(|t, _| svg::Style {
                color: Some(button::text(t, Status::Active).text_color),
            })
            .width(20)
            .height(20),
    )
    .on_press_maybe(on_click)
    .style(|t, s| button::Style {
        border: Border {
            color: Color::parse("#C0CAF5").unwrap(),
            radius: Radius::new(32),
            width: 1.0,
        },
        background: match s {
            button::Status::Hovered => Some(Background::Color(Color::parse("#32333D").unwrap())),
            _ => None,
        },
        text_color: button::text(t, Status::Active).text_color,
        ..button::text(t, s)
    })
    .padding(6)
}

pub fn empty_b() -> Button<'static, Message> {
    button("")
        .style(|_, _| button::Style {
            background: None,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::new(0),
            },
            shadow: Shadow::default(),
            text_color: Color::TRANSPARENT,
        })
        .width(0)
        .height(0)
}

pub fn mytext_input(
    placeholder: &str,
    value: &str,
    input_on_click: impl Fn(String) -> Message + 'static,
    submit_on_click: Option<Message>,
) -> TextInput<'static, Message> {
    text_input(placeholder, value)
        .style(custom_text_input_style)
        .padding(16)
        .on_input(input_on_click)
        .on_submit_maybe(submit_on_click)
}

fn custom_text_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: iced::Background::Color(Color::parse("#242530").unwrap()),
        border: Border {
            color: Color::WHITE,
            radius: Radius::new(0),
            width: match status {
                text_input::Status::Focused => 1.0,
                _ => 0.0,
            },
        },
        ..text_input::default(theme, status)
    }
}

fn primary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        border: border::rounded(0),
        ..button::primary(theme, status)
    }
}

fn primary_inline_button_style(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        ..button::primary(theme, status)
    }
}

fn danger_button_style(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        border: border::rounded(0),
        ..button::danger(theme, status)
    }
}

pub trait Paddable<'a> {
    fn apply_padding<P: Into<Padding>>(self, padding: P) -> Element<'a, Message, Theme, Renderer>;
}

macro_rules! impl_paddable {
    ($type:ty) => {
        impl<'a> Paddable<'a> for $type {
            fn apply_padding<P: Into<Padding>>(
                self,
                padding: P,
            ) -> Element<'a, Message, Theme, Renderer> {
                self.padding(padding).into()
            }
        }
    };
}

impl_paddable!(Column<'a, Message, Theme, Renderer>);
impl_paddable!(Row<'a, Message, Theme, Renderer>);
impl_paddable!(Container<'a, Message, Theme, Renderer>);

pub fn card<'a, T: Paddable<'a>>(content: T) -> Container<'a, Message> {
    container(
        container(content.apply_padding(16)).style(|_| container::Style {
            border: border::rounded(0),
            background: Some(iced::Background::Color(Color::parse("#242530").unwrap())),
            ..container::Style::default()
        }),
    )
}

pub fn card_clickable<'a, T: Paddable<'a>>(
    content: T,
    on_click: Option<Message>,
) -> Container<'a, Message> {
    container(
        button(content.apply_padding(0))
            .on_press_maybe(on_click)
            .padding(16)
            .style(|t, s| button::Style {
                border: border::rounded(0),
                background: match s {
                    button::Status::Hovered => {
                        Some(Background::Color(Color::parse("#32333D").unwrap()))
                    }
                    _ => Some(iced::Background::Color(Color::parse("#242530").unwrap())),
                },
                ..button::text(t, s)
            }),
    )
}

pub fn ml<'a>(content: Element<'a, Message>, margin_left: f32) -> Container<'a, Message> {
    container(content).padding(Padding {
        top: 0.0,
        left: margin_left,
        right: 0.0,
        bottom: 0.0,
    })
}

pub fn mr<'a>(content: Element<'a, Message>, margin_right: f32) -> Container<'a, Message> {
    container(content).padding(Padding {
        top: 0.0,
        left: 0.0,
        right: margin_right,
        bottom: 0.0,
    })
}

pub fn mb<'a>(content: Element<'a, Message>, margin_bottom: f32) -> Container<'a, Message> {
    container(content).padding(Padding {
        top: 0.0,
        left: 0.0,
        right: 0.0,
        bottom: margin_bottom,
    })
}
