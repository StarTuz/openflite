use iced::widget::container;
use iced::{Color, Theme};

// ============ Color Palette ============
// Based on a modern dark theme with cyan/teal accents

pub const BACKGROUND_DARK: Color = Color::from_rgb(0.04, 0.04, 0.06);
pub const BACKGROUND_CARD: Color = Color::from_rgb(0.07, 0.08, 0.10);
pub const BACKGROUND_ELEVATED: Color = Color::from_rgb(0.10, 0.11, 0.14);

pub const BORDER_SUBTLE: Color = Color::from_rgb(0.15, 0.17, 0.22);
pub const BORDER_ACCENT: Color = Color::from_rgb(0.0, 0.6, 0.8);

pub const TEXT_PRIMARY: Color = Color::from_rgb(0.92, 0.93, 0.95);
pub const TEXT_SECONDARY: Color = Color::from_rgb(0.55, 0.58, 0.65);
pub const TEXT_MUTED: Color = Color::from_rgb(0.38, 0.40, 0.45);

pub const ACCENT_CYAN: Color = Color::from_rgb(0.0, 0.85, 1.0);
pub const ACCENT_GREEN: Color = Color::from_rgb(0.2, 0.9, 0.5);
pub const ACCENT_ORANGE: Color = Color::from_rgb(1.0, 0.65, 0.2);
pub const ACCENT_RED: Color = Color::from_rgb(0.95, 0.3, 0.35);

pub const STATUS_CONNECTED: Color = Color::from_rgb(0.2, 0.95, 0.6);
pub const STATUS_DISCONNECTED: Color = Color::from_rgb(0.95, 0.35, 0.35);
pub const STATUS_PENDING: Color = Color::from_rgb(1.0, 0.8, 0.2);

// ============ Container Styles ============

pub fn header_style(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(BACKGROUND_DARK)),
        border: iced::Border {
            color: BORDER_SUBTLE,
            width: 0.0,
            radius: 0.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..Default::default()
    }
}

pub fn footer_style(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.05, 0.05))),
        border: iced::Border {
            color: ACCENT_RED,
            width: 0.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub fn card_style(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(BACKGROUND_CARD)),
        border: iced::Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        ..Default::default()
    }
}

pub fn card_elevated_style(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(BACKGROUND_ELEVATED)),
        border: iced::Border {
            color: BORDER_ACCENT,
            width: 1.0,
            radius: 10.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.6, 0.8, 0.15),
            offset: iced::Vector::new(0.0, 0.0),
            blur_radius: 20.0,
        },
        ..Default::default()
    }
}

pub fn section_header_style(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.0, 0.0, 0.0, 0.0,
        ))),
        border: iced::Border {
            color: BORDER_SUBTLE,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn status_badge_connected(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.2, 0.95, 0.6, 0.15,
        ))),
        border: iced::Border {
            color: STATUS_CONNECTED,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub fn status_badge_disconnected(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.95, 0.35, 0.35, 0.1,
        ))),
        border: iced::Border {
            color: STATUS_DISCONNECTED,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}
