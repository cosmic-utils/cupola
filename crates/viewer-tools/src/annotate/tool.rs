pub mod crop;
pub mod highlighter;
pub mod pen;
pub mod shapes;
pub mod text;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PenMode {
    #[default]
    Freeform,
    Bezier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransformSubTool {
    #[default]
    Resize,
    Skew,
    Rotate,
}

impl TransformSubTool {
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Resize => "image-resize-symbolic",
            Self::Skew => "object-skew-symbolic",
            Self::Rotate => "object-rotate-right-symbolic",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CropRatio {
    #[default]
    Free,
    Square,
    FourThree,
    SixteenNine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnnotateTool {
    #[default]
    Select,
    Move,
    Transform,
    Pen,
    Highlighter,
    Text,
    Rectangle,
    Ellipse,
    Arrow,
    Line,
    Star,
    Polygon,
    Crop,
}

impl AnnotateTool {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Move => "Move",
            Self::Transform => "Transform",
            Self::Pen => "Pen",
            Self::Highlighter => "Highlighter",
            Self::Rectangle => "Rectangle",
            Self::Ellipse => "Ellipse",
            Self::Line => "Line",
            Self::Arrow => "Arrow",
            Self::Star => "Star",
            Self::Polygon => "Polygon",
            Self::Text => "Text",
            Self::Crop => "Crop",
        }
    }

    pub fn shortcut_key(&self) -> &'static str {
        match self {
            Self::Select => "V",
            Self::Move => "M",
            Self::Transform => "T",
            Self::Pen => "P",
            Self::Highlighter => "H",
            Self::Rectangle => "R",
            Self::Ellipse => "E",
            Self::Line => "L",
            Self::Arrow => "A",
            Self::Star => "S",
            Self::Polygon => "G",
            Self::Text => "X",
            Self::Crop => "C",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Select => "object-select-symbolic",
            Self::Move => "edit-move-symbolic",
            Self::Transform => "image-resize-symbolic",
            Self::Pen => "pen-symbolic",
            Self::Highlighter => "text-highlight-symbolic",
            Self::Text => "insert-text-symbolic",
            Self::Rectangle => "insert-rectangle-symbolic",
            Self::Ellipse => "insert-ellipse-symbolic",
            Self::Arrow => "insert-arrow-symbolic",
            Self::Line => "insert-line-symbolic",
            Self::Star => "insert-star-symbolic",
            Self::Polygon => "insert-polygon-symbolic",
            Self::Crop => "image-crop-symbolic",
        }
    }

    pub fn draw_tools() -> &'static [AnnotateTool] {
        &[AnnotateTool::Pen, AnnotateTool::Highlighter]
    }

    pub fn shape_tools() -> &'static [AnnotateTool] {
        &[
            AnnotateTool::Rectangle,
            AnnotateTool::Ellipse,
            AnnotateTool::Arrow,
            AnnotateTool::Line,
            AnnotateTool::Star,
            AnnotateTool::Polygon,
        ]
    }
}
