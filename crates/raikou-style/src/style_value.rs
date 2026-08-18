//! Style value representations for the Raikou styling system.
//!
//! This module defines the typed values that can be assigned to styleable properties,
//! including raw values, token references, and special values like `None` and `Default`.

use std::fmt::Debug;

use raikou_core::{Color, Length};
use smol_str::SmolStr;

#[derive(Clone, Debug, PartialEq)]
pub enum StyleValue<T: Clone + Debug + PartialEq> {
    Value(T),
    Token(Token<T>),
    None,
    Default,
}

impl<T: Clone + Debug + PartialEq> StyleValue<T> {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }

    pub fn is_token(&self) -> bool {
        matches!(self, Self::Token(_))
    }

    pub fn unwrap_value(self) -> T {
        match self {
            Self::Value(v) => v,
            _ => panic!("StyleValue::unwrap_value called on non-Value variant"),
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::Value(v) => v,
            _ => default,
        }
    }

    pub fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
        match self {
            Self::Value(v) => v,
            _ => f(),
        }
    }
}

impl<T: Clone + Debug + PartialEq> Default for StyleValue<T> {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token<T: Clone + Debug + PartialEq> {
    pub scale: TokenScale,
    pub name: SmolStr,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Clone + Debug + PartialEq> Token<T> {
    pub fn new(scale: TokenScale, name: impl Into<SmolStr>) -> Self {
        Self {
            scale,
            name: name.into(),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenScale {
    Color,
    Space,
    Size,
    Radius,
    FontSize,
    FontWeight,
    FontFamily,
    LineHeight,
    LetterSpacing,
    Shadow,
    Duration,
}

impl TokenScale {
    pub fn color() -> Self {
        Self::Color
    }

    pub fn space() -> Self {
        Self::Space
    }

    pub fn size() -> Self {
        Self::Size
    }

    pub fn radius() -> Self {
        Self::Radius
    }

    pub fn font_size() -> Self {
        Self::FontSize
    }

    pub fn font_weight() -> Self {
        Self::FontWeight
    }

    pub fn font_family() -> Self {
        Self::FontFamily
    }

    pub fn line_height() -> Self {
        Self::LineHeight
    }

    pub fn letter_spacing() -> Self {
        Self::LetterSpacing
    }

    pub fn shadow() -> Self {
        Self::Shadow
    }

    pub fn duration() -> Self {
        Self::Duration
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColorValue {
    Solid(Color),
    Transparent,
    CurrentColor,
}

impl ColorValue {
    pub fn to_color(&self) -> Color {
        match self {
            Self::Solid(c) => *c,
            Self::Transparent => Color::TRANSPARENT,
            Self::CurrentColor => Color::new(0.0, 0.0, 0.0, 1.0),
        }
    }
}

impl From<Color> for ColorValue {
    fn from(c: Color) -> Self {
        Self::Solid(c)
    }
}

impl From<(u8, u8, u8)> for ColorValue {
    fn from(rgb: (u8, u8, u8)) -> Self {
        Self::Solid(Color::new(
            rgb.0 as f32 / 255.0,
            rgb.1 as f32 / 255.0,
            rgb.2 as f32 / 255.0,
            1.0,
        ))
    }
}

impl From<(u8, u8, u8, u8)> for ColorValue {
    fn from(rgba: (u8, u8, u8, u8)) -> Self {
        Self::Solid(Color::new(
            rgba.0 as f32 / 255.0,
            rgba.1 as f32 / 255.0,
            rgba.2 as f32 / 255.0,
            rgba.3 as f32 / 255.0,
        ))
    }
}

impl From<f32> for ColorValue {
    fn from(_: f32) -> Self {
        Self::CurrentColor
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SizeValue {
    Length(Length),
    Auto,
    Fill,
    Fit,
    None,
}

impl SizeValue {
    pub fn px(value: f32) -> Self {
        Self::Length(Length::fixed(value))
    }

    pub fn percent(value: f32) -> Self {
        Self::Length(Length::percent(value))
    }
}

impl Default for SizeValue {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SpacingValue {
    Length(Length),
    None,
}

impl SpacingValue {
    pub fn px(value: f32) -> Self {
        Self::Length(Length::fixed(value))
    }
}

impl Default for SpacingValue {
    fn default() -> Self {
        Self::None
    }
}
