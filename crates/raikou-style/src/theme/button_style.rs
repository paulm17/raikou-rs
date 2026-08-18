//! Button-specific style resolution on top of the recipe/state theme system.
//!
//! Keeps the builder-facing [`ButtonStyle`] and [`ButtonVariant`] types (in
//! backend-agnostic raikou-core units) while resolving them from the theme's
//! `button` recipe and its `appearance`/`size` variants plus hover/pressed
//! state styles.

use raikou_core::{Color, ControlSize, Thickness};

use crate::property::{box_style, layout, text_style};
use crate::recipe::{RecipeKey, VariantMap};
use crate::state::WidgetState;
use crate::style::Style;
use crate::Theme;

/// Visual appearance of a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    /// Solid accent fill.
    Filled,
    /// Transparent fill with an accent border.
    Outline,
    /// Fully transparent until hovered.
    Ghost,
    /// Muted accent tint.
    Subtle,
    /// Accent-colored label without a button chrome.
    Link,
}

impl ButtonVariant {
    /// The variant key used by theme recipes.
    pub fn name(self) -> &'static str {
        match self {
            ButtonVariant::Filled => "filled",
            ButtonVariant::Outline => "outline",
            ButtonVariant::Ghost => "ghost",
            ButtonVariant::Subtle => "subtle",
            ButtonVariant::Link => "link",
        }
    }
}

impl Default for ButtonVariant {
    fn default() -> Self {
        Self::Filled
    }
}

/// Resolved visual style for a raikou Button, in backend-agnostic units.
#[derive(Debug, Clone)]
pub struct ButtonStyle {
    /// Fill color of the normal state.
    pub background: Color,
    /// Fill color of the hovered state.
    pub hover: Color,
    /// Fill color of the pressed state.
    pub pressed: Color,
    /// Label color.
    pub text: Color,
    /// Stroke color of the border.
    pub border: Color,
    /// Border stroke thickness.
    pub border_thickness: Thickness,
    /// Corner radius in logical pixels.
    pub corner_radius: f32,
    /// Label font size in logical pixels.
    pub font_size: f32,
    /// Content padding.
    pub padding: Thickness,
}

impl Theme {
    /// Resolves the visual style of a button for the given variant and size.
    ///
    /// Resolves the theme's `button` recipe with the `appearance` and `size`
    /// variants under the idle/hovered/pressed states. Hover/pressed follow
    /// the recipe's `OPACITY` state styles (0.9 / 0.8 of the normal color),
    /// matching the reference recipe model.
    pub fn resolve_button_style(&self, variant: ButtonVariant, size: ControlSize) -> ButtonStyle {
        let key = RecipeKey::base("button");
        let variants = variant_map(variant, size);

        let idle = self
            .resolve_component_style(&key, &variants, &WidgetState::new())
            .unwrap_or_default();
        let hovered = self
            .resolve_component_style(&key, &variants, &WidgetState::new().hovered())
            .unwrap_or_default();
        let pressed = self
            .resolve_component_style(&key, &variants, &WidgetState::new().pressed())
            .unwrap_or_default();

        let fallback_bg = self.color("surface.panel").unwrap_or(Color::new(0.97, 0.97, 0.98, 1.0));
        let fallback_text = self.color("text.primary").unwrap_or(Color::new(0.09, 0.09, 0.10, 1.0));

        let background = style_color(&idle, box_style::BACKGROUND, fallback_bg);
        let text = style_color(&idle, text_style::COLOR, fallback_text);
        let border = style_color(&idle, box_style::BORDER_COLOR, Color::TRANSPARENT);

        let corner_radius = style_f32(&idle, box_style::BORDER_RADIUS, 8.0);
        let font_size = style_f32(&idle, text_style::FONT_SIZE, 16.0);
        let padding_px = style_f32(&idle, layout::PADDING, 12.0);
        let border_width = style_f32(&idle, box_style::BORDER_WIDTH, 0.0);

        let hover_alpha = style_f32(&hovered, box_style::OPACITY, 1.0);
        let pressed_alpha = style_f32(&pressed, box_style::OPACITY, 1.0);

        ButtonStyle {
            background,
            hover: with_alpha(background, background.alpha * hover_alpha),
            pressed: with_alpha(background, background.alpha * pressed_alpha),
            text,
            border,
            border_thickness: if border_width > 0.0 {
                Thickness::uniform(border_width)
            } else {
                Thickness::ZERO
            },
            corner_radius,
            font_size,
            padding: Thickness::uniform(padding_px),
        }
    }
}

fn variant_map(variant: ButtonVariant, size: ControlSize) -> VariantMap {
    let mut map = VariantMap::new();
    map.insert("appearance", variant.name());
    map.insert("size", size_variant(size));
    map
}

fn size_variant(size: ControlSize) -> &'static str {
    match size {
        ControlSize::XSmall => "xsmall",
        ControlSize::Small => "small",
        ControlSize::Medium => "medium",
        ControlSize::Large => "large",
        ControlSize::XLarge => "xlarge",
    }
}

fn style_color(style: &Style, property: crate::PropertyId, fallback: Color) -> Color {
    match style
        .get(&property)
        .and_then(crate::style::StyleValueEntry::as_value)
    {
        Some(crate::style::StyleValueHolder::Color(color)) => *color,
        _ => fallback,
    }
}

fn style_f32(style: &Style, property: crate::PropertyId, fallback: f32) -> f32 {
    match style
        .get(&property)
        .and_then(crate::style::StyleValueEntry::as_value)
    {
        Some(crate::style::StyleValueHolder::F32(value)) => *value,
        _ => fallback,
    }
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color::new(color.red, color.green, color.blue, alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn filled_medium_resolves_blue9() {
        let theme = Theme::light();
        let style = theme.resolve_button_style(ButtonVariant::Filled, ControlSize::Medium);
        assert!(approx(style.background.red, 0.13));
        assert!(approx(style.background.green, 0.39));
        assert!(approx(style.background.blue, 0.94));
        assert!(approx(style.background.alpha, 1.0));
    }

    #[test]
    fn hover_and_pressed_apply_opacity() {
        let theme = Theme::light();
        let style = theme.resolve_button_style(ButtonVariant::Filled, ControlSize::Medium);
        assert!(approx(style.hover.alpha, 0.9));
        assert!(approx(style.pressed.alpha, 0.8));
        assert!(approx(style.hover.red, style.background.red));
    }

    #[test]
    fn outline_has_border() {
        let theme = Theme::light();
        let style = theme.resolve_button_style(ButtonVariant::Outline, ControlSize::Medium);
        assert!(approx(style.border_thickness.left, 1.0));
    }

    #[test]
    fn size_variant_changes_font_and_padding() {
        let theme = Theme::light();
        let small = theme.resolve_button_style(ButtonVariant::Filled, ControlSize::Small);
        let large = theme.resolve_button_style(ButtonVariant::Filled, ControlSize::Large);
        assert!(small.font_size < large.font_size);
        assert!(approx(small.padding.left, 8.0));
        assert!(approx(large.padding.left, 16.0));
    }
}

