use fyrox::{
    asset::{
        event::{ResourceEvent, ResourceEventSender},
        Resource,
    },
    core::{pool::Handle, watcher::FileSystemWatcher},
    engine::Engine,
    gui::{
        font::{BOLD_ITALIC, BUILT_IN_BOLD, BUILT_IN_FONT, BUILT_IN_ITALIC},
        UserInterface,
    },
};
use std::{
    path::PathBuf,
    sync::{mpsc, Arc},
    time::Duration,
};

/// Watches a `.ui` file and swaps the live `UserInterface` out of the resource
/// whenever the file changes on disk. The heavy lifting (file watching, reloading,
/// event broadcast) is done by the engine's resource manager; this type only
/// orchestrates it and hands the reloaded UI to the caller.
pub struct HotReload {
    pub ui_resource: Resource<UserInterface>,
    event_rx: mpsc::Receiver<ResourceEvent>,
    #[allow(dead_code)]
    event_subscription: Handle<ResourceEventSender>,
    pub applied: bool,
}

/// What a [`HotReload::pump`] cycle concluded.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ApplyKind {
    /// Nothing to do.
    None,
    /// The UI finished its (first) load from disk.
    Load,
    /// The UI was reloaded because the file changed.
    Reload,
}

impl HotReload {
    /// Creates the watcher for the directory containing `ui_path` (when `watch` is
    /// true), requests the UI resource and subscribes to reload events. The registry
    /// must be loaded before any UI resource is requested; we do that here.
    pub fn setup(
        engine: &mut Engine,
        ui_path: PathBuf,
        watch: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        engine.resource_manager.update_or_load_registry();

        // The exported `.ui` references the built-in default font by a fixed UUID.
        // For that reference to resolve on load, the built-in fonts must be
        // registered in the resource manager; otherwise the loader treats the UUID
        // as a file path and panics ("Font reader must be initialized!").
        for font in [BUILT_IN_FONT.clone(), BUILT_IN_BOLD.clone(), BUILT_IN_ITALIC.clone(), BOLD_ITALIC.clone()] {
            engine.resource_manager.state().register_built_in_resource(font);
        }

        let ui_resource: Resource<UserInterface> = engine.resource_manager.request(&ui_path);

        if watch {
            let watch_dir = ui_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            let watcher = FileSystemWatcher::new(&watch_dir, Duration::from_millis(500))?;
            engine.resource_manager.state().set_watcher(Some(watcher));
        }

        let (tx, rx) = mpsc::channel();
        let event_subscription = engine.resource_manager.state().event_broadcaster.add(tx);

        Ok(Self {
            ui_resource,
            event_rx: rx,
            event_subscription,
            applied: false,
        })
    }

    /// Pulls filesystem events (which triggers reloads in the resource manager) and
    /// reports whether the live UI should be swapped.
    ///
    /// The initial load fires a `Loaded` event; every later change fires `Reloaded`.
    /// `Loaded` only applies when nothing has been applied yet — the watcher can
    /// re-fire `Loaded` for the file we wrote during setup, and that must not count
    /// as a reload.
    pub fn pump(&mut self, engine: &mut Engine) -> ApplyKind {
        engine.resource_manager.state().process_filesystem_events();

        let mut result = ApplyKind::None;
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                ResourceEvent::Loaded(res) => {
                    // `UntypedResource` is a newtype over the shared header Arc, so a
                    // pointer comparison identifies "this is our resource" precisely.
                    if Arc::ptr_eq(&res.0, &self.ui_resource.as_ref().0) && !self.applied {
                        result = ApplyKind::Load;
                    }
                }
                ResourceEvent::Reloaded(res) => {
                    if Arc::ptr_eq(&res.0, &self.ui_resource.as_ref().0) {
                        result = ApplyKind::Reload;
                    }
                }
                _ => {}
            }
        }

        // The initial load may have completed before we subscribed to the
        // broadcaster; fall back to checking the resource state directly.
        if result == ApplyKind::None
            && !self.applied
            && self.ui_resource.data_ref().as_loaded_ref().is_some()
        {
            result = ApplyKind::Load;
        }

        result
    }

    /// Moves the loaded `UserInterface` out of the resource, leaving a default UI in
    /// its place. Only call this when [`Self::pump`] returned true.
    pub fn take_ui(&mut self) -> Option<UserInterface> {
        let result = {
            let mut guard = self.ui_resource.data_ref();
            match guard.as_loaded_mut() {
                Some(ui) => Some(std::mem::replace(ui, UserInterface::default())),
                None => None,
            }
        };
        if result.is_some() {
            self.applied = true;
        }
        result
    }
}