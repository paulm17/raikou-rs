use crate::screens::virtual_list::VirtualList;
use fyrox::{
    core::{color::Color, pool::Handle, uuid::Uuid},
    gui::{
        border::BorderBuilder,
        brush::Brush,
        button::{ButtonBuilder, ButtonMessage},
        formatted_text::WrapMode,
        grid::{Column, GridBuilder, Row},
        message::UiMessage,
        stack_panel::StackPanelBuilder,
        tab_control::{TabControlBuilder, TabDefinition},
        text::{TextBuilder, TextMessage},
        text_box::{TextBoxBuilder, TextCommitMode},
        toggle::{ToggleButtonBuilder, ToggleButtonMessage},
        widget::{WidgetBuilder, WidgetMessage},
        HorizontalAlignment, Orientation, Thickness, UiNode, UserInterface,
    },
};

/// The first real interactive screen of the app: a small form built entirely in
/// code. Widgets are mutated exclusively via messages (`MessageDirection::ToWidget`
/// on `ui.send`), never by writing widget fields directly.
pub struct MainScreen {
    pub name_field: Handle<UiNode>,
    pub greet_button: Handle<UiNode>,
    pub counter_button: Handle<UiNode>,
    pub clear_button: Handle<UiNode>,
    pub counter_label: Handle<UiNode>,
    pub greeting_label: Handle<UiNode>,
    pub virtual_list: VirtualList,
    /// The top-level panel. main.rs sizes it to the window on resize so the
    /// layout is bounded to the window instead of growing to content size.
    pub root: Handle<UiNode>,
    /// Toggle button handles for the 3D scene controls (found by name after a
    /// hot-reload).
    pub toggle_spin: Handle<UiNode>,
    pub toggle_orbit: Handle<UiNode>,
    pub toggle_always: Handle<UiNode>,
    /// 3D scene flags, mirrored from the toggles. main.rs pushes these into the
    /// `Scene3D` each frame.
    pub spin: bool,
    pub orbit: bool,
    pub always_spin: bool,
    pub name: String,
    pub clicks: u32,
}

/// Number of logical rows the dense list holds. Far more than the row pool that
/// actually exists in the scene — this is what virtualization is for.
pub const LIST_TOTAL_ROWS: usize = 200;
pub const LIST_ROW_HEIGHT: f32 = 22.0;

