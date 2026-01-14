use iced::widget::container;
use iced::{Color, Theme};

pub fn header_style(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb(0.05, 0.05, 0.07))),
        border: iced::Border {
            color: Color::from_rgb(0.1, 0.1, 0.15),
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn footer_style(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.05, 0.05))),
        ..Default::default()
    }
}

pub fn card_style(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.1))),
        border: iced::Border {
            color: Color::from_rgb(0.15, 0.15, 0.2),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}
