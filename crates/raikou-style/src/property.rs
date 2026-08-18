//! Styleable property registration and lookup model.
//!
//! This module provides the foundation for a typed Rust-native property system
//! inspired by Avalonia's StyledProperty. Properties registered in this system
//! can participate in styling, inheritance, validation, and precedence resolution.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use raikou_core::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PropertyId {
    namespace: &'static str,
    name: &'static str,
}

impl PropertyId {
    pub const fn new(namespace: &'static str, name: &'static str) -> Self {
        Self { namespace, name }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn namespace(&self) -> &'static str {
        self.namespace
    }
}

#[derive(Debug, Clone)]
pub struct PropertyInfo {
    pub id: PropertyId,
    pub type_id: TypeId,
    pub inheritable: bool,
}

#[derive(Clone, Debug)]
pub struct PropertyRegistry {
    properties: HashMap<PropertyId, PropertyInfo>,
}

impl PropertyRegistry {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }

    pub fn register<T: Any + 'static>(&mut self, id: PropertyId, inheritable: bool) {
        self.properties.insert(
            id,
            PropertyInfo {
                id,
                type_id: TypeId::of::<T>(),
                inheritable,
            },
        );
    }

    pub fn get(&self, id: &PropertyId) -> Option<&PropertyInfo> {
        self.properties.get(id)
    }
}

impl Default for PropertyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub trait StyledProperty: Sized + 'static {
    const ID: PropertyId;
    const INHERITABLE: bool = false;
    type Value: Clone + 'static;

    fn default_value() -> Self::Value;
}

pub struct Property<P: StyledProperty> {
    value: Option<P::Value>,
}

impl<P: StyledProperty> Property<P> {
    pub fn new(value: Option<P::Value>) -> Self {
        Self { value }
    }

    pub fn set(&mut self, value: P::Value) {
        self.value = Some(value);
    }

    pub fn get(&self) -> Option<&P::Value> {
        self.value.as_ref()
    }

    pub fn take(&mut self) -> Option<P::Value> {
        self.value.take()
    }

    pub fn is_set(&self) -> bool {
        self.value.is_some()
    }
}

impl<P: StyledProperty> Default for Property<P> {
    fn default() -> Self {
        Self::new(None)
    }
}

#[derive(Clone, Debug)]
pub enum AlignItems {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}

impl Default for AlignItems {
    fn default() -> Self {
        Self::Stretch
    }
}

#[derive(Clone, Debug)]
pub enum JustifyContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Default for JustifyContent {
    fn default() -> Self {
        Self::Start
    }
}

#[derive(Clone, Debug)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl Default for FlexDirection {
    fn default() -> Self {
        Self::Row
    }
}

#[derive(Clone, Debug)]
pub enum Cursor {
    Default,
    Pointer,
    Text,
    Crosshair,
    Move,
    Wait,
    Help,
    Progress,
    NotAllowed,
}

impl Default for Cursor {
    fn default() -> Self {
        Self::Default
    }
}

pub mod layout {
    use super::*;

    pub const WIDTH: PropertyId = PropertyId::new("layout", "width");
    pub const HEIGHT: PropertyId = PropertyId::new("layout", "height");
    pub const MIN_WIDTH: PropertyId = PropertyId::new("layout", "min_width");
    pub const MIN_HEIGHT: PropertyId = PropertyId::new("layout", "min_height");
    pub const MAX_WIDTH: PropertyId = PropertyId::new("layout", "max_width");
    pub const MAX_HEIGHT: PropertyId = PropertyId::new("layout", "max_height");
    pub const MARGIN: PropertyId = PropertyId::new("layout", "margin");
    pub const PADDING: PropertyId = PropertyId::new("layout", "padding");
    pub const ALIGN_ITEMS: PropertyId = PropertyId::new("layout", "align_items");
    pub const JUSTIFY_CONTENT: PropertyId = PropertyId::new("layout", "justify_content");
    pub const FLEX_DIRECTION: PropertyId = PropertyId::new("layout", "flex_direction");
    pub const FLEX_GROW: PropertyId = PropertyId::new("layout", "flex_grow");
    pub const FLEX_SHRINK: PropertyId = PropertyId::new("layout", "flex_shrink");

    pub const WIDTH_LENGTH: PropertyId = PropertyId::new("layout", "width_length");
    pub const HEIGHT_LENGTH: PropertyId = PropertyId::new("layout", "height_length");
    pub const PADDING_STRUCT: PropertyId = PropertyId::new("layout", "padding_struct");
    pub const MARGIN_STRUCT: PropertyId = PropertyId::new("layout", "margin_struct");
}

pub mod box_style {
    use super::*;

