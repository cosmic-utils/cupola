//! FlexGrid widget - A responsive CSS Grid layout with optional integrated scrolling
//!
//! # Example
//! ```rust
//! flex_grid(cells)
//!     .item_width(100.0)
//!     .spacing(8)
//!     .scrollable(Id::new("my-grid"))
//!     .scroll_to_item(focused_index)
//!     .on_scroll(|vp| Message::Scroll(vp))
//!     .on_layout_changed(|cols, row_height| Message::LayoutChanged(cols, row_height))
//!     .into_element()
//! ```

use std::{cell::Cell, f32};

use cosmic::{
    Element, Renderer,
    iced::{
        Length, Padding, Point, Rectangle, Size,
        advanced::{
            Clipboard, Layout, Shell, Widget,
            layout::{Limits, Node},
            overlay, renderer as iced_renderer,
            widget::{Id, Operation, Tree},
        },
        event::{self, Event},
        mouse::{self, Cursor},
        widget::scrollable::Viewport,
    },
    widget::{container, scrollable},
};
use taffy::{
    Dimension, Display, GridPlacement, JustifyItems, LengthPercentage, Size as TaffySize, Style,
    TaffyTree, prelude::fr,
};

/// Scroll request calculated by FlexGrid
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollRequest {
    pub offset_y: f32,
}

/// A responsive grid layout builder that can optionally include scrolling.
///
/// Use `into_element()` to build the final widget.
pub struct FlexGrid<'a, M> {
    children: Vec<Element<'a, M>>,
    padding: Padding,
    column_spacing: u16,
    row_spacing: u16,
    width: Length,
    height: Length,
    item_width: f32,
    min_columns: usize,
    max_columns: Option<usize>,

    // Scrolling support
    scrollable_id: Option<Id>,
    scroll_to_index: Option<usize>,
    on_scroll: Option<Box<dyn Fn(Viewport) -> M + 'a>>,

    // Layout reporting - now includes optional scroll request
    on_layout_changed: Option<Box<dyn Fn(usize, f32, Option<ScrollRequest>) -> M + 'a>>,
}

/// Creates a new FlexGrid builder with the given children.
pub fn flex_grid<'a, M>(children: Vec<Element<'a, M>>) -> FlexGrid<'a, M> {
    FlexGrid {
        children,
        padding: Padding::ZERO,
        column_spacing: 0,
        row_spacing: 0,
        width: Length::Fill,
        height: Length::Shrink,
        item_width: 100.0,
        min_columns: 1,
        max_columns: None,
        scrollable_id: None,
        scroll_to_index: None,
        on_scroll: None,
        on_layout_changed: None,
    }
}

impl<'a, M> FlexGrid<'a, M> {
    /// Sets the width of each item in the grid.
    pub fn item_width(mut self, width: f32) -> Self {
        self.item_width = width;
        self
    }

    /// Sets the horizontal spacing between columns.
    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self
    }

    /// Sets the vertical spacing between rows.
    pub fn row_spacing(mut self, spacing: u16) -> Self {
        self.row_spacing = spacing;
        self
    }

    /// Sets both column and row spacing.
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self.row_spacing = spacing;
        self
    }

    /// Sets the minimum number of columns.
    pub fn min_columns(mut self, min: usize) -> Self {
        self.min_columns = min.max(1);
        self
    }

    /// Sets the maximum number of columns.
    pub fn max_columns(mut self, max: usize) -> Self {
        self.max_columns = Some(max);
        self
    }

    /// Sets the padding around the grid content.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the grid.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the grid.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Enables scrolling with the given ID.
    ///
    /// When enabled, the grid will be wrapped in a scrollable container.
    pub fn scrollable(mut self, id: Id) -> Self {
        self.scrollable_id = Some(id);
        self
    }

    /// Scrolls to make the item at the given index visible.
    ///
    /// Requires `scrollable()` to be set. The scroll position will be
    /// reported via `on_layout_changed` callback.
    pub fn scroll_to_item(mut self, index: usize) -> Self {
        self.scroll_to_index = Some(index);
        self
    }

    /// Sets a callback for scroll events.
    ///
    /// Requires `scrollable()` to be set.
    pub fn on_scroll<F>(mut self, f: F) -> Self
    where
        F: Fn(Viewport) -> M + 'a,
    {
        self.on_scroll = Some(Box::new(f));
        self
    }

    /// Sets a callback that fires when the layout changes.
    ///
    /// Reports (columns, row_height, scroll_request) whenever layout changes
    /// or a scroll is needed. The caller should issue the scroll command
    /// if scroll_request is Some.
    pub fn on_layout_changed<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, f32, Option<ScrollRequest>) -> M + 'a,
    {
        self.on_layout_changed = Some(Box::new(f));
        self
    }

    /// Builds the final element.
    ///
    /// If `scrollable()` was called, wraps the grid in a scrollable container.
    pub fn into_element(self) -> Element<'a, M>
    where
        M: Clone + 'static,
    {
        let inner = FlexGridInner {
            children: self.children,
            padding: self.padding,
            column_spacing: self.column_spacing,
            row_spacing: self.row_spacing,
            width: self.width,
            height: self.height,
            item_width: self.item_width,
            min_columns: self.min_columns,
            max_columns: self.max_columns,
            on_layout_changed: self.on_layout_changed,
            last_layout: Cell::new((0, 0)),
            scroll_to_index: self.scroll_to_index,
        };

        if let Some(id) = self.scrollable_id {
            let mut scroll = scrollable(container(inner).width(Length::Fill))
                .id(id)
                .width(Length::Fill)
                .height(Length::Fill);

            if let Some(on_scroll) = self.on_scroll {
                scroll = scroll.on_scroll(on_scroll);
            }

            scroll.into()
        } else {
            inner.into()
        }
    }
}

