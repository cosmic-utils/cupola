use cosmic::iced::Color;

/// Preset annotation colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnnotateColor(pub Color);

impl AnnotateColor {
    pub fn presets() -> Vec<AnnotateColor> {
        vec![
            AnnotateColor(Color::WHITE),
            AnnotateColor(Color::from_rgb(1.0, 0.0, 0.0)),
            AnnotateColor(Color::from_rgb(1.0, 0.65, 0.0)),
            AnnotateColor(Color::from_rgb(0.0, 1.0, 0.0)),
            AnnotateColor(Color::from_rgb(0.0, 0.0, 1.0)),
            AnnotateColor(Color::BLACK),
        ]
    }
}

impl Default for AnnotateColor {
    fn default() -> Self {
        Self(Color::BLACK)
    }
}
