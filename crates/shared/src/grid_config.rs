use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridCellConfiguration {
    pub cell_size: u32,
    pub spacing: u32,
    pub padding: u32,
    pub background_color: String,
    pub keyboard_focus_index: Option<usize>,
    pub accessibility_mode: bool,
}

impl GridCellConfiguration {
    pub fn new(cell_size: u32) -> Self {
        Self {
            cell_size,
            spacing: 8,                              // Default 8px spacing
            padding: 4,                              // Default 4px padding
            background_color: "#000000".to_string(), // Default black background
            keyboard_focus_index: None,
            accessibility_mode: false,
        }
    }

    pub fn with_spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn with_padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_background_color(mut self, color: String) -> Self {
        self.background_color = color;
        self
    }

    pub fn with_keyboard_focus(mut self, index: Option<usize>) -> Self {
        self.keyboard_focus_index = index;
        self
    }

    pub fn with_accessibility_mode(mut self, enabled: bool) -> Self {
        self.accessibility_mode = enabled;
        self
    }

    pub fn inner_size(&self) -> u32 {
        self.cell_size.saturating_sub(2 * self.padding)
    }

    pub fn total_cell_size(&self) -> u32 {
        self.cell_size + self.spacing
    }

    pub fn can_fit_in_width(&self, container_width: u32) -> usize {
        if self.total_cell_size() == 0 {
            return 0;
        }
        (container_width / self.total_cell_size()) as usize
    }

    pub fn grid_dimensions(&self, total_items: usize, container_width: u32) -> (usize, usize) {
        let cols = self.can_fit_in_width(container_width);
        let rows = if cols == 0 {
            0
        } else {
            (total_items + cols - 1) / cols
        };
        (cols, rows)
    }

    pub fn cell_position(&self, index: usize, container_width: u32) -> (u32, u32) {
        let (cols, _) = self.grid_dimensions(index + 1, container_width);
        let row = index / cols;
        let col = index % cols;

        let x = (col as u32) * self.total_cell_size();
        let y = (row as u32) * self.total_cell_size();

        (x, y)
    }

    pub fn cell_bounds(&self, index: usize, container_width: u32) -> (u32, u32, u32, u32) {
        let (x, y) = self.cell_position(index, container_width);
        (x, y, x + self.cell_size, y + self.cell_size)
    }

    pub fn inner_bounds(&self, index: usize, container_width: u32) -> (u32, u32, u32, u32) {
        let (x1, y1, x2, y2) = self.cell_bounds(index, container_width);
        (
            x1 + self.padding,
            y1 + self.padding,
            x2 - self.padding,
            y2 - self.padding,
        )
    }

    pub fn is_valid(&self) -> bool {
        self.cell_size >= 256 && // Must be at least thumbnail size
        self.spacing <= self.cell_size &&
        self.padding * 2 < self.cell_size &&
        !self.background_color.is_empty()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.cell_size < 256 {
            return Err("Cell size must be at least 256px".to_string());
        }

        if self.spacing > self.cell_size {
            return Err("Spacing cannot exceed cell size".to_string());
        }

        if self.padding * 2 >= self.cell_size {
            return Err("Padding is too large for cell size".to_string());
        }

        if self.background_color.is_empty() {
            return Err("Background color cannot be empty".to_string());
        }

        Ok(())
    }

    pub fn move_focus_up(
        &mut self,
        current_index: usize,
        total_items: usize,
        container_width: u32,
    ) -> Option<usize> {
        let (cols, _) = self.grid_dimensions(total_items, container_width);
        if cols == 0 {
            return None;
        }

        let current_row = current_index / cols;
        let current_col = current_index % cols;

        if current_row > 0 {
            let new_index = (current_row - 1) * cols + current_col;
            if new_index < total_items {
                self.keyboard_focus_index = Some(new_index);
                return Some(new_index);
            }
        }
        None
    }

    pub fn move_focus_down(
        &mut self,
        current_index: usize,
        total_items: usize,
        container_width: u32,
    ) -> Option<usize> {
        let (cols, rows) = self.grid_dimensions(total_items, container_width);
        if cols == 0 {
            return None;
        }

        let current_row = current_index / cols;
        let current_col = current_index % cols;

        if current_row < rows - 1 {
            let new_index = (current_row + 1) * cols + current_col;
            if new_index < total_items {
                self.keyboard_focus_index = Some(new_index);
                return Some(new_index);
            }
        }
        None
    }

