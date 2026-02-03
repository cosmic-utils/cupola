use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavState {
    pub current_index: usize,
    pub paths: Vec<PathBuf>,
    pub total_count: usize,
}

impl Default for NavState {
    fn default() -> Self {
        Self {
            current_index: 0,
            paths: Vec::new(),
            total_count: 0,
        }
    }
}

impl NavState {
    pub fn new(paths: Vec<PathBuf>) -> Self {
        let total_count = paths.len();
        Self {
            current_index: 0,
            paths,
            total_count,
        }
    }

    pub fn current(&self) -> Option<&PathBuf> {
        self.paths.get(self.current_index)
    }

    pub fn go_next(&mut self) -> Option<&PathBuf> {
        if self.current_index + 1 < self.total_count {
            self.current_index += 1;
            self.current()
        } else {
            None
        }
    }

    pub fn go_previous(&mut self) -> Option<&PathBuf> {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.current()
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
}