pub fn build(ui: &mut UserInterface) -> MainScreen {
    // Build the virtualized list first: it needs its own `build_ctx` borrow, which
    // must not overlap with the `ctx` borrow used for the rest of the form below.
    let virtual_list = VirtualList::build(ui, 360.0, 260.0, LIST_TOTAL_ROWS, LIST_ROW_HEIGHT);
    let list_scroll_viewer: Handle<UiNode> = virtual_list.scroll_viewer;

    let ctx = &mut ui.build_ctx();

    let title: Handle<UiNode> = TextBuilder::new(
        WidgetBuilder::new()
            .with_name("title")
            .with_margin(Thickness::uniform(4.0)),
    )
    .with_text("raikou — form demo")
    .with_font_size(28.0.into())
    .build(ctx)
    .transmute();

    // Grid layout: 2 columns (fixed labels, stretchable controls), 3 auto rows.
    let name_label: Handle<UiNode> = TextBuilder::new(
        WidgetBuilder::new()
            .on_row(0)
            .on_column(0)
            .with_margin(Thickness::uniform(6.0)),
    )
    .with_text("Name:")
    .build(ctx)
    .transmute();

    let name_field: Handle<UiNode> = TextBoxBuilder::new(
        WidgetBuilder::new()
            .with_name("name_field")
            .on_row(0)
            .on_column(1)
            .with_height(28.0)
            .with_margin(Thickness::uniform(4.0)),
    )
    .with_text("Alice")
    .with_text_commit_mode(TextCommitMode::Immediate)
    .build(ctx)
    .transmute();

    let counter_label: Handle<UiNode> = TextBuilder::new(
        WidgetBuilder::new()
            .on_row(1)
            .on_column(0)
            .with_margin(Thickness::uniform(6.0)),
    )
    .with_text("Counter")
    .build(ctx)
    .transmute();

    let counter_value: Handle<UiNode> = TextBuilder::new(
        WidgetBuilder::new()
            .with_name("counter_label")
            .on_row(1)
            .on_column(1)
            .with_margin(Thickness::uniform(6.0)),
    )
    .with_text("clicks: 0")
    .build(ctx)
    .transmute();

    let greeting_label: Handle<UiNode> = TextBuilder::new(
        WidgetBuilder::new()
            .on_row(2)
            .on_column(0)
            .with_margin(Thickness::uniform(6.0)),
    )
    .with_text("Greeting")
    .build(ctx)
    .transmute();

    let greeting_value: Handle<UiNode> = TextBuilder::new(
        WidgetBuilder::new()
            .with_name("greeting_label")
            .on_row(2)
            .on_column(1)
            .with_margin(Thickness::uniform(6.0)),
    )
    .with_text("Hello, !")
    .with_font_size(18.0.into())
    .with_wrap(WrapMode::Word)
    .build(ctx)
    .transmute();

    let form_grid = GridBuilder::new(
        WidgetBuilder::new()
            .with_child(name_label)
            .with_child(name_field)
            .with_child(counter_label)
            .with_child(counter_value)
            .with_child(greeting_label)
            .with_child(greeting_value),
    )
    .add_column(Column::strict(120.0))
    .add_column(Column::stretch())
    .add_row(Row::auto())
    .add_row(Row::auto())
    .add_row(Row::auto())
    .build(ctx);

    let form_panel = BorderBuilder::new(
        WidgetBuilder::new()
            .with_background(Brush::Solid(Color::opaque(40, 40, 46)).into())
            .with_margin(Thickness::uniform(8.0))
            .with_child(form_grid),
    )
    .with_corner_radius(6.0.into())
    .build(ctx);

    let greet_button: Handle<UiNode> = ButtonBuilder::new(
        WidgetBuilder::new()
            .with_name("greet_button")
            .with_width(80.0)
            .with_height(32.0)
            .with_margin(Thickness::uniform(4.0)),
    )
    .with_text("Greet")
    .build(ctx)
    .transmute();

    let counter_button: Handle<UiNode> = ButtonBuilder::new(
        WidgetBuilder::new()
            .with_name("counter_button")
            .with_width(90.0)
            .with_height(32.0)
            .with_margin(Thickness::uniform(4.0)),
    )
    .with_text("Click me")
    .build(ctx)
    .transmute();

    let clear_button: Handle<UiNode> = ButtonBuilder::new(
        WidgetBuilder::new()
            .with_name("clear_button")
            .with_width(80.0)
            .with_height(32.0)
            .with_margin(Thickness::uniform(4.0)),
    )
    .with_text("Clear")
    .build(ctx)
    .transmute();

    let buttons = StackPanelBuilder::new(
        WidgetBuilder::new()
            .with_horizontal_alignment(HorizontalAlignment::Right)
            .with_child(greet_button)
            .with_child(counter_button)
            .with_child(clear_button),
    )
    .with_orientation(Orientation::Horizontal)
    .build(ctx);

    // 3D scene controls: three toggles mirroring the Scene3D flags.
    let spin_text: Handle<UiNode> =
        TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(4.0)))
            .with_text("Rotate cube")
            .build(ctx)
            .transmute();
    let orbit_text: Handle<UiNode> =
        TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(4.0)))
            .with_text("Orbit camera")
            .build(ctx)
            .transmute();
    let always_text: Handle<UiNode> =
        TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(4.0)))
            .with_text("Always spin")
            .build(ctx)
            .transmute();

    let toggle_spin: Handle<UiNode> = ToggleButtonBuilder::new(
        WidgetBuilder::new()
            .with_name("toggle_spin")
            .with_width(140.0)
            .with_height(28.0)
            .with_margin(Thickness::uniform(2.0)),
    )
    .with_toggled(false)
    .with_content(spin_text)
    .build(ctx)
    .transmute();

    let toggle_orbit: Handle<UiNode> = ToggleButtonBuilder::new(
        WidgetBuilder::new()
            .with_name("toggle_orbit")
            .with_width(140.0)
            .with_height(28.0)
            .with_margin(Thickness::uniform(2.0)),
    )
    .with_toggled(false)
    .with_content(orbit_text)
    .build(ctx)
    .transmute();

    let toggle_always: Handle<UiNode> = ToggleButtonBuilder::new(
        WidgetBuilder::new()
            .with_name("toggle_always")
            .with_width(140.0)
            .with_height(28.0)
            .with_margin(Thickness::uniform(2.0)),
    )
    .with_toggled(false)
    .with_content(always_text)
    .build(ctx)
    .transmute();

    let form_content: Handle<UiNode> = StackPanelBuilder::new(
        WidgetBuilder::new()
            .with_margin(Thickness::uniform(8.0))
            .with_child(form_panel)
            .with_child(buttons),
    )
    .build(ctx)
    .transmute();

    // List tab content: header + the virtualized dense list.
    let list_header: Handle<UiNode> =
        TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(4.0)))
            .with_text(format!("Dense list — {LIST_TOTAL_ROWS} rows"))
            .with_font_size(16.0.into())
            .build(ctx)
            .transmute();

    let list_content: Handle<UiNode> = StackPanelBuilder::new(
        WidgetBuilder::new()
            .with_margin(Thickness::uniform(8.0))
            .with_child(list_header)
            .with_child(list_scroll_viewer),
    )
    .build(ctx)
    .transmute();

    // 3D tab content: the scene controls. The 3D scene renders full-window BEHIND
    // the UI, so this tab's content must stay transparent for it to show through
    // (see make_tab_backdrop_transparent).
    let scene_content: Handle<UiNode> = StackPanelBuilder::new(
        WidgetBuilder::new()
            .with_margin(Thickness::uniform(8.0))
            .with_child(toggle_spin)
            .with_child(toggle_orbit)
            .with_child(toggle_always),
    )
    .build(ctx)
    .transmute();

    let mut tab = |text: &str| -> Handle<UiNode> {
        TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(6.0)))
            .with_text(text)
            .build(ctx)
            .transmute()
    };

    let tabs: Handle<UiNode> =
        TabControlBuilder::new(WidgetBuilder::new().with_name("tabs").with_height(420.0))
            .with_tab(TabDefinition {
                uuid: Uuid::new_v4(),
                header: tab("Form"),
                content: form_content,
                can_be_closed: false,
                user_data: None,
            })
            .with_tab(TabDefinition {
                uuid: Uuid::new_v4(),
                header: tab("List"),
                content: list_content,
                can_be_closed: false,
                user_data: None,
            })
            .with_tab(TabDefinition {
                uuid: Uuid::new_v4(),
                header: tab("3D"),
                content: scene_content,
                can_be_closed: false,
                user_data: None,
            })
            .build(ctx)
            .transmute();

    let root = StackPanelBuilder::new(
        WidgetBuilder::new()
            .with_name("root")
            .with_margin(Thickness::uniform(12.0))
            .with_child(title)
            .with_child(tabs),
    )
    .build(ctx)
    .transmute();

    // The TabControl's inner backdrop border is built with an opaque dark brush
    // that would hide the 3D scene; make it fully transparent so the scene shows
    // through behind the tab content.
    MainScreen::make_tab_backdrop_transparent(ui, tabs);

    MainScreen {
        name_field,
        greet_button,
        counter_button,
        clear_button,
        counter_label: counter_value,
        greeting_label: greeting_value,
        virtual_list,
        root,
        toggle_spin,
        toggle_orbit,
        toggle_always,
        spin: false,
        orbit: false,
        always_spin: false,
        name: "Alice".to_string(),
        clicks: 0,
    }
}

