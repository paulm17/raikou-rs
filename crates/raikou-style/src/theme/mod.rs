//! Theme system for the Raikou styling framework.
//!
//! This module provides the theme infrastructure including token scales,
//! semantic token aliases, and theme builder API.

pub mod button_style;
pub mod control_theme;
pub mod fluent;
pub mod fyrox_bridge;
pub mod provider;
pub mod tokens;
pub mod variant;

pub use button_style::{ButtonStyle, ButtonVariant};
pub use control_theme::{ControlTheme, ControlThemeRegistry};
pub use provider::ThemeVariant;
pub use provider::{ThemeProvider, ThemeProviderExt, TokenValue};
pub use tokens::{
    ColorScale, ComponentThemeRegistry, RadiusScale, ShadowScale, SizeScale, SpaceScale,
    TokenRegistry, TypographyScale,
};
pub use variant::ThemeVariantScope;

use crate::property::{box_style, layout, text_style};
use crate::recipe::{ComponentRecipe, RecipeKey};
use crate::state::WidgetState;
use crate::style::{StylePrecedence, StyleSource};
use crate::style_value::TokenScale;
use raikou_core::Color;
use smol_str::SmolStr;
use std::sync::Arc;

pub struct ThemeBuilder {
    name: SmolStr,
    colors: ColorScale,
    space: SpaceScale,
    sizes: SizeScale,
    radii: RadiusScale,
    typography: TypographyScale,
    shadows: ShadowScale,
    components: ComponentThemeRegistry,
    variant: ThemeVariant,
}

impl ThemeBuilder {
    pub fn new(name: SmolStr) -> Self {
        Self {
            name,
            colors: ColorScale::new(),
            space: SpaceScale::new(),
            sizes: SizeScale::new(),
            radii: RadiusScale::new(),
            typography: TypographyScale::new(),
            shadows: ShadowScale::new(),
            components: ComponentThemeRegistry::new(),
            variant: ThemeVariant::Default,
        }
    }

    pub fn colors(mut self, f: impl FnOnce(&mut ColorScale)) -> Self {
        f(&mut self.colors);
        self
    }

    pub fn space(mut self, f: impl FnOnce(&mut SpaceScale)) -> Self {
        f(&mut self.space);
        self
    }

    pub fn sizes(mut self, f: impl FnOnce(&mut SizeScale)) -> Self {
        f(&mut self.sizes);
        self
    }

    pub fn radii(mut self, f: impl FnOnce(&mut RadiusScale)) -> Self {
        f(&mut self.radii);
        self
    }

    pub fn typography(mut self, f: impl FnOnce(&mut TypographyScale)) -> Self {
        f(&mut self.typography);
        self
    }

    pub fn shadows(mut self, f: impl FnOnce(&mut ShadowScale)) -> Self {
        f(&mut self.shadows);
        self
    }

    pub fn variant(mut self, variant: ThemeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn component(
        mut self,
        key: RecipeKey,
        f: impl FnOnce(&mut ComponentRecipeBuilder),
    ) -> Self {
        let mut builder = ComponentRecipeBuilder::new(key.component().clone());
        f(&mut builder);
        self.components.register(key, builder.build());
        self
    }

    pub fn build(self) -> Theme {
        Theme {
            name: self.name,
            colors: self.colors,
            space: self.space,
            sizes: self.sizes,
            radii: self.radii,
            typography: self.typography,
            shadows: self.shadows,
            components: self.components,
            variant: self.variant,
        }
    }
}

pub struct ComponentRecipeBuilder {
    component: SmolStr,
    base_style: crate::Style,
    variants: std::collections::HashMap<SmolStr, crate::Style>,
    state_styles: Vec<(WidgetState, crate::Style)>,
}

impl ComponentRecipeBuilder {
    pub fn new(component: impl Into<SmolStr>) -> Self {
        Self {
            component: component.into(),
            base_style: crate::Style::new(),
            variants: std::collections::HashMap::new(),
            state_styles: Vec::new(),
        }
    }

    pub fn base(&mut self, f: impl FnOnce(&mut crate::Style)) -> &mut Self {
        f(&mut self.base_style);
        self
    }