    pub fn move_focus_left(
        &mut self,
        current_index: usize,
        _total_items: usize,
        container_width: u32,
    ) -> Option<usize> {
        let (cols, _) = self.grid_dimensions(1, container_width);
        if cols == 0 {
            return None;
        }

        let current_col = current_index % cols;
        if current_col > 0 {
            let new_index = current_index - 1;
            self.keyboard_focus_index = Some(new_index);
            return Some(new_index);
        }
        None
    }

    pub fn move_focus_right(
        &mut self,
        current_index: usize,
        total_items: usize,
        container_width: u32,
    ) -> Option<usize> {
        let (cols, _) = self.grid_dimensions(total_items, container_width);
        if cols == 0 {
            return None;
        }

        let current_col = current_index % cols;
        if current_col < cols - 1 && current_index + 1 < total_items {
            let new_index = current_index + 1;
            self.keyboard_focus_index = Some(new_index);
            return Some(new_index);
        }
        None
    }
}

impl Default for GridCellConfiguration {
    fn default() -> Self {
        Self::new(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_cell_configuration_creation() {
        let config = GridCellConfiguration::new(256);
        assert_eq!(config.cell_size, 256);
        assert_eq!(config.spacing, 8);
        assert_eq!(config.padding, 4);
        assert_eq!(config.background_color, "#000000");
        assert_eq!(config.keyboard_focus_index, None);
        assert!(!config.accessibility_mode);
    }

    #[test]
    fn test_grid_cell_configuration_builder() {
        let config = GridCellConfiguration::new(300)
            .with_spacing(10)
            .with_padding(8)
            .with_background_color("#ffffff".to_string())
            .with_keyboard_focus(Some(5))
            .with_accessibility_mode(true);

        assert_eq!(config.cell_size, 300);
        assert_eq!(config.spacing, 10);
        assert_eq!(config.padding, 8);
        assert_eq!(config.background_color, "#ffffff");
        assert_eq!(config.keyboard_focus_index, Some(5));
        assert!(config.accessibility_mode);
    }

    #[test]
    fn test_grid_dimensions() {
        let config = GridCellConfiguration::new(256).with_spacing(8);
        let container_width = 800;

        // 800 / (256 + 8) = 3 columns
        assert_eq!(config.can_fit_in_width(container_width), 3);

        // 10 items, 3 columns = 4 rows (3+3+3+1)
        let (cols, rows) = config.grid_dimensions(10, container_width);
        assert_eq!(cols, 3);
        assert_eq!(rows, 4);
    }

    #[test]
    fn test_cell_position() {
        let config = GridCellConfiguration::new(256).with_spacing(8);
        let container_width = 800;

        // Index 0: (0, 0)
        let (x, y) = config.cell_position(0, container_width);
        assert_eq!(x, 0);
        assert_eq!(y, 0);

        // Index 1: (264, 0) - cell_size + spacing
        let (x, y) = config.cell_position(1, container_width);
        assert_eq!(x, 264);
        assert_eq!(y, 0);

        // Index 3: (0, 264) - new row
        let (x, y) = config.cell_position(3, container_width);
        assert_eq!(x, 0);
        assert_eq!(y, 264);
    }

    #[test]
    fn test_focus_navigation() {
        let mut config = GridCellConfiguration::new(256).with_spacing(8);
        let container_width = 800;
        let total_items = 10;

        // Set initial focus to index 4
        config.keyboard_focus_index = Some(4);

        // Move left (from index 4 to 3)
        let new_focus = config.move_focus_left(4, total_items, container_width);
        assert_eq!(new_focus, Some(3));
        assert_eq!(config.keyboard_focus_index, Some(3));

        // Try to move left from index 3 (should fail - at column 0 of row 1)
        let new_focus = config.move_focus_left(3, total_items, container_width);
        assert_eq!(new_focus, None);

        // Move up (from index 2, row 0, col 2 to row -1 - should fail)
        let new_focus = config.move_focus_up(2, total_items, container_width);
        assert_eq!(new_focus, None);

        // Move down from index 2 to index 5
        let new_focus = config.move_focus_down(2, total_items, container_width);
        assert_eq!(new_focus, Some(5));
    }

    #[test]
    fn test_validation() {
        let valid = GridCellConfiguration::new(256);
        assert!(valid.is_valid());
        assert!(valid.validate().is_ok());

        let invalid = GridCellConfiguration::new(100); // Too small
        assert!(!invalid.is_valid());
        assert!(invalid.validate().is_err());

        let mut invalid = GridCellConfiguration::new(256);
        invalid.padding = 200; // Too large
        assert!(!invalid.is_valid());
        assert!(invalid.validate().is_err());
    }
}