impl MainScreen {
    /// Locates a widget by the name it was built with. The name is the only link
    /// between the code and a UI that was authored or edited as a `.ui` file.
    fn find_node(ui: &UserInterface, name: &str) -> Handle<UiNode> {
        for (handle, node) in ui.nodes().pair_iter() {
            if node.name() == name {
                return handle;
            }
        }
        Handle::NONE
    }

    /// Rebuilds a `MainScreen` from a UI that was loaded from a `.ui` file. Runtime
    /// state (`name`, `clicks`) is passed in so a hot-reload can preserve it across
    /// the swap.
    pub fn from_loaded_ui(ui: &mut UserInterface, name: &str, clicks: u32) -> MainScreen {
        let name_field = Self::find_node(ui, "name_field");
        let greet_button = Self::find_node(ui, "greet_button");
        let counter_button = Self::find_node(ui, "counter_button");
        let clear_button = Self::find_node(ui, "clear_button");
        let counter_label = Self::find_node(ui, "counter_label");
        let greeting_label = Self::find_node(ui, "greeting_label");
        let root = Self::find_node(ui, "root");
        let tabs = Self::find_node(ui, "tabs");
        let toggle_spin = Self::find_node(ui, "toggle_spin");
        let toggle_orbit = Self::find_node(ui, "toggle_orbit");
        let toggle_always = Self::find_node(ui, "toggle_always");
        let list = Self::find_node(ui, "list");

        let virtual_list = VirtualList::from_loaded(ui, list);

        let screen = MainScreen {
            name_field,
            greet_button,
            counter_button,
            clear_button,
            counter_label,
            greeting_label,
            virtual_list,
            root,
            toggle_spin,
            toggle_orbit,
            toggle_always,
            spin: false,
            orbit: false,
            always_spin: false,
            name: name.to_string(),
            clicks,
        };
        MainScreen::make_tab_backdrop_transparent(ui, tabs);
        screen
    }

