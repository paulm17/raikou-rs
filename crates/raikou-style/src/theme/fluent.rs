//! Avalonia Fluent–derived themes.
//!
//! Token values are transcribed from the Avalonia `FluentTheme` source
//! (`BaseColorsPalette.xaml`, `BaseResources.xaml`, `FluentControlResources.xaml`):
//! the UWP/WinUI system palette with the default accent `#0078D7` and its
//! luminosity shades (`Light1..3` / `Dark1..3`).

use super::{Theme, ThemeVariant};
use crate::property::{box_style, layout, text_style};
use crate::recipe::RecipeKey;
use crate::state::WidgetState;
use crate::style::StylePrecedence;
use raikou_core::Color;

/// Converts a packed `0xRRGGBBAA` value into a [`Color`].
const fn hex(rgb: u32, alpha: f32) -> Color {
    let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
    let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
    let b = (rgb & 0xFF) as f32 / 255.0;
    Color::new(r, g, b, alpha)
}

// Default accent (Windows blue) and its luminosity shades.
const ACCENT: u32 = 0x0078D7;
const ACCENT_LIGHT1: u32 = 0x429CE3;
const ACCENT_LIGHT2: u32 = 0x76B9ED;
const ACCENT_LIGHT3: u32 = 0xA6D8FF;
const ACCENT_DARK1: u32 = 0x005A9E;
const ACCENT_DARK2: u32 = 0x004275;
const ACCENT_DARK3: u32 = 0x002642;

fn shared_typography(
    t: &mut crate::theme::tokens::TypographyScale,
) {
    t.font_family("sans-serif", "'Segoe UI Variable Text', 'Segoe UI', Inter, system-ui, sans-serif");
    t.font_family("mono", "ui-monospace, 'Cascadia Mono', 'SF Mono', Menlo, Consolas, monospace");
    t.font_size("xs", 12.0);
    t.font_size("sm", 14.0);
    t.font_size("md", 16.0);
    t.font_size("lg", 18.0);
    t.font_size("xl", 20.0);
    t.font_size("2xl", 24.0);
    t.font_size("3xl", 28.0);
    t.font_size("4xl", 36.0);
    t.font_weight("thin", 100.0);
    t.font_weight("light", 300.0);
    t.font_weight("regular", 400.0);
    t.font_weight("medium", 500.0);
    t.font_weight("semibold", 600.0);
    t.font_weight("bold", 700.0);
    t.font_weight("extrabold", 800.0);
    t.font_weight("black", 900.0);
    t.line_height("none", 1.0);
    t.line_height("tight", 1.25);
    t.line_height("snug", 1.375);
    t.line_height("normal", 1.5);
    t.line_height("relaxed", 1.625);
    t.line_height("loose", 2.0);
    t.letter_spacing("tighter", -0.8);
    t.letter_spacing("tight", -0.4);
    t.letter_spacing("normal", 0.0);
    t.letter_spacing("wide", 0.4);
    t.letter_spacing("wider", 0.8);
    t.letter_spacing("widest", 1.6);
}

fn shared_space(s: &mut crate::theme::tokens::SpaceScale) {
    s.insert("0", 0.0);
    s.insert("1", 4.0);
    s.insert("2", 8.0);
    s.insert("3", 12.0);
    s.insert("4", 16.0);
    s.insert("5", 20.0);
    s.insert("6", 24.0);
    s.insert("8", 32.0);
    s.insert("10", 40.0);
    s.insert("12", 48.0);
    s.insert("16", 64.0);
    s.insert("20", 80.0);
    s.insert("24", 96.0);
}

fn shared_radii(r: &mut crate::theme::tokens::RadiusScale) {
    r.insert("none", 0.0);
    // ControlCornerRadius = 3, OverlayCornerRadius = 5 in Avalonia Fluent.
    r.insert("sm", 3.0);
    r.insert("md", 5.0);
    r.insert("lg", 8.0);
    r.insert("xl", 12.0);
    r.insert("2xl", 16.0);
    r.insert("pill", 999.0);
    r.insert("full", 999.0);
}

