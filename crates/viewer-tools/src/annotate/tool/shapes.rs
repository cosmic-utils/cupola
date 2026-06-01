mod operation;
mod preview;

pub use operation::ShapeOperation;
pub use preview::ShapePreview;

use crate::renderer::{build_path, fill_on_pixmap, stroke_on_pixmap};
use cosmic::iced::{Color, Point, Rectangle, Size};
use tiny_skia::{LineCap, LineJoin, Pixmap, Rect};
use std::f32::consts::{FRAC_PI_2, PI};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Rectangle,
    Ellipse,
    Line,
    Arrow,
    Star,
    Polygon,
}

/// Render a shape onto a Pixmap overlay (replaces draw_shape which used iced Frame).
pub(crate) fn render_shape(
    kind: ShapeKind,
    start: Point,
    end: Point,
    stroke_color: Color,
    fill_color: Option<Color>,
    width: f32,
    pixmap: &mut Pixmap,
    scale: f32,
) {
    match kind {
        ShapeKind::Star | ShapeKind::Polygon => {
            let verts = match kind {
                ShapeKind::Star => star_vertices(start, end),
                ShapeKind::Polygon => polygon_vertices(start, end, 6),
                _ => unreachable!(),
            };
            let Some(path) = build_path(|pb| {
                if let Some(first) = verts.first() {
                    pb.move_to(first.x, first.y);
                    for v in &verts[1..] {
                        pb.line_to(v.x, v.y);
                    }
                    pb.close();
                }
            }) else {
                return;
            };
            fill_on_pixmap(pixmap, &path, fill_color.unwrap_or(stroke_color));
        }
        ShapeKind::Arrow => {
            // Stroke the shaft
            if let Some(shaft) = build_path(|pb| {
                pb.move_to(start.x, start.y);
                pb.line_to(end.x, end.y);
            }) {
                stroke_on_pixmap(
                    pixmap,
                    &shaft,
                    stroke_color,
                    width / scale,
                    LineCap::Round,
                    LineJoin::Round,
                );
            }
            // Fill the arrowhead
            let (tip, left, right) = arrow_head_points(start, end, scale);
            if let Some(head) = build_path(|pb| {
                pb.move_to(tip.x, tip.y);
                pb.line_to(left.x, left.y);
                pb.line_to(right.x, right.y);
                pb.close();
            }) {
                fill_on_pixmap(pixmap, &head, stroke_color);
            }
        }
        _ => {
            let Some(path) = build_shape_path(kind, start, end) else {
                return;
            };
            if matches!(kind, ShapeKind::Rectangle | ShapeKind::Ellipse)
                && let Some(fill) = fill_color
            {
                fill_on_pixmap(pixmap, &path, fill);
            }
            stroke_on_pixmap(
                pixmap,
                &path,
                stroke_color,
                width / scale,
                LineCap::Round,
                LineJoin::Round,
            );
        }
    }
}

fn build_shape_path(
    kind: ShapeKind,
    start: Point,
    end: Point,
) -> Option<tiny_skia::Path> {
    build_path(|pb| match kind {
        ShapeKind::Rectangle => {
            let rect = normalize_rect(start, end);
            if let Some(r) = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height) {
                pb.push_rect(r);
            }
        }
        ShapeKind::Ellipse => {
            let rect = normalize_rect(start, end);
            let r = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height)
                .unwrap_or_else(|| Rect::from_xywh(0.0, 0.0, 1.0, 1.0).unwrap());
            pb.push_oval(r);
        }
        ShapeKind::Line => {
            pb.move_to(start.x, start.y);
            pb.line_to(end.x, end.y);
        }
        _ => {}
    })
}

pub(crate) fn normalize_rect(a: Point, b: Point) -> Rectangle {
    Rectangle::new(
        Point::new(a.x.min(b.x), a.y.min(b.y)),
        Size::new((a.x - b.x).abs(), (a.y - b.y).abs()),
    )
}

/// Returns (tip, left, right) points for a filled arrowhead triangle.
fn arrow_head_points(start: Point, end: Point, scale: f32) -> (Point, Point, Point) {
    let delta_x = end.x - start.x;
    let delta_y = end.y - start.y;
    let len = (delta_x * delta_x + delta_y * delta_y).sqrt();
    if len < 1.0 {
        return (end, end, end);
    }

    let head_len = (len * 0.25).min(30.0 / scale);
    let head_width = head_len * 0.5;
    let unit_x = delta_x / len;
    let unit_y = delta_y / len;
    let perp_x = -unit_y;
    let perp_y = unit_x;

    let base = Point::new(end.x - unit_x * head_len, end.y - unit_y * head_len);
    let left = Point::new(
        base.x + perp_x * head_width / 2.0,
        base.y + perp_y * head_width / 2.0,
    );
    let right = Point::new(
        base.x - perp_x * head_width / 2.0,
        base.y - perp_y * head_width / 2.0,
    );

    (end, left, right)
}

/// Arrow line segments for apply() (shaft + head lines).
pub(crate) fn arrow_segments(start: Point, end: Point) -> Vec<(Point, Point)> {
    let delta_x = end.x - start.x;
    let delta_y = end.y - start.y;
    let len = (delta_x * delta_x + delta_y * delta_y).sqrt();
    if len < 1.0 {
        return vec![(start, end)];
    }

    let head_len = (len * 0.25).min(30.0);
    let head_width = head_len * 0.5;
    let unit_x = delta_x / len;
    let unit_y = delta_y / len;
    let perp_x = -unit_y;
    let perp_y = unit_x;

    let base = Point::new(end.x - unit_x * head_len, end.y - unit_y * head_len);
    let left = Point::new(
        base.x + perp_x * head_width / 2.0,
        base.y - perp_y * head_width / 2.0,
    );
    let right = Point::new(
        base.x - perp_x * head_width / 2.0,
        base.y - perp_y * head_width / 2.0,
    );

    vec![(start, end), (left, end), (right, end)]
}

pub(crate) fn star_vertices(start: Point, end: Point) -> Vec<Point> {
    let bounds = normalize_rect(start, end);
    let center_x = bounds.x + bounds.width / 2.0;
    let center_y = bounds.y + bounds.height / 2.0;
    let outer_x = bounds.width / 2.0;
    let outer_y = bounds.height / 2.0;

    // Inner radius ~ 38% of outer for classic 5-pointed star
    let inner_x = outer_x * 0.38;
    let inner_y = outer_y * 0.38;

    let points = 5;
    let start_angle = -FRAC_PI_2;

    (0..points * 2)
        .map(|i| {
            let angle = start_angle + PI * i as f32 / points as f32;
            let (rx, ry) = if i % 2 == 0 {
                (outer_x, outer_y)
            } else {
                (inner_x, inner_y)
            };
            Point::new(center_x + rx * angle.cos(), center_y + ry * angle.sin())
        })
        .collect()
}

pub(crate) fn polygon_vertices(start: Point, end: Point, sides: usize) -> Vec<Point> {
    let bounds = normalize_rect(start, end);
    let center_x = bounds.x + bounds.width / 2.0;
    let center_y = bounds.y + bounds.height / 2.0;
    let rx = bounds.width / 2.0;
    let ry = bounds.height / 2.0;
    let start_angle = -FRAC_PI_2;

    (0..sides)
        .map(|i| {
            let angle = start_angle + 2.0 * PI * i as f32 / sides as f32;
            Point::new(center_x + rx * angle.cos(), center_y + ry * angle.sin())
        })
        .collect()
}
