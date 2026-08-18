//! Theme seam for raikou components.
//!
//! This phase resolves a component's visual style from its variant/size/state
//! into concrete fyrox values (colors, corner radius, border). The body is a
//! small hardcoded token set for now; the full raikou recipe/state theme
//! system (tokens, variants, pseudoclasses) will replace these bodies in a
//! later phase **without changing the builder API**.

use fyrox::core::color::Color;
use fyrox::gui::Thickness;

use raikou_core::ControlSize;

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

/// Resolved visual style for a raikou Button.
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

impl ButtonStyle {
    fn base(_variant: ButtonVariant, size: ControlSize) -> Self {
        Self {
            background: Color::TRANSPARENT,
            hover: Color::TRANSPARENT,
            pressed: Color::TRANSPARENT,
            text: Color::WHITE,
            border: Color::TRANSPARENT,
            border_thickness: Thickness::zero(),
            corner_radius: 4.0,
            font_size: size.font_size(),
            padding: size.padding(),
        }
    }
}

/// A raikou theme: a bag of design tokens plus per-component resolution.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Semantic color tokens. Keys follow the raikou names, e.g.
    /// `"accent.solid"`, `"surface.panel"`, `"text.primary"`.
    pub colors: Vec<(&'static str, Color)>,
    /// Whether this is a light or dark theme.
    pub dark: bool,
}

impl Theme {
    /// The default light theme (raikou `default_light`).
    pub fn light() -> Self {
        Self {
            colors: default_light_colors(),
            dark: false,
        }
    }

    /// The default dark theme (raikou `default_dark`).
    pub fn dark() -> Self {
        Self {
            colors: default_dark_colors(),
            dark: true,
        }
    }

    /// Looks up a semantic color token by name.
    pub fn color(&self, name: &str) -> Option<Color> {
        self.colors
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, color)| *color)
    }

    /// Resolves the visual style of a button for the given variant and size.
    ///
    /// This is the phase-1 seam: a variant/state aware resolution function
    /// that produces concrete fyrox values. The full recipe/state system will
    /// replace its body later without touching the builder API.
    pub fn resolve_button_style(&self, variant: ButtonVariant, size: ControlSize) -> ButtonStyle {
        let accent = self.color("accent.solid").unwrap_or_else(|| Color::opaque(37, 99, 235));
        let accent_hover = self.color("accent.hover").unwrap_or_else(|| Color::opaque(29, 78, 216));
        let accent_pressed = self.color("accent.pressed").unwrap_or_else(|| Color::opaque(30, 64, 175));
        let accent_muted = self.color("accent.muted").unwrap_or_else(|| Color::opaque(219, 234, 254));
        let text_primary = self.color("text.primary").unwrap_or_else(|| Color::opaque(30, 41, 59));
        let surface_panel = self.color("surface.panel").unwrap_or_else(|| Color::WHITE);
        let surface_muted = self.color("surface.muted").unwrap_or_else(|| Color::opaque(241, 245, 249));

        let mut style = ButtonStyle::base(variant, size);
        match variant {
            ButtonVariant::Filled => {
                style.background = accent;
                style.hover = accent_hover;
                style.pressed = accent_pressed;
                style.text = self.color("text.on.accent").unwrap_or(Color::WHITE);
                style.border_thickness = Thickness::zero();
            }
            ButtonVariant::Outline => {
                style.background = surface_panel;
                style.hover = accent_muted;
                style.pressed = accent_pressed;
                style.text = accent;
                style.border = accent;
                style.border_thickness = Thickness::uniform(1.0);
            }
            ButtonVariant::Ghost => {
                style.background = Color::TRANSPARENT;
                style.hover = surface_muted;
                style.pressed = accent_pressed;
                style.text = text_primary;
            }
            ButtonVariant::Subtle => {
                style.background = accent_muted;
                style.hover = accent_hover;
                style.pressed = accent_pressed;
                style.text = accent;
            }
            ButtonVariant::Link => {
                style.background = Color::TRANSPARENT;
                style.hover = accent_muted;
                style.pressed = accent_pressed;
                style.text = accent;
                style.border_thickness = Thickness::zero();
            }
        }
        style
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

fn default_light_colors() -> Vec<(&'static str, Color)> {
    vec![
        ("text.primary", Color::opaque(30, 41, 59)),
        ("text.muted", Color::opaque(100, 116, 139)),
        ("text.on.accent", Color::WHITE),
        ("surface.canvas", Color::opaque(248, 250, 252)),
        ("surface.panel", Color::WHITE),
        ("surface.elevated", Color::WHITE),
        ("surface.sunken", Color::opaque(241, 245, 249)),
        ("surface.muted", Color::opaque(241, 245, 249)),
        ("accent.solid", Color::opaque(37, 99, 235)),
        ("accent.hover", Color::opaque(29, 78, 216)),
        ("accent.pressed", Color::opaque(30, 64, 175)),
        ("accent.muted", Color::opaque(219, 234, 254)),
        ("accent.contrast", Color::opaque(59, 130, 246)),
        ("border.default", Color::opaque(203, 213, 225)),
        ("border.subtle", Color::opaque(226, 232, 240)),
        ("border.emphasis", Color::opaque(148, 163, 184)),
        ("focus.ring", Color::opaque(37, 99, 235)),
    ]
}

fn default_dark_colors() -> Vec<(&'static str, Color)> {
    vec![
        ("text.primary", Color::opaque(241, 245, 249)),
        ("text.muted", Color::opaque(148, 163, 184)),
        ("text.on.accent", Color::WHITE),
        ("surface.canvas", Color::opaque(15, 23, 42)),
        ("surface.panel", Color::opaque(30, 41, 59)),
        ("surface.elevated", Color::opaque(51, 65, 85)),
        ("surface.sunken", Color::opaque(15, 23, 42)),
        ("surface.muted", Color::opaque(51, 65, 85)),
        ("accent.solid", Color::opaque(59, 130, 246)),
        ("accent.hover", Color::opaque(37, 99, 235)),
        ("accent.pressed", Color::opaque(29, 78, 216)),
        ("accent.muted", Color::opaque(30, 64, 175)),
        ("accent.contrast", Color::opaque(147, 197, 253)),
        ("border.default", Color::opaque(51, 65, 85)),
        ("border.subtle", Color::opaque(71, 85, 105)),
        ("border.emphasis", Color::opaque(100, 116, 139)),
        ("focus.ring", Color::opaque(59, 130, 246)),
    ]
}
