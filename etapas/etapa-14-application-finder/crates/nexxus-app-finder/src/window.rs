//! Backend-neutral host contract for the Finder's compact own window.

use nexxus_ui::LogicalSize;

use crate::FinderAction;

/// Engineering defaults that realize the normative "small/compact" Finder
/// without coupling Stage 14 to X11 or Wayland window objects.
pub const FINDER_PREFERRED_SIZE: LogicalSize = LogicalSize::new(560.0, 360.0);
pub const FINDER_MINIMUM_SIZE: LogicalSize = LogicalSize::new(420.0, 240.0);

#[derive(Clone, Debug, PartialEq)]
pub struct FinderWindowSpec {
    pub title: String,
    pub preferred_size: LogicalSize,
    pub minimum_size: LogicalSize,
}

impl FinderWindowSpec {
    pub fn compact() -> Self {
        Self {
            title: "Application Finder".to_owned(),
            preferred_size: FINDER_PREFERRED_SIZE,
            minimum_size: FINDER_MINIMUM_SIZE,
        }
    }
}

impl Default for FinderWindowSpec {
    fn default() -> Self {
        Self::compact()
    }
}

/// Requests emitted to the session/backend host. Showing also requests focus so
/// text input can begin immediately after `Super+F`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FinderWindowRequest {
    ShowAndFocus,
    Hide,
}

impl FinderWindowRequest {
    /// Converts only visibility-changing controller actions into host work.
    pub fn from_action(action: &FinderAction) -> Option<Self> {
        match action {
            FinderAction::Opened => Some(Self::ShowAndFocus),
            FinderAction::Closed | FinderAction::Launch(_) => Some(Self::Hide),
            FinderAction::None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_surface_is_compact_and_resizable_above_minimum() {
        let spec = FinderWindowSpec::default();
        assert!(spec.preferred_size.width >= spec.minimum_size.width);
        assert!(spec.preferred_size.height >= spec.minimum_size.height);
        assert!(spec.preferred_size.width < 800.0);
        assert!(spec.preferred_size.height < 600.0);
    }

    #[test]
    fn open_and_close_actions_map_to_host_window_requests() {
        assert_eq!(
            FinderWindowRequest::from_action(&FinderAction::Opened),
            Some(FinderWindowRequest::ShowAndFocus)
        );
        assert_eq!(
            FinderWindowRequest::from_action(&FinderAction::Closed),
            Some(FinderWindowRequest::Hide)
        );
    }
}
