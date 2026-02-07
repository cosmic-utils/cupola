#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CropRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DragHandle {
    #[default]
    None,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
    Move,
}

#[derive(Debug, Clone, Default)]
pub struct CropSelection {
    pub region: Option<(f32, f32, f32, f32)>,
    pub is_dragging: bool,
    pub drag_handle: DragHandle,
    pub drag_start: Option<(f32, f32)>,
    pub drag_start_region: Option<(f32, f32, f32, f32)>,
}

impl CropSelection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_new_selection(&mut self, x: f32, y: f32) {
        self.region = Some((x, y, 0.0, 0.0));
        self.is_dragging = true;
        self.drag_handle = DragHandle::None;
        self.drag_start = Some((x, y));
        self.drag_start_region = None;
    }

    pub fn start_handle_drag(&mut self, handle: DragHandle, x: f32, y: f32) {
        self.is_dragging = true;
        self.drag_handle = handle;
        self.drag_start = Some((x, y));
        self.drag_start_region = self.region;
    }

    pub fn update_drag(&mut self, x: f32, y: f32, img_width: f32, img_height: f32) {
        if !self.is_dragging {
            return;
        }

        match self.drag_handle {
            DragHandle::None => {
                if let Some((start_x, start_y)) = self.drag_start {
                    let min_x = start_x.min(x).max(0.0);
                    let min_y = start_y.min(y).max(0.0);
                    let max_x = start_x.max(x).min(img_width);
                    let max_y = start_y.max(y).min(img_height);

                    self.region = Some((min_x, min_y, max_x - min_x, max_y - min_y));
                }
            }
            DragHandle::Move => {
                if let (Some((start_x, start_y)), Some((rx, ry, rw, rh))) =
                    (self.drag_start, self.drag_start_region)
                {
                    let dx = x - start_x;
                    let dy = y - start_y;
                    let new_x = (rx + dx).max(0.0).min(img_width - rw);
                    let new_y = (ry + dy).max(0.0).min(img_height - rh);
                    self.region = Some((new_x, new_y, rw, rh));
                }
            }
            _ => {
                if let Some((rx, ry, rw, rh)) = self.drag_start_region {
                    let (new_x, new_y, new_w, new_h) =
                        self.resize_region(rx, ry, rw, rh, x, y, img_width, img_height);
                    self.region = Some((new_x, new_y, new_w, new_h));
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn resize_region(
        &self,
        rx: f32,
        ry: f32,
        rw: f32,
        rh: f32,
        x: f32,
        y: f32,
        img_width: f32,
        img_height: f32,
    ) -> (f32, f32, f32, f32) {
        let right = rx + rw;
        let bottom = ry + rh;
        let x = x.max(0.0).min(img_width);
        let y = y.max(0.0).min(img_height);

        match self.drag_handle {
            DragHandle::TopLeft => {
                let new_x = x.min(right - 10.0);
                let new_y = y.min(bottom - 10.0);

                (new_x, new_y, right - new_x, bottom - new_y)
            }
            DragHandle::TopRight => {
                let new_right = x.max(rx + 10.0);
                let new_y = y.min(bottom - 10.0);

                (rx, new_y, new_right - rx, bottom - new_y)
            }
            DragHandle::BottomLeft => {
                let new_x = x.min(right - 10.0);
                let new_bottom = y.max(ry + 10.0);

                (new_x, ry, right - new_x, new_bottom - ry)
            }
            DragHandle::BottomRight => {
                let new_right = x.max(rx + 10.0);
                let new_bottom = y.max(ry + 10.0);

                (rx, ry, new_right - rx, new_bottom - ry)
            }
            DragHandle::Top => {
                let new_y = y.min(bottom - 10.0);

                (rx, new_y, rw, bottom - new_y)
            }
            DragHandle::Bottom => {
                let new_bottom = y.max(ry + 10.0);

                (rx, ry, rw, new_bottom - ry)
            }
            DragHandle::Left => {
                let new_x = x.min(right - 10.0);

                (new_x, ry, right - new_x, rh)
            }
            DragHandle::Right => {
                let new_right = x.max(rx + 10.0);

                (rx, ry, new_right - rx, rh)
            }
            _ => (rx, ry, rw, rh),
        }
    }

    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.drag_start = None;
        self.drag_start_region = None;
    }

    pub fn to_crop_region(&self) -> Option<CropRegion> {
        self.region.and_then(|(x, y, w, h)| {
            if w > 1.0 && h > 1.0 {
                Some(CropRegion {
                    x: x as u32,
                    y: y as u32,
                    width: w as u32,
                    height: h as u32,
                })
            } else {
                None
            }
        })
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn has_selection(&self) -> bool {
        self.region
            .map(|(_, _, w, h)| w > 1.0 && h > 1.0)
            .unwrap_or(false)
    }
}
