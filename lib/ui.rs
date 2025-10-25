use crate::lib::common::Message;
use iced::border::Radius;
use iced::widget::text::IntoFragment;
use iced::widget::{button, container, text, text_input, Button, Column, Container, TextInput};
use iced::{border, Border, Color, Element, Padding, Renderer, Theme};

pub fn mybutton<'a>(string: impl IntoFragment<'a>, msg: Option<Message>) -> Button<'a, Message> {
    button(text(string))
        .on_press_maybe(msg)
        .style(custom_button_style)
}

pub fn mytext_input<'a>(
    placeholder: &str,
    value: &str,
    input_msg: &'a impl Fn(String) -> Message,
    submit_msg: Message,
) -> TextInput<'a, Message> {
    text_input(placeholder, value)
        .style(custom_text_input_style)
        .padding([12, 12])
        .on_input(input_msg)
        .on_submit(submit_msg)
}

pub fn custom_text_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: iced::Background::Color(Color::parse("#242530").unwrap()),
        border: Border {
            color: Color::WHITE,
            radius: Radius::new(8),
            width: match status {
                text_input::Status::Focused => 1.0,
                _ => 0.0,
            },
        },
        ..text_input::default(theme, status)
    }
}

pub fn custom_button_style(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        border: border::rounded(8),
        ..button::primary(theme, status)
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

pub fn card<'a, T: Paddable<'a>>(content: T) -> Container<'a, Message> {
    container(
        container(content.apply_padding(16)).style(|_| container::Style {
            border: border::rounded(8),
            background: Some(iced::Background::Color(Color::parse("#242530").unwrap())),
            ..container::Style::default()
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