fn shared_shadows_light(s: &mut crate::theme::tokens::ShadowScale) {
    use crate::property::Shadow;
    s.insert("none", Shadow::default());
    s.insert("sm", Shadow::new(0.0, 1.0, 2.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.098)));
    s.insert("md", Shadow::new(0.0, 4.0, 8.0, -1.0, Color::new(0.0, 0.0, 0.0, 0.14)));
    s.insert("lg", Shadow::new(0.0, 8.0, 16.0, -2.0, Color::new(0.0, 0.0, 0.0, 0.19)));
    s.insert("xl", Shadow::new(0.0, 16.0, 32.0, -4.0, Color::new(0.0, 0.0, 0.0, 0.24)));
    s.insert("focus", Shadow::new(0.0, 0.0, 0.0, 2.0, hex(ACCENT_LIGHT1, 0.55)));
    s.insert("inner", Shadow::new(0.0, 2.0, 4.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.098)));
}

fn shared_shadows_dark(s: &mut crate::theme::tokens::ShadowScale) {
    use crate::property::Shadow;
    s.insert("none", Shadow::default());
    s.insert("sm", Shadow::new(0.0, 1.0, 2.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.35)));
    s.insert("md", Shadow::new(0.0, 4.0, 8.0, -1.0, Color::new(0.0, 0.0, 0.0, 0.45)));
    s.insert("lg", Shadow::new(0.0, 8.0, 16.0, -2.0, Color::new(0.0, 0.0, 0.0, 0.55)));
    s.insert("xl", Shadow::new(0.0, 16.0, 32.0, -4.0, Color::new(0.0, 0.0, 0.0, 0.65)));
    s.insert("focus", Shadow::new(0.0, 0.0, 0.0, 2.0, hex(ACCENT_LIGHT1, 0.60)));
    s.insert("inner", Shadow::new(0.0, 2.0, 4.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.35)));
}