    pub fn variant(
        &mut self,
        group: &str,
        name: &str,
        f: impl FnOnce(&mut crate::Style),
    ) -> &mut Self {
        let key = SmolStr::from(format!("{}:{}", group, name));
        let mut style = crate::Style::new();
        f(&mut style);
        self.variants.insert(key, style);
        self
    }

    pub fn state(
        &mut self,
        widget_state: WidgetState,
        f: impl FnOnce(&mut crate::Style),
    ) -> &mut Self {
        let mut style = crate::Style::new();
        f(&mut style);
        self.state_styles.push((widget_state, style));
        self
    }

    pub fn build(self) -> ComponentRecipe {
        use crate::recipe::StateStyleMap;
        let mut state_map = StateStyleMap::new();
        for (state, style) in self.state_styles {
            state_map.insert(state, style);
        }
        ComponentRecipe {
            key: RecipeKey::base(self.component),
            base_style: self.base_style,
            variants: self.variants,
            compound_variants: Vec::new(),
            default_variants: crate::recipe::VariantMap::new(),
            state_styles: state_map,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub name: SmolStr,
    pub colors: ColorScale,
    pub space: SpaceScale,
    pub sizes: SizeScale,
    pub radii: RadiusScale,
    pub typography: TypographyScale,
    pub shadows: ShadowScale,
    pub components: ComponentThemeRegistry,
    variant: ThemeVariant,
}

impl Theme {
    pub fn component_theme(&self, key: &RecipeKey) -> Option<ControlTheme> {
        self.get_component_recipe(key).map(|recipe| {
            ControlTheme::new(
                key.component().clone(),
                key.component().clone(),
                (*recipe).clone(),
            )
        })
    }

    pub fn resolve_component_style(
        &self,
        key: &RecipeKey,
        variants: &crate::recipe::VariantMap,
        state: &WidgetState,
    ) -> Option<crate::Style> {
        let recipe = self.get_component_recipe(key)?;
        let mut style = crate::Style::new();
        style.merge(recipe.base_style());
        style.merge(&recipe.resolve_variants(variants, state));
        Some(style)
    }

    pub fn builder(name: impl Into<SmolStr>) -> ThemeBuilder {
        ThemeBuilder::new(name.into())
    }

    /// The default light theme (raikou `default_light`).
    pub fn light() -> Self {
        let mut theme = Self::default_light();
        theme.variant = ThemeVariant::Light;
        theme
    }

    /// The default dark theme (raikou `default_dark`).
    pub fn dark() -> Self {
        let mut theme = Self::default_dark();
        theme.variant = ThemeVariant::Dark;
        theme
    }

    /// The Avalonia Fluent light theme.
    pub fn fluent_light() -> Self {
        fluent::fluent_light()
    }

    /// The Avalonia Fluent dark theme.
    pub fn fluent_dark() -> Self {
        fluent::fluent_dark()
    }

    /// Looks up a semantic color token by name, resolving aliases.
    pub fn color(&self, name: &str) -> Option<Color> {
        self.colors.resolve(name)
    }

    pub fn default_light() -> Self {
        Self::builder("light")
            .variant(ThemeVariant::Light)
            .colors(|c| {
                c.raw("white", Color::new(1.0, 1.0, 1.0, 1.0));
                c.raw("black", Color::new(0.0, 0.0, 0.0, 1.0));
                c.raw("slate.1", Color::new(0.99, 0.99, 0.99, 1.0));
                c.raw("slate.2", Color::new(0.97, 0.97, 0.98, 1.0));
                c.raw("slate.3", Color::new(0.94, 0.94, 0.95, 1.0));
                c.raw("slate.4", Color::new(0.90, 0.90, 0.91, 1.0));
                c.raw("slate.5", Color::new(0.84, 0.84, 0.86, 1.0));
                c.raw("slate.6", Color::new(0.73, 0.73, 0.75, 1.0));
                c.raw("slate.7", Color::new(0.57, 0.57, 0.59, 1.0));
                c.raw("slate.8", Color::new(0.44, 0.44, 0.46, 1.0));
                c.raw("slate.9", Color::new(0.33, 0.33, 0.35, 1.0));
                c.raw("slate.10", Color::new(0.23, 0.23, 0.24, 1.0));
                c.raw("slate.11", Color::new(0.15, 0.15, 0.16, 1.0));
                c.raw("slate.12", Color::new(0.09, 0.09, 0.10, 1.0));
                c.raw("blue.1", Color::new(0.99, 0.99, 1.0, 1.0));
                c.raw("blue.2", Color::new(0.97, 0.98, 1.0, 1.0));
                c.raw("blue.3", Color::new(0.90, 0.94, 1.0, 1.0));
                c.raw("blue.4", Color::new(0.79, 0.88, 1.0, 1.0));
                c.raw("blue.5", Color::new(0.64, 0.78, 1.0, 1.0));
                c.raw("blue.6", Color::new(0.48, 0.68, 1.0, 1.0));
                c.raw("blue.7", Color::new(0.35, 0.57, 0.97, 1.0));
                c.raw("blue.8", Color::new(0.25, 0.48, 0.98, 1.0));
                c.raw("blue.9", Color::new(0.13, 0.39, 0.94, 1.0));
                c.raw("blue.10", Color::new(0.09, 0.30, 0.82, 1.0));
                c.raw("blue.11", Color::new(0.06, 0.22, 0.60, 1.0));
                c.raw("blue.12", Color::new(0.04, 0.13, 0.37, 1.0));
                c.alias("text.primary", "slate.12");
                c.alias("text.secondary", "slate.10");
                c.alias("text.muted", "slate.7");
                c.alias("surface.canvas", "slate.1");
                c.alias("surface.panel", "slate.2");
                c.alias("surface.elevated", "white");
                c.alias("surface.sunken", "slate.3");
                c.alias("accent.solid", "blue.9");
                c.alias("accent.hover", "blue.8");
                c.alias("accent.pressed", "blue.7");
                c.alias("accent.muted", "blue.4");
                c.alias("accent.contrast", "white");
                c.alias("border.default", "slate.4");
                c.alias("border.subtle", "slate.3");
                c.alias("border.emphasis", "slate.6");
                c.alias_color("success.solid", Color::new(0.13, 0.78, 0.38, 1.0));
                c.alias_color("warning.solid", Color::new(0.98, 0.76, 0.05, 1.0));
                c.alias_color("error.solid", Color::new(0.92, 0.12, 0.12, 1.0));
            })
            .space(|s| {
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
            })
            .radii(|r| {
                r.insert("none", 0.0);
                r.insert("sm", 4.0);
                r.insert("md", 8.0);
                r.insert("lg", 12.0);
                r.insert("xl", 16.0);
                r.insert("2xl", 24.0);
                r.insert("pill", 999.0);
                r.insert("full", 999.0);
            })
            .typography(|t| {
                t.font_family(
                    "sans-serif",
                    "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
                );
                t.font_family(
                    "mono",
                    "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, monospace",
                );
                t.font_size("xs", 12.0);
                t.font_size("sm", 14.0);
                t.font_size("md", 16.0);
                t.font_size("lg", 18.0);
                t.font_size("xl", 20.0);
                t.font_size("2xl", 24.0);
                t.font_size("3xl", 30.0);
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
            })
            .shadows(|s| {
                s.insert("none", crate::property::Shadow::default());
                s.insert(
                    "sm",
                    crate::property::Shadow::new(
                        0.0,
                        1.0,
                        2.0,
                        0.0,
                        Color::new(0.0, 0.0, 0.0, 0.098),
                    ),
                );
                s.insert(
                    "md",
                    crate::property::Shadow::new(
                        0.0,
                        4.0,
                        6.0,
                        -1.0,
                        Color::new(0.0, 0.0, 0.0, 0.157),
                    ),
                );
                s.insert(
                    "lg",
                    crate::property::Shadow::new(
                        0.0,
                        10.0,
                        15.0,
                        -3.0,
                        Color::new(0.0, 0.0, 0.0, 0.196),
                    ),
                );
                s.insert(
                    "xl",
                    crate::property::Shadow::new(
                        0.0,
                        20.0,
                        25.0,
                        -5.0,
                        Color::new(0.0, 0.0, 0.0, 0.235),
                    ),
                );
                s.insert(
                    "focus",
                    crate::property::Shadow::new(
                        0.0,
                        0.0,
                        0.0,
                        3.0,
                        Color::new(0.231, 0.510, 0.965, 0.502),
                    ),
                );
                s.insert(
                    "inner",
                    crate::property::Shadow::new(
                        0.0,
                        2.0,
                        4.0,
                        0.0,
                        Color::new(0.0, 0.0, 0.0, 0.098),
                    ),
                );
            })
            .component(RecipeKey::base("button"), |b| {
                b.base(|s| {
                    s.set_color(
                        box_style::BACKGROUND,
                        Color::new(0.13, 0.39, 0.94, 1.0),
                        StylePrecedence::BaseRecipe,
                        StyleSource::Recipe,
                    );
                    s.set_color(
                        text_style::COLOR,
                        Color::new(1.0, 1.0, 1.0, 1.0),
                        StylePrecedence::BaseRecipe,
                        StyleSource::Recipe,
                    );
                    s.set_f32(
                        box_style::BORDER_RADIUS,
                        8.0,
                        StylePrecedence::BaseRecipe,
                        StyleSource::Recipe,
                    );
                    s.set_f32(
                        layout::PADDING,
                        12.0,
                        StylePrecedence::BaseRecipe,
                        StyleSource::Recipe,
                    );
                    s.set_f32(
                        text_style::FONT_SIZE,
                        16.0,
                        StylePrecedence::BaseRecipe,
                        StyleSource::Recipe,
                    );
                    s.set_f32(
                        text_style::FONT_WEIGHT,
                        500.0,
                        StylePrecedence::BaseRecipe,
                        StyleSource::Recipe,
                    );
                });
                b.variant("appearance", "filled", |s| {
                    s.set_color(
                        box_style::BACKGROUND,
                        Color::new(0.13, 0.39, 0.94, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_color(
                        text_style::COLOR,
                        Color::new(1.0, 1.0, 1.0, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                });
                b.variant("appearance", "outline", |s| {
                    s.set_color(
                        box_style::BACKGROUND,
                        Color::new(0.97, 0.97, 0.98, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_color(
                        box_style::BORDER_COLOR,
                        Color::new(0.90, 0.90, 0.91, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_f32(
                        box_style::BORDER_WIDTH,
                        1.0,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_color(
                        text_style::COLOR,
                        Color::new(0.09, 0.09, 0.10, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                });
                b.variant("appearance", "ghost", |s| {
                    s.set_color(
                        box_style::BACKGROUND,
                        Color::TRANSPARENT,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_color(
                        text_style::COLOR,
                        Color::new(0.09, 0.09, 0.10, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                });
                b.variant("appearance", "subtle", |s| {
                    s.set_color(
                        box_style::BACKGROUND,
                        Color::new(0.97, 0.97, 0.98, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_color(
                        text_style::COLOR,
                        Color::new(0.09, 0.09, 0.10, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                });
                b.variant("appearance", "link", |s| {
                    s.set_color(
                        box_style::BACKGROUND,
                        Color::TRANSPARENT,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_color(
                        text_style::COLOR,
                        Color::new(0.13, 0.39, 0.94, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                });
                b.variant("size", "small", |s| {
                    s.set_f32(
                        layout::PADDING,
                        8.0,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_f32(
                        text_style::FONT_SIZE,
                        14.0,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                });
                b.variant("size", "medium", |s| {
                    s.set_f32(
                        layout::PADDING,
                        12.0,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_f32(
                        text_style::FONT_SIZE,
                        16.0,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                });
                b.variant("size", "large", |s| {
                    s.set_f32(
                        layout::PADDING,
                        16.0,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_f32(
                        text_style::FONT_SIZE,
                        18.0,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                });
                b.state(WidgetState::new().hovered(), |s| {
                    s.set_f32(
                        box_style::OPACITY,
                        0.9,
                        StylePrecedence::StateStyle,
                        StyleSource::State,
                    );
                });
                b.state(WidgetState::new().pressed(), |s| {
                    s.set_f32(
                        box_style::OPACITY,
                        0.8,
                        StylePrecedence::StateStyle,
                        StyleSource::State,
                    );
                });
                b.state(WidgetState::new().focused(), |s| {
                    s.set_f32(
                        box_style::BORDER_WIDTH,
                        2.0,
                        StylePrecedence::StateStyle,
                        StyleSource::State,
                    );
                    s.set_color(
                        box_style::BORDER_COLOR,
                        Color::new(0.13, 0.39, 0.94, 1.0),
                        StylePrecedence::StateStyle,
                        StyleSource::State,
                    );
                });
                b.state(WidgetState::new().disabled(), |s| {
                    s.set_f32(
                        box_style::OPACITY,
                        0.5,
                        StylePrecedence::StateStyle,
                        StyleSource::State,
                    );
                    s.set_color(
                        text_style::COLOR,
                        Color::new(0.5, 0.5, 0.5, 1.0),
                        StylePrecedence::StateStyle,
                        StyleSource::State,
                    );
                });
            })
            .build()
    }

    pub fn default_dark() -> Self {
        Self::builder("dark")
            .variant(ThemeVariant::Dark)
            .colors(|c| {
                c.raw("white", Color::new(1.0, 1.0, 1.0, 1.0));
                c.raw("black", Color::new(0.0, 0.0, 0.0, 1.0));
                c.raw("slate.1", Color::new(0.09, 0.09, 0.11, 1.0));
                c.raw("slate.2", Color::new(0.13, 0.13, 0.15, 1.0));
                c.raw("slate.3", Color::new(0.18, 0.18, 0.21, 1.0));
                c.raw("slate.4", Color::new(0.23, 0.23, 0.26, 1.0));
                c.raw("slate.5", Color::new(0.30, 0.30, 0.33, 1.0));
                c.raw("slate.6", Color::new(0.42, 0.42, 0.46, 1.0));
                c.raw("slate.7", Color::new(0.57, 0.57, 0.60, 1.0));
                c.raw("slate.8", Color::new(0.71, 0.71, 0.74, 1.0));
                c.raw("slate.9", Color::new(0.80, 0.80, 0.83, 1.0));
                c.raw("slate.10", Color::new(0.87, 0.87, 0.89, 1.0));
                c.raw("slate.11", Color::new(0.92, 0.92, 0.94, 1.0));
                c.raw("slate.12", Color::new(0.97, 0.97, 0.98, 1.0));
                c.raw("blue.1", Color::new(0.04, 0.10, 0.22, 1.0));
                c.raw("blue.2", Color::new(0.06, 0.15, 0.32, 1.0));
                c.raw("blue.3", Color::new(0.10, 0.22, 0.49, 1.0));
                c.raw("blue.4", Color::new(0.17, 0.32, 0.65, 1.0));
                c.raw("blue.5", Color::new(0.27, 0.44, 0.80, 1.0));
                c.raw("blue.6", Color::new(0.37, 0.55, 0.91, 1.0));
                c.raw("blue.7", Color::new(0.47, 0.65, 0.97, 1.0));
                c.raw("blue.8", Color::new(0.57, 0.75, 1.0, 1.0));
                c.raw("blue.9", Color::new(0.65, 0.81, 1.0, 1.0));
                c.raw("blue.10", Color::new(0.74, 0.87, 1.0, 1.0));
                c.raw("blue.11", Color::new(0.85, 0.93, 1.0, 1.0));
                c.raw("blue.12", Color::new(0.95, 0.97, 1.0, 1.0));
                c.alias("text.primary", "slate.12");
                c.alias("text.secondary", "slate.10");
                c.alias("text.muted", "slate.7");
                c.alias("surface.canvas", "slate.1");
                c.alias("surface.panel", "slate.2");
                c.alias("surface.elevated", "slate.3");
                c.alias("surface.sunken", "slate.1");
                c.alias("accent.solid", "blue.9");
                c.alias("accent.hover", "blue.8");
                c.alias("accent.pressed", "blue.7");
                c.alias("accent.muted", "blue.5");
                c.alias("accent.contrast", "slate.1");
                c.alias("border.default", "slate.4");
                c.alias("border.subtle", "slate.3");
                c.alias("border.emphasis", "slate.6");
                c.alias_color("success.solid", Color::new(0.30, 0.85, 0.50, 1.0));
                c.alias_color("warning.solid", Color::new(1.0, 0.85, 0.15, 1.0));
                c.alias_color("error.solid", Color::new(1.0, 0.25, 0.25, 1.0));
            })
            .space(|s| {
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
            })
            .radii(|r| {
                r.insert("none", 0.0);
                r.insert("sm", 4.0);
                r.insert("md", 8.0);
                r.insert("lg", 12.0);
                r.insert("xl", 16.0);
                r.insert("2xl", 24.0);
                r.insert("pill", 999.0);
                r.insert("full", 999.0);
            })
            .typography(|t| {
                t.font_family(
                    "sans-serif",
                    "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
                );
                t.font_family(
                    "mono",
                    "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, monospace",
                );
                t.font_size("xs", 12.0);
                t.font_size("sm", 14.0);
                t.font_size("md", 16.0);
                t.font_size("lg", 18.0);
                t.font_size("xl", 20.0);
                t.font_size("2xl", 24.0);
                t.font_size("3xl", 30.0);
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
            })
            .shadows(|s| {
                s.insert("none", crate::property::Shadow::default());
                s.insert(
                    "sm",
                    crate::property::Shadow::new(
                        0.0,
                        1.0,
                        2.0,
                        0.0,
                        Color::new(0.0, 0.0, 0.0, 0.196),
                    ),
                );
                s.insert(
                    "md",
                    crate::property::Shadow::new(
                        0.0,
                        4.0,
                        6.0,
                        -1.0,
                        Color::new(0.0, 0.0, 0.0, 0.275),
                    ),
                );
                s.insert(
                    "lg",
                    crate::property::Shadow::new(
                        0.0,
                        10.0,
                        15.0,
                        -3.0,
                        Color::new(0.0, 0.0, 0.0, 0.314),
                    ),
                );
                s.insert(
                    "xl",
                    crate::property::Shadow::new(
                        0.0,
                        20.0,
                        25.0,
                        -5.0,
                        Color::new(0.0, 0.0, 0.0, 0.353),
                    ),
                );
                s.insert(
                    "focus",
                    crate::property::Shadow::new(
                        0.0,
                        0.0,
                        0.0,
                        3.0,
                        Color::new(0.231, 0.510, 0.965, 0.502),
                    ),
                );
                s.insert(
                    "inner",
                    crate::property::Shadow::new(
                        0.0,
                        2.0,
                        4.0,
                        0.0,
                        Color::new(0.0, 0.0, 0.0, 0.196),
                    ),
                );
            })
            .component(RecipeKey::base("button"), |b| {
                b.base(|s| {
                    s.set_color(
                        box_style::BACKGROUND,
                        Color::new(0.65, 0.81, 1.0, 1.0),
                        StylePrecedence::BaseRecipe,
                        StyleSource::Recipe,
                    );
                    s.set_color(
                        text_style::COLOR,
                        Color::new(0.09, 0.09, 0.11, 1.0),
                        StylePrecedence::BaseRecipe,
                        StyleSource::Recipe,
                    );
                    s.set_f32(
                        box_style::BORDER_RADIUS,
                        8.0,
                        StylePrecedence::BaseRecipe,
                        StyleSource::Recipe,
                    );
                    s.set_f32(
                        layout::PADDING,
                        12.0,
                        StylePrecedence::BaseRecipe,
                        StyleSource::Recipe,
                    );
                    s.set_f32(
                        text_style::FONT_SIZE,
                        16.0,
                        StylePrecedence::BaseRecipe,
                        StyleSource::Recipe,
                    );
                    s.set_f32(
                        text_style::FONT_WEIGHT,
                        500.0,
                        StylePrecedence::BaseRecipe,
                        StyleSource::Recipe,
                    );
                });
                b.variant("appearance", "filled", |s| {
                    s.set_color(
                        box_style::BACKGROUND,
                        Color::new(0.65, 0.81, 1.0, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_color(
                        text_style::COLOR,
                        Color::new(0.09, 0.09, 0.11, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                });
                b.variant("appearance", "outline", |s| {
                    s.set_color(
                        box_style::BACKGROUND,
                        Color::new(0.13, 0.13, 0.15, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_color(
                        box_style::BORDER_COLOR,
                        Color::new(0.30, 0.30, 0.33, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_f32(
                        box_style::BORDER_WIDTH,
                        1.0,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_color(
                        text_style::COLOR,
                        Color::new(0.97, 0.97, 0.98, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                });
                b.variant("appearance", "ghost", |s| {
                    s.set_color(
                        box_style::BACKGROUND,
                        Color::TRANSPARENT,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_color(
                        text_style::COLOR,
                        Color::new(0.97, 0.97, 0.98, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                });
                b.variant("appearance", "subtle", |s| {
                    s.set_color(
                        box_style::BACKGROUND,
                        Color::new(0.18, 0.18, 0.21, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_color(
                        text_style::COLOR,
                        Color::new(0.97, 0.97, 0.98, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                });
                b.variant("appearance", "link", |s| {
                    s.set_color(
                        box_style::BACKGROUND,
                        Color::TRANSPARENT,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_color(
                        text_style::COLOR,
                        Color::new(0.65, 0.81, 1.0, 1.0),
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                });
                b.variant("size", "small", |s| {
                    s.set_f32(
                        layout::PADDING,
                        8.0,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_f32(
                        text_style::FONT_SIZE,
                        14.0,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                });
                b.variant("size", "medium", |s| {
                    s.set_f32(
                        layout::PADDING,
                        12.0,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_f32(
                        text_style::FONT_SIZE,
                        16.0,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                });
                b.variant("size", "large", |s| {
                    s.set_f32(
                        layout::PADDING,
                        16.0,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                    s.set_f32(
                        text_style::FONT_SIZE,
                        18.0,
                        StylePrecedence::Variant,
                        StyleSource::Variant,
                    );
                });
                b.state(WidgetState::new().hovered(), |s| {
                    s.set_f32(
                        box_style::OPACITY,
                        0.9,
                        StylePrecedence::StateStyle,
                        StyleSource::State,
                    );
                });
                b.state(WidgetState::new().pressed(), |s| {
                    s.set_f32(
                        box_style::OPACITY,
                        0.8,
                        StylePrecedence::StateStyle,
                        StyleSource::State,
                    );
                });
                b.state(WidgetState::new().focused(), |s| {
                    s.set_f32(
                        box_style::BORDER_WIDTH,
                        2.0,
                        StylePrecedence::StateStyle,
                        StyleSource::State,
                    );
                    s.set_color(
                        box_style::BORDER_COLOR,
                        Color::new(0.65, 0.81, 1.0, 1.0),
                        StylePrecedence::StateStyle,
                        StyleSource::State,
                    );
                });
                b.state(WidgetState::new().disabled(), |s| {
                    s.set_f32(
                        box_style::OPACITY,
                        0.5,
                        StylePrecedence::StateStyle,
                        StyleSource::State,
                    );
                    s.set_color(
                        text_style::COLOR,
                        Color::new(0.5, 0.5, 0.5, 1.0),
                        StylePrecedence::StateStyle,
                        StyleSource::State,
                    );
                });
            })
            .build()
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

impl ThemeProvider for Theme {
    fn get_token(&self, scale: TokenScale, name: &str) -> Option<TokenValue> {
        match scale {
            TokenScale::Color => {
                let color = self.colors.resolve(name)?;
                Some(TokenValue::Color(color))
            }
            TokenScale::Space => {
                let space = self.space.resolve(name)?;
                Some(TokenValue::F32(space))
            }
            TokenScale::Size => {
                let size = self.sizes.resolve(name)?;
                Some(TokenValue::F32(size))
            }
            TokenScale::Radius => {
                let radius = self.radii.resolve(name)?;
                Some(TokenValue::F32(radius))
            }
            TokenScale::FontSize => {
                let size = self.typography.get_font_size(name)?;
                Some(TokenValue::F32(size))
            }
            TokenScale::FontWeight => {
                let weight = self.typography.get_font_weight(name)?;
                Some(TokenValue::F32(weight))
            }
            TokenScale::FontFamily => {
                let family = self.typography.get_font_family(name)?.to_string();
                Some(TokenValue::String(family))
            }
            TokenScale::LineHeight => {
                let height = self.typography.get_line_height(name)?;
                Some(TokenValue::F32(height))
            }
            TokenScale::LetterSpacing => {
                let spacing = self.typography.get_letter_spacing(name)?;
                Some(TokenValue::F32(spacing))
            }
            TokenScale::Shadow => {
                let shadow = self.shadows.resolve(name)?;
                Some(TokenValue::Shadow(shadow))
            }
            TokenScale::Duration => None,
        }
    }

    fn get_component_recipe(&self, key: &RecipeKey) -> Option<Arc<ComponentRecipe>> {
        self.components.get(key).cloned()
    }

    fn variant(&self) -> ThemeVariant {
        self.variant
    }
}