    /// The TabControl wraps its content in an inner border built with an opaque
    /// dark brush. That backdrop would hide the 3D scene, so make it fully
    /// transparent to let the scene show through behind the tab content.
    fn make_tab_backdrop_transparent(ui: &mut UserInterface, tabs: Handle<UiNode>) {
        let backdrop = ui
            .nodes()
            .try_get(tabs)
            .ok()
            .and_then(|tab| tab.children().first())
            .copied();
        if let Some(backdrop) = backdrop {
            ui.send(
                backdrop,
                WidgetMessage::Background(Brush::Solid(Color::from_rgba(0, 0, 0, 0)).into()),
            );
        }
    }

    /// Pushes the runtime state into the widgets via messages. Called after a
    /// hot-reload swap so the fresh widgets reflect the state that was preserved.
    pub fn push_state(&self, ui: &mut UserInterface) {
        ui.send(self.name_field, TextMessage::Text(self.name.clone()));
        ui.send(
            self.counter_label,
            TextMessage::Text(format!("clicks: {}", self.clicks)),
        );
        ui.send(
            self.greeting_label,
            TextMessage::Text(format!("Hello, {}!", self.name)),
        );
    }

    pub fn handle_ui_message(&mut self, message: &UiMessage, ui: &mut UserInterface) {
        if let Some(ButtonMessage::Click) = message.data() {
            let destination = message.destination();
            if destination == self.greet_button {
                ui.send(
                    self.greeting_label,
                    TextMessage::Text(format!("Hello, {}!", self.name)),
                );
            } else if destination == self.counter_button {
                self.clicks += 1;
                ui.send(
                    self.counter_label,
                    TextMessage::Text(format!("clicks: {}", self.clicks)),
                );
            } else if destination == self.clear_button {
                self.name.clear();
                self.clicks = 0;
                ui.send(self.name_field, TextMessage::Text(String::new()));
                ui.send(
                    self.counter_label,
                    TextMessage::Text("clicks: 0".to_string()),
                );
                ui.send(
                    self.greeting_label,
                    TextMessage::Text("Hello, !".to_string()),
                );
            }
        }

        if message.destination() == self.name_field {
            // Respond to Text in BOTH directions: real typing arrives FromWidget,
            // programmatic sets (e.g. Clear) arrive ToWidget.
            if let Some(TextMessage::Text(text)) = message.data() {
                self.name = text.clone();
                ui.send(
                    self.greeting_label,
                    TextMessage::Text(format!("Hello, {}!", text)),
                );
            }
        }

        // 3D scene toggles. The ToggleButton emits Toggled(bool) FromWidget on
        // mouse-up. Update the flags either way.
        if message.destination() == self.toggle_spin {
            if let Some(ToggleButtonMessage::Toggled(on)) = message.data() {
                self.spin = *on;
            }
        } else if message.destination() == self.toggle_orbit {
            if let Some(ToggleButtonMessage::Toggled(on)) = message.data() {
                self.orbit = *on;
            }
        } else if message.destination() == self.toggle_always {
            if let Some(ToggleButtonMessage::Toggled(on)) = message.data() {
                self.always_spin = *on;
            }
        }
    }

    /// Per-frame update. Currently just re-syncs the virtualized list with the
    /// current scroll position.
    pub fn update(&mut self, ui: &mut UserInterface) {
        self.virtual_list.refresh(ui);
    }
}