/// Fluent "standard" button recipe (non-accent): translucent gray fill,
/// transparent border, high-contrast label; hover lightens, press darkens.
fn button_recipe(b: &mut super::ComponentRecipeBuilder, dark: bool) {
    let (fg, bg_normal, fg_disabled, accent_bg, accent_bg_hover, accent_bg_pressed) = if dark {
        (
            Color::new(1.0, 1.0, 1.0, 1.0),
            Color::new(1.0, 1.0, 1.0, 0.2),
            Color::new(1.0, 1.0, 1.0, 0.4),
            hex(ACCENT, 1.0),
            hex(ACCENT_LIGHT1, 1.0),
            hex(ACCENT_DARK1, 1.0),
        )
    } else {
        (
            Color::new(0.0, 0.0, 0.0, 1.0),
            Color::new(0.0, 0.0, 0.0, 0.2),
            Color::new(0.0, 0.0, 0.0, 0.4),
            hex(ACCENT, 1.0),
            hex(ACCENT_LIGHT1, 1.0),
            hex(ACCENT_DARK1, 1.0),
        )
    };

    b.base(|s| {
        s.set_color(box_style::BACKGROUND, accent_bg, StylePrecedence::BaseRecipe, crate::style::StyleSource::Recipe);
        s.set_color(text_style::COLOR, Color::new(1.0, 1.0, 1.0, 1.0), StylePrecedence::BaseRecipe, crate::style::StyleSource::Recipe);
        s.set_f32(box_style::BORDER_RADIUS, 3.0, StylePrecedence::BaseRecipe, crate::style::StyleSource::Recipe);
        s.set_f32(layout::PADDING, 6.0, StylePrecedence::BaseRecipe, crate::style::StyleSource::Recipe);
        s.set_f32(text_style::FONT_SIZE, 14.0, StylePrecedence::BaseRecipe, crate::style::StyleSource::Recipe);
        s.set_f32(text_style::FONT_WEIGHT, 400.0, StylePrecedence::BaseRecipe, crate::style::StyleSource::Recipe);
    });
    b.variant("appearance", "filled", |s| {
        s.set_color(box_style::BACKGROUND, accent_bg, StylePrecedence::Variant, crate::style::StyleSource::Variant);
        s.set_color(text_style::COLOR, Color::new(1.0, 1.0, 1.0, 1.0), StylePrecedence::Variant, crate::style::StyleSource::Variant);
    });
    b.variant("appearance", "outline", |s| {
        s.set_color(box_style::BACKGROUND, bg_normal, StylePrecedence::Variant, crate::style::StyleSource::Variant);
        s.set_color(text_style::COLOR, fg, StylePrecedence::Variant, crate::style::StyleSource::Variant);
    });
    b.variant("appearance", "ghost", |s| {
        s.set_color(box_style::BACKGROUND, Color::TRANSPARENT, StylePrecedence::Variant, crate::style::StyleSource::Variant);
        s.set_color(text_style::COLOR, fg, StylePrecedence::Variant, crate::style::StyleSource::Variant);
    });
    b.variant("appearance", "subtle", |s| {
        s.set_color(box_style::BACKGROUND, bg_normal, StylePrecedence::Variant, crate::style::StyleSource::Variant);
        s.set_color(text_style::COLOR, fg, StylePrecedence::Variant, crate::style::StyleSource::Variant);
    });
    b.variant("appearance", "link", |s| {
        s.set_color(box_style::BACKGROUND, Color::TRANSPARENT, StylePrecedence::Variant, crate::style::StyleSource::Variant);
        s.set_color(text_style::COLOR, hex(ACCENT, 1.0), StylePrecedence::Variant, crate::style::StyleSource::Variant);
    });
    b.variant("size", "small", |s| {
        s.set_f32(layout::PADDING, 4.0, StylePrecedence::Variant, crate::style::StyleSource::Variant);
        s.set_f32(text_style::FONT_SIZE, 12.0, StylePrecedence::Variant, crate::style::StyleSource::Variant);
    });
    b.variant("size", "medium", |s| {
        s.set_f32(layout::PADDING, 6.0, StylePrecedence::Variant, crate::style::StyleSource::Variant);
        s.set_f32(text_style::FONT_SIZE, 14.0, StylePrecedence::Variant, crate::style::StyleSource::Variant);
    });
    b.variant("size", "large", |s| {
        s.set_f32(layout::PADDING, 8.0, StylePrecedence::Variant, crate::style::StyleSource::Variant);
        s.set_f32(text_style::FONT_SIZE, 16.0, StylePrecedence::Variant, crate::style::StyleSource::Variant);
    });
    b.state(WidgetState::new().hovered(), |s| {
        s.set_color(box_style::BACKGROUND, accent_bg_hover, StylePrecedence::StateStyle, crate::style::StyleSource::State);
    });
    b.state(WidgetState::new().pressed(), |s| {
        s.set_color(box_style::BACKGROUND, accent_bg_pressed, StylePrecedence::StateStyle, crate::style::StyleSource::State);
    });
    b.state(WidgetState::new().focused(), |s| {
        s.set_f32(box_style::BORDER_WIDTH, 2.0, StylePrecedence::StateStyle, crate::style::StyleSource::State);
        s.set_color(box_style::BORDER_COLOR, hex(ACCENT, 0.55), StylePrecedence::StateStyle, crate::style::StyleSource::State);
    });
    b.state(WidgetState::new().disabled(), |s| {
        s.set_f32(box_style::OPACITY, 0.5, StylePrecedence::StateStyle, crate::style::StyleSource::State);
        s.set_color(box_style::BACKGROUND, bg_normal, StylePrecedence::StateStyle, crate::style::StyleSource::State);
        s.set_color(text_style::COLOR, fg_disabled, StylePrecedence::StateStyle, crate::style::StyleSource::State);
    });
}

