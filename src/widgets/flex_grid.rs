use std::{cell::Cell, f32};

use cosmic::{
    Element, Renderer,
    iced::{
        Length, Padding, Point, Rectangle, Size,
        advanced::{
            Clipboard, Layout, Shell, Widget,
            layout::{Limits, Node},
            overlay, renderer as iced_renderer,
            widget::{Operation, Tree},
        },
        event::{self, Event},
        mouse::{self, Cursor},
    },
};
use taffy::{
    Dimension, Display, GridPlacement, JustifyItems, LengthPercentage, Size as TaffySize, Style,
    TaffyTree, prelude::fr,
};

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
    last_layout: Cell<(usize, f32)>,
    on_layout_changed: Option<Box<dyn Fn(usize, f32) -> M + 'a>>, // (cols, row_height)
}

pub fn flex_grid<'a, M>(children: Vec<Element<'a, M>>) -> FlexGrid<'a, M> {
    FlexGrid {
        children,
        padding: Padding::ZERO,
        column_spacing: 0,
        row_spacing: 0,
        width: Length::Shrink,
        height: Length::Shrink,
        item_width: 100.0,
        min_columns: 1,
        max_columns: None,
        last_layout: Cell::new((0, 0.0)),
        on_layout_changed: None,
    }
}

impl<'a, M> FlexGrid<'a, M> {
    pub fn item_width(mut self, width: f32) -> Self {
        self.item_width = width;
        self
    }

    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self
    }

    pub fn row_spacing(mut self, spacing: u16) -> Self {
        self.row_spacing = spacing;
        self
    }

    pub fn spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self.row_spacing = spacing;
        self
    }

    pub fn min_columns(mut self, min: usize) -> Self {
        self.min_columns = min.max(1);
        self
    }

    pub fn max_columns(mut self, max: usize) -> Self {
        self.max_columns = Some(max);
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Calculate column count for a given available width
    pub fn calculate_columns(&self, available_width: f32) -> usize {
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

    /// Update the columns on a resize event
    pub fn on_layout_changed<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, f32) -> M + 'a,
    {
        self.on_layout_changed = Some(Box::new(f));
        self
    }
}

impl<'a, M: Clone + 'static> Widget<M, cosmic::Theme, Renderer> for FlexGrid<'a, M> {
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
        // Calculates current columns based on layout bounds
        let available_width = layout.bounds().width - self.padding.horizontal();
        let cols = self.calculate_columns(available_width);

        // Caculate row height from first child's layout
        let row_height = layout
            .children()
            .next()
            .map(|child| child.bounds().height)
            .unwrap_or(0.0);

        // Fire callback if columns changed
        if let Some(ref on_layout_changed) = self.on_layout_changed {
            let current = (cols, row_height);
            if current != self.last_layout.get() {
                self.last_layout.set(current);
                shell.publish((on_layout_changed)(cols, row_height));
            }
        }

        let mut status = event::Status::Ignored;

        for ((child, state), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            let child_status = child.as_widget_mut().on_event(
                state,
                event.clone(),
                layout,
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

impl<'a, M: Clone + 'static> From<FlexGrid<'a, M>> for Element<'a, M> {
    fn from(grid: FlexGrid<'a, M>) -> Self {
        Element::new(grid)
    }
}
