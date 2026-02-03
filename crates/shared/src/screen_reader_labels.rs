use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenReaderLabel {
    pub template: String,
    pub variables: std::collections::HashMap<String, String>,
}

impl ScreenReaderLabel {
    pub fn new<S: Into<String>>(template: S) -> Self {
        Self {
            template: template.into(),
            variables: std::collections::HashMap::new(),
        }
    }

    pub fn with_variable<S: Into<String>>(mut self, key: &str, value: S) -> Self {
        self.variables.insert(key.to_string(), value.into());
        self
    }

    pub fn render(&self, _context: &std::collections::HashMap<String, String>) -> String {
        let mut rendered = self.template.clone();

        for (key, value) in &self.variables {
            rendered = rendered.replace(&format!("{{{{{}}}}}", key), value);
        }

        rendered
    }

    pub fn add_filename_variable(mut self, filename: &str) -> Self {
        self.variables
            .insert("filename".to_string(), filename.to_string());
        self
    }

    pub fn add_dimension_variables(mut self, width: u32, height: u32) -> Self {
        self.variables
            .insert("width".to_string(), width.to_string());
        self.variables
            .insert("height".to_string(), height.to_string());
        self
    }

    pub fn add_aspect_ratio_variable(mut self, aspect_ratio: f64) -> Self {
        self.variables
            .insert("aspect_ratio".to_string(), aspect_ratio.to_string());
        self
    }

    pub fn add_file_size_variable(mut self, size_bytes: u64) -> Self {
        self.variables
            .insert("file_size".to_string(), size_bytes.to_string());
        self
    }

    pub fn get_default_templates() -> Vec<(String, String)> {
        vec![
            (
                "portrait".to_string(),
                "Portrait image ({{width}}×{{height}}) - {{filename}}".to_string(),
            ),
            (
                "landscape".to_string(),
                "Landscape image ({{width}}×{{height}}) - {{filename}}".to_string(),
            ),
            (
                "square".to_string(),
                "Square image ({{width}}×{{height}}) - {{filename}}".to_string(),
            ),
            (
                "loading".to_string(),
                "Loading thumbnail for {{filename}}...".to_string(),
            ),
            (
                "error".to_string(),
                "Failed to generate thumbnail for {{filename}}".to_string(),
            ),
            (
                "ready".to_string(),
                "{{filename}} thumbnail ready".to_string(),
            ),
        ]
    }

    pub fn get_label_for_aspect(aspect_ratio: f64) -> String {
        if (aspect_ratio - 1.0).abs() < f64::EPSILON {
            "square".to_string()
        } else if aspect_ratio < 1.0 {
            "portrait".to_string()
        } else {
            "landscape".to_string()
        }
    }

    pub fn render_with_metadata<S: Into<String>>(
        &self,
        filename: S,
        width: u32,
        height: u32,
        _size_bytes: u64,
        aspect_ratio: f64,
    ) -> String {
        let aspect_label = Self::get_label_for_aspect(aspect_ratio);
        let templates = Self::get_default_templates();
        let template = templates
            .iter()
            .find(|(template_name, _)| template_name == &aspect_label)
            .map(|(_, template)| template.as_str())
            .unwrap_or("Image ({width}×{height})");

        let mut context = std::collections::HashMap::new();
        context.insert("filename".to_string(), filename.into());
        context.insert("width".to_string(), width.to_string());
        context.insert("height".to_string(), height.to_string());

        let mut rendered = template.to_string();
        for (key, value) in &context {
            rendered = rendered.replace(&format!("{{{{{}}}}}", key), value);
        }

        rendered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_reader_label_creation() {
        let label = ScreenReaderLabel::new("Test image ({}×{})");
        assert_eq!(label.template, "Test image ({}×{})");
        assert!(label.variables.is_empty());
    }

    #[test]
    fn test_screen_reader_label_variables() {
        let mut label =
            ScreenReaderLabel::new("Portrait image ({{width}}×{{height}}) - {{filename}}")
                .add_filename_variable("image.jpg")
                .add_dimension_variables(1920, 1080);

        let context = std::collections::HashMap::from([
            ("filename".to_string(), "image.jpg".to_string()),
            ("width".to_string(), "1920".to_string()),
            ("height".to_string(), "1080".to_string()),
        ]);

        let rendered = label.render(&context);
        assert_eq!(rendered, "Portrait image (1920×1080) - image.jpg");
    }

    #[test]
    fn test_aspect_label_determination() {
        assert_eq!(ScreenReaderLabel::get_label_for_aspect(0.9), "portrait");
        assert_eq!(ScreenReaderLabel::get_label_for_aspect(1.1), "landscape");
        assert_eq!(ScreenReaderLabel::get_label_for_aspect(1.0), "square");
        assert_eq!(ScreenReaderLabel::get_label_for_aspect(1.5), "landscape");
    }

    #[test]
    fn test_template_rendering() {
        let label = ScreenReaderLabel::new("Landscape image ({{width}}×{{height}}) - {{filename}}")
            .add_filename_variable("photo.png")
            .add_dimension_variables(4000, 3000);

        let context = std::collections::HashMap::from([
            ("filename".to_string(), "photo.png".to_string()),
            ("width".to_string(), "4000".to_string()),
            ("height".to_string(), "3000".to_string()),
        ]);

        let rendered = label.render(&context);
        assert_eq!(rendered, "Landscape image (4000×3000) - photo.png");
    }
}