/// The Avalonia Fluent light theme ("Default" dictionary).
pub fn fluent_light() -> Theme {
    Theme::builder("fluent-light")
        .variant(ThemeVariant::Light)
        .colors(|c| {
            c.raw("white", Color::new(1.0, 1.0, 1.0, 1.0));
            c.raw("black", Color::new(0.0, 0.0, 0.0, 1.0));

            // Gray ramp anchored on the UWP chrome grays.
            c.raw("slate.1", hex(0xFFFFFF, 1.0));
            c.raw("slate.2", hex(0xF9F9F9, 1.0));
            c.raw("slate.3", hex(0xF2F2F2, 1.0));
            c.raw("slate.4", hex(0xE6E6E6, 1.0));
            c.raw("slate.5", hex(0xCCCCCC, 1.0));
            c.raw("slate.6", hex(0xACACAC, 1.0));
            c.raw("slate.7", hex(0x999999, 1.0));
            c.raw("slate.8", hex(0x767676, 1.0));
            c.raw("slate.9", hex(0x5D5D5D, 1.0));
            c.raw("slate.10", hex(0x464646, 1.0));
            c.raw("slate.11", hex(0x2B2B2B, 1.0));
            c.raw("slate.12", hex(0x171717, 1.0));

            // Accent ramp around #0078D7.
            c.raw("blue.1", hex(0xEDF5FC, 1.0));
            c.raw("blue.2", hex(0xD8EBFA, 1.0));
            c.raw("blue.3", hex(0xB9DDF6, 1.0));
            c.raw("blue.4", hex(0x93CBF1, 1.0));
            c.raw("blue.5", hex(0x66B4EA, 1.0));
            c.raw("blue.6", hex(ACCENT_LIGHT1, 1.0));
            c.raw("blue.7", hex(0x008AE0, 1.0));
            c.raw("blue.8", hex(0x007CD9, 1.0));
            c.raw("blue.9", hex(ACCENT, 1.0));
            c.raw("blue.10", hex(0x0068BD, 1.0));
            c.raw("blue.11", hex(ACCENT_DARK1, 1.0));
            c.raw("blue.12", hex(ACCENT_DARK2, 1.0));

            // Raw UWP system palette (alpha preserved where Avalonia uses it).
            c.alias_color("fluent.base.high", Color::new(0.0, 0.0, 0.0, 1.0));
            c.alias_color("fluent.base.medium.high", Color::new(0.0, 0.0, 0.0, 0.8));
            c.alias_color("fluent.base.medium", Color::new(0.0, 0.0, 0.0, 0.6));
            c.alias_color("fluent.base.medium.low", Color::new(0.0, 0.0, 0.0, 0.4));
            c.alias_color("fluent.base.low", Color::new(0.0, 0.0, 0.0, 0.2));
            c.alias_color("fluent.alt.high", Color::new(1.0, 1.0, 1.0, 1.0));
            c.alias_color("fluent.chrome.low", hex(0xF2F2F2, 1.0));
            c.alias_color("fluent.chrome.medium", hex(0xE6E6E6, 1.0));
            c.alias_color("fluent.chrome.high", hex(0xCCCCCC, 1.0));
            c.alias_color("fluent.chrome.gray", hex(0x767676, 1.0));
            c.alias_color("fluent.chrome.disabled.high", hex(0xCCCCCC, 1.0));
            c.alias_color("fluent.chrome.disabled.low", hex(0x7A7A7A, 1.0));
            c.alias_color("fluent.list.low", Color::new(0.0, 0.0, 0.0, 0.1));
            c.alias_color("fluent.list.medium", Color::new(0.0, 0.0, 0.0, 0.2));
            c.alias_color("fluent.transient.border", Color::new(0.0, 0.0, 0.0, 0.14));
            c.alias_color("fluent.control.solid", hex(0xFBFBFB, 1.0));
            c.alias_color("fluent.control.hover", hex(0xF5F5F5, 1.0));
            c.alias_color("fluent.control.pressed", hex(0xEBEBEB, 1.0));
            c.alias_color("fluent.accent", hex(ACCENT, 1.0));
            c.alias_color("fluent.accent.light1", hex(ACCENT_LIGHT1, 1.0));
            c.alias_color("fluent.accent.light2", hex(ACCENT_LIGHT2, 1.0));
            c.alias_color("fluent.accent.light3", hex(ACCENT_LIGHT3, 1.0));
            c.alias_color("fluent.accent.dark1", hex(ACCENT_DARK1, 1.0));
            c.alias_color("fluent.accent.dark2", hex(ACCENT_DARK2, 1.0));
            c.alias_color("fluent.accent.dark3", hex(ACCENT_DARK3, 1.0));
            c.alias_color("fluent.error.text", hex(0xC50500, 1.0));

            c.alias("text.primary", "slate.12");
            c.alias("text.secondary", "slate.10");
            c.alias("text.muted", "slate.8");
            c.alias("surface.canvas", "slate.1");
            c.alias("surface.panel", "slate.3");
            c.alias("surface.elevated", "white");
            c.alias("surface.sunken", "slate.4");
            c.alias("accent.solid", "blue.9");
            c.alias("accent.hover", "blue.6");
            c.alias("accent.pressed", "blue.11");
            c.alias_color("accent.muted", hex(ACCENT, 0.36));
            c.alias("accent.contrast", "white");
            c.alias_color("border.subtle", Color::new(0.0, 0.0, 0.0, 0.14));
            c.alias_color("border.default", Color::new(0.0, 0.0, 0.0, 0.4));
            c.alias_color("border.emphasis", Color::new(0.0, 0.0, 0.0, 0.6));
            c.alias_color("success.solid", hex(0x128B44, 1.0));
            c.alias_color("warning.solid", hex(0xFFC316, 1.0));
            c.alias_color("error.solid", hex(0xF03916, 1.0));
        })
        .space(shared_space)
        .radii(shared_radii)
        .typography(shared_typography)
        .shadows(shared_shadows_light)
        .component(RecipeKey::base("button"), |b| button_recipe(b, false))
        .build()
}