    pub const BACKGROUND: PropertyId = PropertyId::new("box", "background");
    pub const BORDER_COLOR: PropertyId = PropertyId::new("box", "border_color");
    pub const BORDER_WIDTH: PropertyId = PropertyId::new("box", "border_width");
    pub const BORDER_RADIUS: PropertyId = PropertyId::new("box", "border_radius");
    pub const BORDER_RADIUS_STRUCT: PropertyId = PropertyId::new("box", "border_radius_struct");
    pub const SHADOW: PropertyId = PropertyId::new("box", "shadow");
    pub const OPACITY: PropertyId = PropertyId::new("box", "opacity");
}

pub mod text_style {
    use super::*;

    pub const COLOR: PropertyId = PropertyId::new("text", "color");
    pub const FONT_FAMILY: PropertyId = PropertyId::new("text", "font_family");
    pub const FONT_SIZE: PropertyId = PropertyId::new("text", "font_size");
    pub const FONT_WEIGHT: PropertyId = PropertyId::new("text", "font_weight");
    pub const LINE_HEIGHT: PropertyId = PropertyId::new("text", "line_height");
    pub const LETTER_SPACING: PropertyId = PropertyId::new("text", "letter_spacing");
    pub const TEXT_ALIGN: PropertyId = PropertyId::new("text", "text_align");
    pub const TEXT_DECORATION: PropertyId = PropertyId::new("text", "text_decoration");
}

pub mod interaction_style {
    use super::*;

    pub const CURSOR: PropertyId = PropertyId::new("interaction", "cursor");
    pub const TRANSITION: PropertyId = PropertyId::new("interaction", "transition");
}

pub fn register_core_properties(registry: &mut PropertyRegistry) {
    registry.register::<f32>(layout::WIDTH, false);
    registry.register::<f32>(layout::HEIGHT, false);
    registry.register::<f32>(layout::MIN_WIDTH, false);
    registry.register::<f32>(layout::MIN_HEIGHT, false);
    registry.register::<f32>(layout::MAX_WIDTH, false);
    registry.register::<f32>(layout::MAX_HEIGHT, false);
    registry.register::<raikou_core::geometry::Thickness>(layout::MARGIN, false);
    registry.register::<raikou_core::geometry::Thickness>(layout::PADDING, false);
    registry.register::<AlignItems>(layout::ALIGN_ITEMS, false);
    registry.register::<JustifyContent>(layout::JUSTIFY_CONTENT, false);
    registry.register::<FlexDirection>(layout::FLEX_DIRECTION, false);
    registry.register::<f32>(layout::FLEX_GROW, false);
    registry.register::<f32>(layout::FLEX_SHRINK, false);

    registry.register::<raikou_core::Length>(layout::WIDTH_LENGTH, false);
    registry.register::<raikou_core::Length>(layout::HEIGHT_LENGTH, false);
    registry.register::<raikou_core::Padding>(layout::PADDING_STRUCT, false);
    registry.register::<raikou_core::Margin>(layout::MARGIN_STRUCT, false);

    registry.register::<Color>(box_style::BACKGROUND, false);
    registry.register::<Color>(box_style::BORDER_COLOR, false);
    registry.register::<f32>(box_style::BORDER_WIDTH, false);
    registry.register::<f32>(box_style::BORDER_RADIUS, false);
    registry.register::<raikou_core::Radius>(box_style::BORDER_RADIUS_STRUCT, false);
    registry.register::<Option<Shadow>>(box_style::SHADOW, false);
    registry.register::<f32>(box_style::OPACITY, false);

    registry.register::<Color>(text_style::COLOR, true);
    registry.register::<String>(text_style::FONT_FAMILY, true);
    registry.register::<f32>(text_style::FONT_SIZE, true);
    registry.register::<f32>(text_style::FONT_WEIGHT, true);
    registry.register::<f32>(text_style::LINE_HEIGHT, true);
    registry.register::<f32>(text_style::LETTER_SPACING, true);
    registry.register::<TextAlign>(text_style::TEXT_ALIGN, true);

    registry.register::<Cursor>(interaction_style::CURSOR, true);
    registry.register::<Option<Transition>>(interaction_style::TRANSITION, false);
}

#[derive(Clone, Debug)]
pub struct Shadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

impl Shadow {
    pub fn new(offset_x: f32, offset_y: f32, blur: f32, spread: f32, color: Color) -> Self {
        Self {
            offset_x,
            offset_y,
            blur,
            spread,
            color,
        }
    }
}

impl Default for Shadow {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            blur: 0.0,
            spread: 0.0,
            color: Color::TRANSPARENT,
        }
    }
}

#[derive(Clone, Debug)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
}

impl Default for TextAlign {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Clone, Debug)]
pub enum TextDecoration {
    None,
    Underline,
    Overline,
    LineThrough,
}

impl Default for TextDecoration {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Debug)]
pub struct Transition {
    pub property: PropertyId,
    pub duration_ms: u32,
    pub easing: Easing,
}

#[derive(Clone, Debug)]
pub enum Easing {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl Default for Easing {
    fn default() -> Self {
        Self::Ease
    }
}
