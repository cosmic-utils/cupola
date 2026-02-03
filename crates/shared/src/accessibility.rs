use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AriaRole {
    Image,
    Group,
    Listbox,
    Option,
    Application,
    Dialog,
    Alert,
    Status,
    ProgressIndicator,
    Button,
    Link,
    Navigation,
}

impl Default for AriaRole {
    fn default() -> Self {
        Self::Image
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessibilityInfo {
    pub role: AriaRole,
    pub label: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
    pub properties: std::collections::HashMap<String, String>,
    pub live_region: bool,
    pub atomic: bool,
    pub relevant: bool,
    pub busy: bool,
    pub disabled: bool,
    pub expanded: bool,
    pub col_index: Option<usize>,
    pub col_span: Option<usize>,
    pub row_index: Option<usize>,
    pub setsize: Option<usize>,
}

impl AccessibilityInfo {
    pub fn new(role: AriaRole) -> Self {
        Self {
            role,
            label: None,
            description: None,
            state: None,
            properties: std::collections::HashMap::new(),
            live_region: false,
            atomic: false,
            relevant: true,
            busy: false,
            disabled: false,
            expanded: false,
            col_index: None,
            col_span: None,
            row_index: None,
            setsize: None,
        }
    }

    pub fn image() -> Self {
        Self::new(AriaRole::Image)
    }

    pub fn with_label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_state<S: Into<String>>(mut self, state: S) -> Self {
        self.state = Some(state.into());
        self
    }

    pub fn with_property(mut self, key: &str, value: &str) -> Self {
        self.properties.insert(key.to_string(), value.to_string());
        self
    }

    pub fn make_live_region(mut self) -> Self {
        self.live_region = true;
        self
    }

    pub fn make_atomic(mut self) -> Self {
        self.atomic = true;
        self
    }

    pub fn make_relevant(mut self) -> Self {
        self.relevant = true;
        self
    }

    pub fn make_busy(mut self) -> Self {
        self.busy = true;
        self
    }

    pub fn make_disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub fn make_expanded(mut self) -> Self {
        self.expanded = true;
        self
    }

    pub fn with_grid_position(mut self, col: usize, row: usize) -> Self {
        self.col_index = Some(col);
        self.row_index = Some(row);
        self
    }

    pub fn with_listbox_size(mut self, setsize: usize) -> Self {
        self.setsize = Some(setsize);
        self
    }

    pub fn get_role_description(&self) -> &'static str {
        match self.role {
            AriaRole::Image => "A graphical representation that can be used to convey meaning",
            AriaRole::Group => {
                "A set of user interface objects which are not intended to be included in the page summary"
            }
            AriaRole::Listbox => {
                "A widget that allows the user to select one or more items from a list of options"
            }
            AriaRole::Option => "A selectable choice",
            AriaRole::Application => "A region of the page designated as a distinct landmark",
            AriaRole::Dialog => {
                "A dialog is a window designed to interrupt the current processing and ask the user for input"
            }
            AriaRole::Alert => "A message with important, and often time-sensitive, information",
            AriaRole::Status => {
                "A region that provides up-to-date information about the status of the current context"
            }
            AriaRole::ProgressIndicator => {
                "An element that indicates that a task is in progress and that it may take a long time to complete"
            }
            AriaRole::Button => "A clickable element that allows the user to trigger an action",
            AriaRole::Link => "A hyperlink that can be followed to navigate to another location",
            AriaRole::Navigation => {
                "A collection of elements that allow the user to navigate the interface"
            }
        }
    }

    pub fn generate_aria_attributes(&self) -> String {
        let mut attrs = vec![format!("role={}", self.get_role_description())];

        if let Some(ref label) = self.label {
            attrs.push(format!("aria-label=\"{}\"", label));
        }

        if let Some(ref description) = self.description {
            attrs.push(format!("aria-description=\"{}\"", description));
        }

        if let Some(ref _state) = self.state {
            attrs.push(format!(
                "aria-live=\"{}\"",
                if self.busy { "true" } else { "false" }
            ));
            attrs.push(format!(
                "aria-busy=\"{}\"",
                if self.busy { "true" } else { "false" }
            ));
            attrs.push(format!(
                "aria-expanded=\"{}\"",
                if self.expanded { "true" } else { "false" }
            ));
        }

        for (key, value) in &self.properties {
            attrs.push(format!("aria-{}=\"{}\"", key, value));
        }

        if self.live_region {
            attrs.push("aria-live=\"polite\"".to_string());
        }

        if self.atomic {
            attrs.push("aria-atomic=\"true\"".to_string());
        }

        if self.relevant {
            attrs.push("aria-relevant=\"true\"".to_string());
        }

        if self.disabled {
            attrs.push("aria-disabled=\"true\"".to_string());
        }

        if let Some(col) = self.col_index {
            attrs.push(format!("aria-colindex=\"{}\"", col));
        }

        if let Some(row) = self.row_index {
            attrs.push(format!("aria-rowindex=\"{}\"", row));
        }

        if let Some(setsize) = self.setsize {
            attrs.push(format!("aria-setsize=\"{}\"", setsize));
        }

        attrs.join(" ")
    }

    pub fn get_aria_label(&self) -> String {
        if let Some(ref label) = self.label {
            label.clone()
        } else {
            self.get_default_label()
        }
    }

    fn get_default_label(&self) -> String {
        match self.role {
            AriaRole::Image => "Image".to_string(),
            AriaRole::Group => "Thumbnail group".to_string(),
            AriaRole::Listbox => "Thumbnail options".to_string(),
            AriaRole::Option => "Thumbnail option".to_string(),
            AriaRole::Application => "Thumbnail viewer".to_string(),
            AriaRole::Dialog => "Thumbnail generation dialog".to_string(),
            AriaRole::Alert => "Thumbnail status".to_string(),
            AriaRole::Status => "Thumbnail processing status".to_string(),
            AriaRole::ProgressIndicator => "Thumbnail loading".to_string(),
            AriaRole::Button => "Thumbnail action".to_string(),
            AriaRole::Link => "Thumbnail link".to_string(),
            AriaRole::Navigation => "Thumbnail navigation".to_string(),
        }
    }
}