/// The Avalonia Fluent dark theme ("Dark" dictionary).
pub fn fluent_dark() -> Theme {
    Theme::builder("fluent-dark")
        .variant(ThemeVariant::Dark)
        .colors(|c| {
            c.raw("white", Color::new(1.0, 1.0, 1.0, 1.0));
            c.raw("black", Color::new(0.0, 0.0, 0.0, 1.0));

            // Gray ramp anchored on the dark chrome surfaces.
            c.raw("slate.1", hex(0x191919, 1.0));
            c.raw("slate.2", hex(0x1F1F1F, 1.0));
            c.raw("slate.3", hex(0x232323, 1.0));
            c.raw("slate.4", hex(0x2B2B2B, 1.0));
            c.raw("slate.5", hex(0x333333, 1.0));
            c.raw("slate.6", hex(0x3D3D3D, 1.0));
            c.raw("slate.7", hex(0x4D4D4D, 1.0));
            c.raw("slate.8", hex(0x595959, 1.0));
            c.raw("slate.9", hex(0x696969, 1.0));
            c.raw("slate.10", hex(0x767676, 1.0));
            c.raw("slate.11", hex(0x858585, 1.0));
            c.raw("slate.12", hex(0xF2F2F2, 1.0));

            // Accent ramp: darkest shades first, accent mid-ramp.
            c.raw("blue.1", hex(0x001731, 1.0));
            c.raw("blue.2", hex(0x002142, 1.0));
            c.raw("blue.3", hex(0x002B54, 1.0));
            c.raw("blue.4", hex(ACCENT_DARK2, 1.0));
            c.raw("blue.5", hex(0x004A8F, 1.0));
            c.raw("blue.6", hex(ACCENT_DARK1, 1.0));
            c.raw("blue.7", hex(ACCENT, 1.0));
            c.raw("blue.8", hex(0x1F86DE, 1.0));
            c.raw("blue.9", hex(ACCENT_LIGHT1, 1.0));
            c.raw("blue.10", hex(ACCENT_LIGHT2, 1.0));
            c.raw("blue.11", hex(ACCENT_LIGHT3, 1.0));
            c.raw("blue.12", hex(0xC9E7FF, 1.0));

            // Raw UWP system palette (dark values).
            c.alias_color("fluent.base.high", Color::new(1.0, 1.0, 1.0, 1.0));
            c.alias_color("fluent.base.medium.high", Color::new(1.0, 1.0, 1.0, 0.8));
            c.alias_color("fluent.base.medium", Color::new(1.0, 1.0, 1.0, 0.6));
            c.alias_color("fluent.base.medium.low", Color::new(1.0, 1.0, 1.0, 0.4));
            c.alias_color("fluent.base.low", Color::new(1.0, 1.0, 1.0, 0.2));
            c.alias_color("fluent.alt.high", Color::new(0.0, 0.0, 0.0, 1.0));
            c.alias_color("fluent.chrome.low", hex(0x171717, 1.0));
            c.alias_color("fluent.chrome.medium", hex(0x1F1F1F, 1.0));
            c.alias_color("fluent.chrome.medium.low", hex(0x2B2B2B, 1.0));
            c.alias_color("fluent.chrome.high", hex(0x767676, 1.0));
            c.alias_color("fluent.chrome.disabled.high", hex(0x333333, 1.0));
            c.alias_color("fluent.chrome.disabled.low", hex(0x858585, 1.0));
            c.alias_color("fluent.list.low", Color::new(1.0, 1.0, 1.0, 0.1));
            c.alias_color("fluent.list.medium", Color::new(1.0, 1.0, 1.0, 0.2));
            c.alias_color("fluent.transient.border", Color::new(0.0, 0.0, 0.0, 0.36));
            c.alias_color("fluent.control.solid", hex(0x272727, 1.0));
            c.alias_color("fluent.control.hover", hex(0x303030, 1.0));
            c.alias_color("fluent.control.pressed", hex(0x383838, 1.0));
            c.alias_color("fluent.accent", hex(ACCENT, 1.0));
            c.alias_color("fluent.accent.light1", hex(ACCENT_LIGHT1, 1.0));
            c.alias_color("fluent.accent.light2", hex(ACCENT_LIGHT2, 1.0));
            c.alias_color("fluent.accent.light3", hex(ACCENT_LIGHT3, 1.0));
            c.alias_color("fluent.accent.dark1", hex(ACCENT_DARK1, 1.0));
            c.alias_color("fluent.accent.dark2", hex(ACCENT_DARK2, 1.0));
            c.alias_color("fluent.accent.dark3", hex(ACCENT_DARK3, 1.0));
            c.alias_color("fluent.error.text", hex(0xFFF000, 1.0));

            c.alias("text.primary", "slate.12");
            c.alias("text.secondary", "slate.11");
            c.alias("text.muted", "slate.10");
            c.alias("surface.canvas", "slate.1");
            c.alias("surface.panel", "slate.2");
            c.alias("surface.elevated", "slate.4");
            c.alias("surface.sunken", "black");
            c.alias("accent.solid", "blue.7");
            c.alias("accent.hover", "blue.9");
            c.alias("accent.pressed", "blue.6");
            c.alias_color("accent.muted", hex(ACCENT, 0.36));
            c.alias("accent.contrast", "white");
            c.alias_color("border.subtle", Color::new(1.0, 1.0, 1.0, 0.2));
            c.alias_color("border.default", Color::new(1.0, 1.0, 1.0, 0.36));
            c.alias_color("border.emphasis", Color::new(1.0, 1.0, 1.0, 0.6));
            c.alias_color("success.solid", hex(0x1F9E45, 1.0));
            c.alias_color("warning.solid", hex(0xFDB328, 1.0));
            c.alias_color("error.solid", hex(0xBD202C, 1.0));
        })
        .space(shared_space)
        .radii(shared_radii)
        .typography(shared_typography)
        .shadows(shared_shadows_dark)
        .component(RecipeKey::base("button"), |b| button_recipe(b, true))
        .build()
}

