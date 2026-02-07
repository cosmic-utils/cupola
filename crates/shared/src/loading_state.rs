use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LoadingState {
    #[default]
    Loading,
    Ready,
    Error(String),
    AccessibilityMode,
}

impl LoadingState {
    pub fn is_loading(&self) -> bool {
        matches!(self, LoadingState::Loading)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, LoadingState::Ready)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, LoadingState::Error(_))
    }

    pub fn is_accessibility_mode(&self) -> bool {
        matches!(self, LoadingState::AccessibilityMode)
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            LoadingState::Error(msg) => Some(msg),
            _ => None,
        }
    }

    pub fn transition_to_ready(self) -> Self {
        Self::Ready
    }

    pub fn transition_to_error(self, error: String) -> Self {
        Self::Error(error)
    }

    pub fn transition_to_accessibility_mode(self) -> Self {
        Self::AccessibilityMode
    }

    pub fn transition_to_loading(self) -> Self {
        Self::Loading
    }

    pub fn can_retry(&self) -> bool {
        matches!(
            self,
            LoadingState::Error(_) | LoadingState::AccessibilityMode
        )
    }

    pub fn requires_user_action(&self) -> bool {
        matches!(
            self,
            LoadingState::Error(_) | LoadingState::AccessibilityMode
        )
    }
}

impl fmt::Display for LoadingState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadingState::Loading => write!(f, "Loading"),
            LoadingState::Ready => write!(f, "Ready"),
            LoadingState::Error(msg) => write!(f, "Error: {}", msg),
            LoadingState::AccessibilityMode => write!(f, "Accessibility Mode"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadingEvent {
    StartLoading,
    LoadComplete,
    LoadError(String),
    ToggleAccessibilityMode,
    RetryRequested,
}

impl LoadingEvent {
    pub fn apply_to_state(self, current_state: LoadingState) -> LoadingState {
        match (self, current_state) {
            (LoadingEvent::StartLoading, _) => LoadingState::Loading,
            (LoadingEvent::LoadComplete, _) => LoadingState::Ready,
            (LoadingEvent::LoadError(msg), _) => LoadingState::Error(msg),
            (LoadingEvent::ToggleAccessibilityMode, LoadingState::AccessibilityMode) => {
                LoadingState::Ready
            }
            (LoadingEvent::ToggleAccessibilityMode, _) => LoadingState::AccessibilityMode,
            (LoadingEvent::RetryRequested, LoadingState::Error(_)) => LoadingState::Loading,
            (LoadingEvent::RetryRequested, state) => state, // No change for other states
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loading_state_properties() {
        let loading = LoadingState::Loading;
        assert!(loading.is_loading());
        assert!(!loading.is_ready());
        assert!(!loading.is_error());
        assert!(!loading.requires_user_action());

        let ready = LoadingState::Ready;
        assert!(!ready.is_loading());
        assert!(ready.is_ready());
        assert!(!ready.is_error());
        assert!(!ready.requires_user_action());

        let error = LoadingState::Error("Test error".to_string());
        assert!(!error.is_loading());
        assert!(!error.is_ready());
        assert!(error.is_error());
        assert!(error.requires_user_action());
        assert_eq!(error.error_message(), Some("Test error"));
    }

    #[test]
    fn test_state_transitions() {
        let mut state = LoadingState::Loading;

        state = state.transition_to_ready();
        assert_eq!(state, LoadingState::Ready);

        state = state.transition_to_error("Failed".to_string());
        assert_eq!(state, LoadingState::Error("Failed".to_string()));

        state = state.transition_to_accessibility_mode();
        assert_eq!(state, LoadingState::AccessibilityMode);

        state = state.transition_to_loading();
        assert_eq!(state, LoadingState::Loading);
    }

    #[test]
    fn test_loading_events() {
        let state = LoadingState::Loading;

        // Test loading complete
        let new_state = LoadingEvent::LoadComplete.apply_to_state(state.clone());
        assert_eq!(new_state, LoadingState::Ready);

        // Test loading error
        let new_state = LoadingEvent::LoadError("Failed".to_string()).apply_to_state(state.clone());
        assert_eq!(new_state, LoadingState::Error("Failed".to_string()));

        // Test accessibility mode toggle
        let new_state = LoadingEvent::ToggleAccessibilityMode.apply_to_state(state);
        assert_eq!(new_state, LoadingState::AccessibilityMode);
    }

    #[test]
    fn test_retry_capabilities() {
        let loading = LoadingState::Loading;
        assert!(!loading.can_retry());

        let ready = LoadingState::Ready;
        assert!(!ready.can_retry());

        let error = LoadingState::Error("Failed".to_string());
        assert!(error.can_retry());

        let accessibility = LoadingState::AccessibilityMode;
        assert!(accessibility.can_retry());
    }

    #[test]
    fn test_display_formatting() {
        let loading = LoadingState::Loading;
        assert_eq!(loading.to_string(), "Loading");

        let ready = LoadingState::Ready;
        assert_eq!(ready.to_string(), "Ready");

        let error = LoadingState::Error("Network error".to_string());
        assert_eq!(error.to_string(), "Error: Network error");

        let accessibility = LoadingState::AccessibilityMode;
        assert_eq!(accessibility.to_string(), "Accessibility Mode");
    }

    #[test]
    fn test_default_state() {
        let default = LoadingState::default();
        assert_eq!(default, LoadingState::Loading);
    }
}
