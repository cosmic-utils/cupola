//! Shared grid layout utilities

use cosmic::iced::{Padding, Rectangle};

/// Configuration for grid layout calculation
#[derive(Debug, Clone)]
pub struct GridConfig {
    pub item_width: f32,
    pub column_spacing: f32,
    pub row_spacing: f32,
    pub min_columns: usize,
    pub max_columns: Option<usize>,
    pub padding: Padding,
}

/// Result of grid layout calculation
#[derive(Debug, Clone)]
pub struct GridMetrics {
    pub cols: usize,
    pub rows: usize,
    pub row_height: f32,
}

/// Calculate number of columns that fit in available width
pub fn calculate_columns(
    available_width: f32,
    item_width: f32,
    column_spacing: f32,
    min_columns: usize,
    max_columns: Option<usize>,
    item_count: usize,
) -> usize {
    if available_width <= 0.0 || item_width <= 0.0 {
        return min_columns;
    }

    let cols =
        ((available_width + column_spacing) / (item_width + column_spacing)).floor() as usize;

    cols.max(min_columns)
        .min(max_columns.unwrap_or(usize::MAX))
        .min(item_count)
        .max(1)
}

/// Calculate the scroll offset needed to bring an item into view
pub fn calculate_scroll_offset(
    target_index: usize,
    cols: usize,
    row_height: f32,
    row_spacing: f32,
    padding_top: f32,
    viewport_top: f32,
    viewport_height: f32,
) -> Option<f32> {
    if cols == 0 || row_height <= 0.0 {
        return None;
    }

    let row = target_index / cols;
    let item_top = padding_top + (row as f32 * (row_height + row_spacing));
    let item_bottom = item_top + row_height;

    let viewport_bottom = viewport_top + viewport_height;

    // Check if item is already fully visible
    if item_top >= viewport_top && item_bottom <= viewport_bottom {
        return None;
    }

    // Scroll to bring item into view
    if item_top < viewport_top {
        // Item is above viewport - scroll up
        Some(item_top)
    } else {
        // Item is below viewport - scroll down
        Some(item_bottom - viewport_height)
    }
}

/// Calculate the index of an item at a given position
pub fn item_at_position(
    position: (f32, f32),
    cols: usize,
    rows: usize,
    item_width: f32,
    row_height: f32,
    column_spacing: f32,
    row_spacing: f32,
    padding: Padding,
    item_count: usize,
) -> Option<usize> {
    let (x, y) = position;

    // Adjust for padding
    let x = x - padding.left;
    let y = y - padding.top;

    if x < 0.0 || y < 0.0 {
        return None;
    }

    // Calculate cell size including spacing
    let cell_width = item_width + column_spacing;
    let cell_height = row_height + row_spacing;

    let col = (x / cell_width).floor() as usize;
    let row = (y / cell_height).floor() as usize;

    if col >= cols || row >= rows {
        return None;
    }

    // Check if position is within the item (not in spacing)
    let x_in_cell = x - (col as f32 * cell_width);
    let y_in_cell = y - (row as f32 * cell_height);

    if x_in_cell > item_width || y_in_cell > row_height {
        return None; // In spacing area
    }

    let index = row * cols + col;
    if index < item_count {
        Some(index)
    } else {
        None
    }
}

/// Calculate centered position for an image within a cell
pub fn calculate_centered_image_bounds(
    cell_bounds: Rectangle,
    image_width: f32,
    image_height: f32,
) -> Rectangle {
    if image_width <= 0.0 || image_height <= 0.0 {
        return cell_bounds;
    }

    let cell_aspect = cell_bounds.width / cell_bounds.height;
    let image_aspect = image_width / image_height;

    let (scaled_width, scaled_height) = if image_aspect > cell_aspect {
        // Image is wider - fit to width
        let w = cell_bounds.width;
        let h = w / image_aspect;
        (w, h)
    } else {
        // Image is taller - fit to height
        let h = cell_bounds.height;
        let w = h * image_aspect;
        (w, h)
    };

    // Center in cell
    let x = cell_bounds.x + (cell_bounds.width - scaled_width) / 2.0;
    let y = cell_bounds.y + (cell_bounds.height - scaled_height) / 2.0;

    Rectangle::new(
        cosmic::iced::Point::new(x, y),
        cosmic::iced::Size::new(scaled_width, scaled_height),
    )
}
