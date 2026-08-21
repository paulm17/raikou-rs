//! Bridges raikou Fluent themes onto fyrox's global [`Style`].
//!
//! Many fyrox widgets (text boxes, decorators, dropdown lists, toggle
//! buttons, ...) fall back to semantic global brushes such as
//! [`Style::BRUSH_DARKER`] when they are not styled explicitly. Out of the box
//! those brushes form a dark palette, which makes unstyled widgets look wrong
//! in a Fluent-themed application. This module maps every global brush onto
//! the equivalent Fluent token so fallback styling matches the theme.

use crate::theme::Theme;
use fyrox::core::color::Color as FyroxColor;
use fyrox::gui::brush::Brush;
use fyrox::gui::button::Button;
use fyrox::gui::dropdown_list::DropdownList;
use fyrox::gui::style::{resource::StyleResource, Style};
use fyrox::gui::toggle::ToggleButton;
use fyrox::gui::Thickness;

fn color(theme: &Theme, name: &str) -> FyroxColor {
    let c = theme.color(name).unwrap_or(raikou_core::Color::new(1.0, 1.0, 1.0, 1.0));
    FyroxColor::from_rgba(
        (c.red * 255.0).round() as u8,
        (c.green * 255.0).round() as u8,
        (c.blue * 255.0).round() as u8,
        (c.alpha * 255.0).round() as u8,
    )
}

fn solid(theme: &Theme, light: &str, dark: &str, dark_mode: bool) -> Brush {
    Brush::Solid(color(theme, if dark_mode { dark } else { light }))
}

/// Builds a fyrox [`Style`] whose global brushes are mapped onto the given
/// raikou theme (normally `Theme::fluent_light()` / `fluent_dark()`).
pub fn fluent_fyrox_style(theme: &Theme, dark: bool) -> Style {
    // Start from the equivalent of fyrox's stock `base_style()` (global font
    // size + per-widget metric styles) so widgets keep sane defaults, then
    // override every global brush with Fluent tokens.
    let mut style = Style::default();
    style
        .set(Style::FONT_SIZE, 14.0f32)
        .merge(&Button::style())
        .merge(&fyrox::gui::check_box::CheckBox::style())
        .merge(&DropdownList::style())
        .merge(&ToggleButton::style());

    // Neutral surface ramp. `DARKER` is the workhorse: it is the default
    // background of text boxes, decorators and dropdown lists. The LIGHT
    // family is the default decorator fill ramp (normal/hover/pressed) used
    // by scroll bar thumbs, toggles and other pressable surfaces, so it maps
    // onto control surface tokens rather than text colors.
    style
        .set(
            Style::BRUSH_DARKEST,
            solid(theme, "slate.5", "slate.1", dark),
        )
        .set(
            Style::BRUSH_DARKER,
            solid(theme, "surface.elevated", "fluent.control.solid", dark),
        )
        .set(Style::BRUSH_DARK, solid(theme, "slate.4", "slate.3", dark))
        .set(
            Style::BRUSH_PRIMARY,
            solid(theme, "border.emphasis", "border.default", dark),
        )
        .set(
            Style::BRUSH_LIGHTER_PRIMARY,
            solid(theme, "slate.5", "border.subtle", dark),
        )
        .set(
            Style::BRUSH_LIGHT,
            solid(theme, "fluent.control.solid", "fluent.control.solid", dark),
        )
        .set(
            Style::BRUSH_LIGHTER,
            solid(theme, "fluent.control.hover", "fluent.control.hover", dark),
        )
        .set(
            Style::BRUSH_LIGHTEST,
            solid(theme, "fluent.control.pressed", "fluent.control.pressed", dark),
        )
        .set(Style::BRUSH_BRIGHT, solid(theme, "accent.solid", "accent.solid", dark))
        .set(
            Style::BRUSH_BRIGHTEST,
            solid(theme, "slate.12", "slate.1", dark),
        );

    // Accent + status colors.
    style
        .set(
            Style::BRUSH_BRIGHT_BLUE,
            Brush::Solid(color(theme, "accent.solid")),
        )
        .set(
            Style::BRUSH_DIM_BLUE,
            Brush::Solid(color(theme, "accent.pressed")),
        )
        .set(
            Style::BRUSH_HIGHLIGHT,
            Brush::Solid(color(theme, "accent.solid")),
        )
        .set(
            Style::BRUSH_TEXT,
            solid(theme, "text.primary", "text.primary", dark),
        )
        .set(
            Style::BRUSH_FOREGROUND,
            solid(theme, "text.primary", "text.primary", dark),
        )
        .set(
            Style::BRUSH_INFORMATION,
            Brush::Solid(color(theme, "accent.solid")),
        )
        .set(
            Style::BRUSH_WARNING,
            Brush::Solid(color(theme, "warning.solid")),
        )
        .set(
            Style::BRUSH_ERROR,
            Brush::Solid(color(theme, "error.solid")),
        )
        .set(
            Style::BRUSH_OK,
            Brush::Solid(color(theme, "success.solid")),
        );

    // Decorator state brushes (checkbox interiors, generic hover surfaces...).
    style
        .set(
            Style::BRUSH_OK_NORMAL,
            solid(theme, "surface.elevated", "fluent.control.solid", dark),
        )
        .set(
            Style::BRUSH_OK_HOVER,
            solid(theme, "fluent.list.low", "fluent.control.hover", dark),
        )
        .set(
            Style::BRUSH_OK_PRESSED,
            solid(theme, "fluent.list.medium", "fluent.control.pressed", dark),
        )
        .set(
            Style::BRUSH_CANCEL_NORMAL,
            solid(theme, "surface.elevated", "fluent.control.solid", dark),
        )
        .set(
            Style::BRUSH_CANCEL_HOVER,
            solid(theme, "fluent.list.low", "fluent.control.hover", dark),
        )
        .set(
            Style::BRUSH_CANCEL_PRESSED,
            solid(theme, "fluent.list.medium", "fluent.control.pressed", dark),
        );

    // Fluent control metrics.
    style
        .set(Button::CORNER_RADIUS, 3.0f32)
        .set(Button::BORDER_THICKNESS, Thickness::uniform(1.0))
        .set(ToggleButton::CORNER_RADIUS, 3.0f32)
        .set(ToggleButton::BORDER_THICKNESS, Thickness::uniform(1.0))
        .set(DropdownList::CORNER_RADIUS, 3.0f32);

    style
}

/// Convenience wrapper producing an embedded [`StyleResource`].
pub fn fluent_fyrox_style_resource(theme: &Theme, dark: bool) -> StyleResource {
    StyleResource::new_embedded(fluent_fyrox_style(theme, dark))
}
