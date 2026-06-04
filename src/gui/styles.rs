// Styles Module
// Custom styles with rounded corners for UI components

use iced::widget::{button, container, text_input, progress_bar};
use iced::{Background, Border, Color, Theme};

/// Rounded corner radius
const RADIUS: f32 = 12.0;
const RADIUS_SMALL: f32 = 8.0;
const RADIUS_LARGE: f32 = 16.0;

/// Primary button style (blue/accent)
pub fn button_primary(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let base = button::Style {
        background: Some(Background::Color(palette.primary.base.color)),
        text_color: Color::WHITE,
        border: Border {
            radius: RADIUS.into(),
            ..Default::default()
        },
        ..Default::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(palette.primary.strong.color)),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(palette.primary.strong.color)),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.5))),
            text_color: Color::from_rgba(0.7, 0.7, 0.7, 0.8),
            ..base
        },
        _ => base,
    }
}

/// Secondary button style (gray)
pub fn button_secondary(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let base = button::Style {
        background: Some(Background::Color(palette.secondary.base.color)),
        text_color: palette.secondary.base.text,
        border: Border {
            radius: RADIUS.into(),
            ..Default::default()
        },
        ..Default::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(palette.secondary.strong.color)),
            ..base
        },
        _ => base,
    }
}

/// Success button style (green)
pub fn button_success(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::from_rgb(0.2, 0.8, 0.4))),
        text_color: Color::WHITE,
        border: Border {
            radius: RADIUS.into(),
            ..Default::default()
        },
        ..Default::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.25, 0.85, 0.45))),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.3, 0.5, 0.3, 0.5))),
            text_color: Color::from_rgba(0.7, 0.7, 0.7, 0.8),
            ..base
        },
        _ => base,
    }
}

/// Danger button style (red)
pub fn button_danger(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::from_rgb(0.9, 0.3, 0.3))),
        text_color: Color::WHITE,
        border: Border {
            radius: RADIUS.into(),
            ..Default::default()
        },
        ..Default::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.95, 0.35, 0.35))),
            ..base
        },
        _ => base,
    }
}

/// Outline button style (bordered)
pub fn button_outline(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: palette.primary.base.color,
        border: Border {
            radius: RADIUS.into(),
            width: 2.0,
            color: palette.primary.base.color,
        },
        ..Default::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.2, 0.5, 0.8, 0.1))),
            ..base
        },
        _ => base,
    }
}

/// Card container style
pub fn card_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.12, 0.12, 0.18, 0.95))),
        text_color: Some(Color::WHITE),
        border: Border {
            radius: RADIUS_LARGE.into(),
            width: 1.0,
            color: Color::from_rgba(0.25, 0.25, 0.35, 0.6),
        },
        ..Default::default()
    }
}

/// Panel container style
pub fn panel_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.15, 0.9))),
        text_color: Some(Color::WHITE),
        border: Border {
            radius: RADIUS.into(),
            width: 1.0,
            color: Color::from_rgba(0.2, 0.2, 0.3, 0.5),
        },
        ..Default::default()
    }
}

/// Input field style
pub fn text_input_style(theme: &Theme, _status: text_input::Status) -> text_input::Style {
    let palette = theme.extended_palette();
    text_input::Style {
        background: Background::Color(Color::from_rgba(0.1, 0.1, 0.15, 0.8)),
        border: Border {
            radius: RADIUS.into(),
            width: 1.5,
            color: Color::from_rgba(0.4, 0.4, 0.5, 0.6),
        },
        icon: Color::from_rgba(0.6, 0.6, 0.7, 0.8),
        placeholder: Color::from_rgba(0.5, 0.5, 0.6, 0.8),
        value: Color::WHITE,
        selection: palette.primary.base.color,
    }
}

/// Progress bar style
pub fn progress_bar_style(_theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: Background::Color(Color::from_rgba(0.2, 0.2, 0.25, 0.8)),
        bar: Background::Color(Color::from_rgb(0.3, 0.7, 0.9)),
        border: Border {
            radius: RADIUS_SMALL.into(),
            ..Default::default()
        },
    }
}

/// Checkbox style
pub fn checkbox_style(theme: &Theme, _status: iced::widget::checkbox::Status) -> iced::widget::checkbox::Style {
    let _palette = theme.extended_palette();
    iced::widget::checkbox::Style {
        background: Background::Color(Color::from_rgba(0.15, 0.15, 0.2, 0.8)),
        border: Border {
            radius: RADIUS_SMALL.into(),
            width: 1.5,
            color: Color::from_rgba(0.4, 0.4, 0.5, 0.6),
        },
        text_color: Some(Color::WHITE),
        icon_color: Color::WHITE,
    }
}

/// Radio button style
pub fn radio_style(theme: &Theme, _status: iced::widget::radio::Status) -> iced::widget::radio::Style {
    let palette = theme.extended_palette();
    iced::widget::radio::Style {
        background: Background::Color(Color::from_rgba(0.15, 0.15, 0.2, 0.8)),
        dot_color: palette.primary.base.color,
        border_width: 1.5,
        border_color: Color::from_rgba(0.4, 0.4, 0.5, 0.6),
        text_color: Some(Color::WHITE),
    }
}

/// Icon button style (transparent until hovered)
pub fn button_icon(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: Color::WHITE,
        border: Border {
            radius: RADIUS.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..Default::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.1))),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.15))),
            ..base
        },
        _ => base,
    }
}
