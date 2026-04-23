use crate::error::SurfaceError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendKind {
    Metal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackendSelection {
    kind: BackendKind,
}

impl BackendSelection {
    pub fn for_current_platform() -> Result<Self, SurfaceError> {
        #[cfg(target_os = "macos")]
        {
            Ok(Self {
                kind: BackendKind::Metal,
            })
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(SurfaceError::UnsupportedPlatform)
        }
    }

    pub fn kind(&self) -> BackendKind {
        self.kind
    }
}

#[cfg(test)]
mod tests {
    use super::{BackendKind, BackendSelection};

    #[test]
    fn backend_selection_is_explicit() {
        let selection = BackendSelection::for_current_platform();

        #[cfg(target_os = "macos")]
        assert_eq!(selection.unwrap().kind(), BackendKind::Metal);

        #[cfg(not(target_os = "macos"))]
        assert!(selection.is_err());
    }
}