/// The inner widget that implements the actual grid layout.
struct FlexGridInner<'a, M> {
    children: Vec<Element<'a, M>>,
    padding: Padding,
    column_spacing: u16,
    row_spacing: u16,
    width: Length,
    height: Length,
    item_width: f32,
    min_columns: usize,
    max_columns: Option<usize>,
    on_layout_changed: Option<Box<dyn Fn(usize, f32, Option<ScrollRequest>) -> M + 'a>>,
    last_layout: Cell<(usize, u32)>, // (cols, row_height as bits)
    scroll_to_index: Option<usize>,
}

impl<'a, M> FlexGridInner<'a, M> {
    /// Calculate column count for a given available width
    fn calculate_columns(&self, available_width: f32) -> usize {
        let spacing = self.column_spacing as f32;
        let item_width = self.item_width;

        if available_width <= 0.0 || item_width <= 0.0 {
            return self.min_columns;
        }

        let cols = ((available_width + spacing) / (item_width + spacing)).floor() as usize;

        cols.max(self.min_columns)
            .min(self.max_columns.unwrap_or(usize::MAX))
            .min(self.children.len())
            .max(1)
    }
}

impl<'a, M: Clone + 'static> Widget<M, cosmic::Theme, Renderer> for FlexGridInner<'a, M> {
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(&mut self.children);
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        if self.children.is_empty() {
            return Node::new(Size::ZERO);
        }

        let limits = limits.width(self.width).height(self.height);
        let max_size = limits.max();
        let available_width = max_size.width - self.padding.horizontal();

        let cols = self.calculate_columns(available_width);
        let rows = (self.children.len() + cols - 1) / cols;

        // Layout children to get their sizes
        let child_limits = Limits::new(Size::ZERO, Size::new(self.item_width, f32::INFINITY));

        let mut child_nodes: Vec<Node> = tree
            .children
            .iter_mut()
            .zip(self.children.iter())
            .map(|(child_tree, child)| {
                child
                    .as_widget()
                    .layout(child_tree, renderer, &child_limits)
            })
            .collect();

        // Build taffy tree for grid layout
        let mut taffy: TaffyTree<()> = TaffyTree::new();

        let grid_style = Style {
            display: Display::Grid,
            grid_template_columns: vec![fr(1.0); cols],
            grid_template_rows: vec![fr(1.0); rows],
            gap: TaffySize {
                width: LengthPercentage::length(self.column_spacing as f32),
                height: LengthPercentage::length(self.row_spacing as f32),
            },
            justify_items: Some(JustifyItems::Center),
            size: TaffySize {
                width: Dimension::length(available_width),
                height: Dimension::auto(),
            },
            ..Default::default()
        };

        let grid_node = taffy.new_with_children(grid_style, &[]).unwrap();

        // Add children to taffy grid
        let mut taffy_children = Vec::with_capacity(self.children.len());
        for (idx, node) in child_nodes.iter().enumerate() {
            let row = (idx / cols) as i16;
            let col = (idx % cols) as i16;

            let child_style = Style {
                grid_row: taffy::Line {
                    start: GridPlacement::Line((row + 1).into()),
                    end: GridPlacement::Auto,
                },
                grid_column: taffy::Line {
                    start: GridPlacement::Line((col + 1).into()),
                    end: GridPlacement::Auto,
                },
                size: TaffySize {
                    width: Dimension::length(node.size().width),
                    height: Dimension::length(node.size().height),
                },
                ..Default::default()
            };

            let child_taffy = taffy.new_leaf(child_style).unwrap();
            taffy_children.push(child_taffy);
        }

        for child in &taffy_children {
            taffy.add_child(grid_node, *child).unwrap();
        }

        // Compute layout
        taffy
            .compute_layout(
                grid_node,
                TaffySize {
                    width: taffy::AvailableSpace::Definite(available_width),
                    height: taffy::AvailableSpace::MaxContent,
                },
            )
            .unwrap();

        // Extract positions from taffy
        for (idx, taffy_child) in taffy_children.iter().enumerate() {
            let layout = taffy.layout(*taffy_child).unwrap();
            child_nodes[idx] = child_nodes[idx].clone().move_to(Point::new(
                layout.location.x + self.padding.left,
                layout.location.y + self.padding.top,
            ));
        }

        let grid_layout = taffy.layout(grid_node).unwrap();
        let content_size = Size::new(
            grid_layout.size.width + self.padding.horizontal(),
            grid_layout.size.height + self.padding.vertical(),
        );

        let final_size = limits.resolve(self.width, self.height, content_size);

        Node::with_children(final_size, child_nodes)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        for ((child, state), layout) in self
            .children
            .iter()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child
                .as_widget()
                .operate(state, layout, renderer, operation);
        }
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) -> event::Status {
        // Calculate current columns based on layout bounds
        let available_width = layout.bounds().width - self.padding.horizontal();
        let cols = self.calculate_columns(available_width);

        // Calculate row height from first child's layout
        let row_height = layout
            .children()
            .next()
            .map(|child| child.bounds().height)
            .unwrap_or(0.0);

        // Check if layout changed
        let current = (cols, row_height.to_bits());
        let layout_changed = current != self.last_layout.get();

        if layout_changed {
            self.last_layout.set(current);
        }

        // Calculate scroll request if we have a target index
        let scroll_request = if let Some(index) = self.scroll_to_index {
            if cols > 0 && row_height > 0.0 {
                let row = index / cols;
                let row_spacing = self.row_spacing as f32;

                let item_top = self.padding.top + (row as f32) * (row_height + row_spacing);
                let item_bottom = item_top + row_height;

                // Get the viewport bounds (the visible area)
                let grid_bounds = layout.bounds();

                // Calculate visible range relative to grid content
                let visible_top = viewport.y - grid_bounds.y;
                let visible_bottom = visible_top + viewport.height;

                // Check if item is outside visible range
                if item_top < visible_top {
                    // Item is above viewport - scroll up
                    Some(ScrollRequest {
                        offset_y: item_top.max(0.0),
                    })
                } else if item_bottom > visible_bottom {
                    // Item is below viewport - scroll down
                    Some(ScrollRequest {
                        offset_y: (item_bottom - viewport.height).max(0.0),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Fire callback if layout changed or scroll needed
        if let Some(ref on_layout_changed) = self.on_layout_changed {
            if layout_changed || scroll_request.is_some() {
                shell.publish((on_layout_changed)(cols, row_height, scroll_request));
            }
        }

        let mut status = event::Status::Ignored;

        for ((child, state), child_layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            let child_status = child.as_widget_mut().on_event(
                state,
                event.clone(),
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

            if child_status == event::Status::Captured {
                status = event::Status::Captured;
            }
        }

        status
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        for ((child, state), layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            let interaction = child
                .as_widget()
                .mouse_interaction(state, layout, cursor, viewport, renderer);

            if interaction != mouse::Interaction::None {
                return interaction;
            }
        }

        mouse::Interaction::None
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &cosmic::Theme,
        style: &iced_renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        for ((child, state), layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            child
                .as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: cosmic::iced::Vector,
    ) -> Option<overlay::Element<'b, M, cosmic::Theme, Renderer>> {
        overlay::from_children(&mut self.children, tree, layout, renderer, translation)
    }
}

impl<'a, M: Clone + 'static> From<FlexGridInner<'a, M>> for Element<'a, M> {
    fn from(grid: FlexGridInner<'a, M>) -> Self {
        Element::new(grid)
    }
}
