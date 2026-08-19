//! raikou-widgets — the raikou component builders.
//!
//! Each component is a builder (`Button::new()...build(cx)`) that constructs
//! the equivalent native fyrox widget, wraps it in a [`Component`], and
//! registers its handlers into a [`ComponentRegistry`] for dispatch from the
//! app's message poll loop.

pub mod accordion;
pub mod box_widget;
pub mod build_cx;
pub mod button;
pub mod checkbox;
pub mod combobox;
pub mod component;
pub mod context_menu;
pub mod convert;
pub mod group;
pub mod image;
pub mod label_widget;
pub mod loading_indicator;
pub mod menu;
pub mod popover;
pub mod progress_bar;
pub mod radio;
pub mod registry;
pub mod scroll_area;
pub mod select;
pub mod slider;
pub mod stack;
pub mod step_input;
pub mod switch;
pub mod table;
pub mod tabs;
pub mod text_area;
pub mod text_input;
pub mod theme_scope;
pub mod tree;

pub use accordion::{Accordion, AccordionHandle};
pub use box_widget::{BoxHandle, BoxWidget};
pub use build_cx::BuildCx;
pub use button::{Button, ButtonHandle, ClickMode};
pub use checkbox::{Checkbox, CheckboxHandle};
pub use combobox::{Combobox, ComboboxHandle};
pub use component::{ClickEvent, Component, ComponentKind};
pub use context_menu::{
    hide_context_menu, show_context_menu, ContextMenu, ContextMenuHandle,
};
pub use convert::{to_fyrox_color, to_fyrox_gradient, to_fyrox_thickness};
pub use group::{Group, GroupHandle};
pub use image::{set_image_fit, set_image_texture, Image, ImageHandle};
pub use label_widget::{set_label_text, Label, LabelHandle};
pub use loading_indicator::{LoadingIndicator, LoadingIndicatorHandle, LoadingIndicatorMode};
pub use menu::{MenuBar, MenuBarHandle, MenuItem};
pub use popover::{hide_popover, show_popover, Popover, PopoverHandle};
pub use progress_bar::{set_progress, ProgressBar, ProgressBarHandle};
pub use radio::{Radio, RadioGroup, RadioGroupHandle, RadioHandle};
pub use registry::ComponentRegistry;
pub use scroll_area::{ScrollArea, ScrollAreaHandle};
pub use select::{Select, SelectHandle};
pub use slider::{Slider, SliderHandle};
pub use stack::{Stack, StackHandle};
pub use step_input::{StepInput, StepInputHandle};
pub use switch::{Switch, SwitchHandle};
pub use table::{Table, TableColumn, TableHandle};
pub use tabs::{Tabs, TabsHandle};
pub use text_area::{TextArea, TextAreaHandle};
pub use text_input::{TextInput, TextInputHandle};
pub use theme_scope::{ThemeScope, ThemeScopeHandle};
pub use tree::{Tree, TreeNode, TreeHandle};
