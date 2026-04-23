use std::sync::{Arc, Mutex};

use accesskit::{ActionRequest, ActivationHandler, DeactivationHandler, TreeUpdate};

#[derive(Clone, Debug, Default)]
pub struct AccessibilityState {
    tree_update: Arc<Mutex<Option<TreeUpdate>>>,
    action_requests: Arc<Mutex<Vec<ActionRequest>>>,
}

impl AccessibilityState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_tree_update(&self, update: TreeUpdate) {
        *self
            .tree_update
            .lock()
            .expect("accessibility tree mutex poisoned") = Some(update);
    }

    pub fn current_tree_update(&self) -> Option<TreeUpdate> {
        self.tree_update
            .lock()
            .expect("accessibility tree mutex poisoned")
            .clone()
    }

    pub fn push_action_request(&self, request: ActionRequest) {
        self.action_requests
            .lock()
            .expect("accessibility action mutex poisoned")
            .push(request);
    }

    pub fn drain_action_requests(&self) -> Vec<ActionRequest> {
        self.action_requests
            .lock()
            .expect("accessibility action mutex poisoned")
            .drain(..)
            .collect()
    }
}

pub struct ActivationBridge {
    state: AccessibilityState,
}

impl ActivationBridge {
    pub fn new(state: AccessibilityState) -> Self {
        Self { state }
    }
}

impl ActivationHandler for ActivationBridge {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        self.state.current_tree_update()
    }
}

pub struct ActionBridge {
    state: AccessibilityState,
}

impl ActionBridge {
    pub fn new(state: AccessibilityState) -> Self {
        Self { state }
    }
}

impl accesskit::ActionHandler for ActionBridge {
    fn do_action(&mut self, request: ActionRequest) {
        self.state.push_action_request(request);
    }
}

pub struct DeactivationBridge;

impl DeactivationHandler for DeactivationBridge {
    fn deactivate_accessibility(&mut self) {}
}
