use crate::{
    ToolOperation, RotateDirection,
    annotate::tool::text::{
        TextSpan, LINE_HEIGHT_FACTOR,
        build_buffer_line, group_spans, span_attrs,
    },
    annotate::tool::text::preview::{TextEditState, TextPreview},
    renderer::{build_path, stroke_on_pixmap},
};
use cosmic::iced::{Color, Point, Rectangle, Size, alignment::Horizontal};
use cosmic::iced_widget::graphics::text::{cosmic_text, font_system};
use image::{DynamicImage, Rgba};
use tiny_skia::{LineCap, LineJoin, Pixmap};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct TextOperation {
    pub position: Point,
    pub spans: Vec<TextSpan>,
    pub color: Color,
    pub font_size: f32,
    pub font_family: &'static str,
    pub alignment: Horizontal,
    pub bounding_box: Rectangle,
    pub edit_scale: f32,
}

impl TextOperation {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        position: Point,
        spans: Vec<TextSpan>,
        color: Color,
        font_size: f32,
        font_family: &'static str,
        alignment: Horizontal,
        bounding_box: Rectangle,
        edit_scale: f32,
    ) -> Self {
        Self {
            position,
            spans,
            color,
            font_size,
            font_family,
            alignment,
            bounding_box,
            edit_scale,
        }
    }

    pub fn bounds(&self) -> Rectangle {
        self.bounding_box
    }

    pub fn hit_test(&self, point: Point) -> bool {
        self.bounds().contains(point)
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.position.x += dx;
        self.position.y += dy;
        self.bounding_box.x += dx;
        self.bounding_box.y += dy;
    }

    pub fn to_preview(&self) -> TextPreview {
        let mut preview = TextPreview::new(
            self.color,
            self.font_size,
            self.font_family,
            false,
            false,
            false,
            self.alignment,
        );
        preview.bounding_box = self.bounding_box;
        preview.last_scale.set(self.edit_scale);
        preview.state = TextEditState::Editing;

        if let Some(last) = self.spans.last() {
            preview.bold = last.bold;
            preview.italic = last.italic;
            preview.underline = last.underline;
            if let Some([r, g, b, a]) = last.color {
                preview.color = Color::from_rgba(r, g, b, a);
            }
            if let Some(fs) = last.font_size {
                preview.font_size = fs;
            }
        }

        preview.init_editor_from_spans(&self.spans);
        preview
    }

    fn build_buffer(&self, scale: f32) -> cosmic_text::Buffer {
        let metrics =
            cosmic_text::Metrics::new(self.font_size, self.font_size * LINE_HEIGHT_FACTOR);
        let default_attrs =
            cosmic_text::Attrs::new().family(cosmic_text::Family::Name(self.font_family));

        let mut font_sys = font_system().write().expect("Write font system");
        let mut buffer = cosmic_text::Buffer::new(font_sys.raw(), metrics);
        buffer.set_size(font_sys.raw(), Some(self.bounding_box.width * scale), None);
        buffer.set_wrap(font_sys.raw(), cosmic_text::Wrap::WordOrGlyph);

        let lines = group_spans(&self.spans);
        buffer.lines.clear();
        for (line_text, line_spans, line_align) in &lines {
            let mut attrs_list = cosmic_text::AttrsList::new(&default_attrs);
            let mut offset = 0;
            for span in line_spans {
                let end = offset + span.text.len();
                attrs_list
                    .add_span(offset..end, &span_attrs(default_attrs.clone(), span, 1.0));
                offset = end;
            }
            buffer
                .lines
                .push(build_buffer_line(line_text, attrs_list, *line_align));
        }
        buffer.shape_until_scroll(font_sys.raw(), false);
        buffer
    }

    fn render_text_to_pixmap(&self, pixmap: &mut Pixmap, scale: f32) {
        let buffer = self.build_buffer(scale);
        let origin = self.bounding_box.position();

        let fallback_color = cosmic_text::Color::rgba(
            (self.color.r * 255.0) as u8,
            (self.color.g * 255.0) as u8,
            (self.color.b * 255.0) as u8,
            (self.color.a * 255.0) as u8,
        );

        let mut font_sys = font_system().write().expect("Write font system");
        let mut swash_cache = cosmic_text::SwashCache::new();
        let pix_w = pixmap.width() as i32;
        let pix_h = pixmap.height() as i32;
        let inv = 1.0 / scale;

        for run in buffer.layout_runs() {
            let line = &buffer.lines[run.line_i];
            let attrs_list = line.attrs_list();

            for glyph in run.glyphs {
                let glyph_color = attrs_list
                    .get_span(glyph.start)
                    .color_opt
                    .unwrap_or(fallback_color);

                let physical = glyph.physical((0.0, 0.0), 1.0);
                let gx = origin.x + glyph.x * inv + glyph.x_offset * inv;
                let gy = origin.y + run.line_y * inv + glyph.y_offset * inv;

                swash_cache.with_pixels(
                    font_sys.raw(),
                    physical.cache_key,
                    glyph_color,
                    |off_x, off_y, color| {
                        let px = (gx as i32) + off_x;
                        let py = (gy as i32) + off_y;

                        if px >= 0 && px < pix_w && py >= 0 && py < pix_h {
                            let alpha = color.a() as f32 / 255.0;
                            if alpha > 0.0 {
                                let idx = (py as u32 * pixmap.width() + px as u32) as usize * 4;
                                let data = pixmap.data_mut();
                                let dst_a = data[idx + 3] as f32 / 255.0;
                                let out_a = alpha + dst_a * (1.0 - alpha);
                                if out_a > 0.0 {
                                    // Pixmap stores premultiplied alpha
                                    let src_r = color.r() as f32 * alpha;
                                    let src_g = color.g() as f32 * alpha;
                                    let src_b = color.b() as f32 * alpha;
                                    let dst_r = data[idx] as f32;
                                    let dst_g = data[idx + 1] as f32;
                                    let dst_b = data[idx + 2] as f32;
                                    data[idx] =
                                        (src_r + dst_r * (1.0 - alpha)).min(255.0) as u8;
                                    data[idx + 1] =
                                        (src_g + dst_g * (1.0 - alpha)).min(255.0) as u8;
                                    data[idx + 2] =
                                        (src_b + dst_b * (1.0 - alpha)).min(255.0) as u8;
                                    data[idx + 3] = (out_a * 255.0).min(255.0) as u8;
                                }
                            }
                        }
                    },
                );
            }

            // Underlines
            if run.glyphs.is_empty() {
                continue;
            }
            let mut i = 0;
            while i < run.glyphs.len() {
                let first = &run.glyphs[i];
                let a = attrs_list.get_span(first.start);
                let mut j = i + 1;
                while j < run.glyphs.len() {
                    if attrs_list.get_span(run.glyphs[j].start) != a {
                        break;
                    }
                    j += 1;
                }

                if a.metadata == 1 {
                    let last = &run.glyphs[j - 1];
                    let span_fs = a.metrics_opt.map_or(self.font_size, |m| {
                        let metrics: cosmic_text::Metrics = m.into();
                        metrics.font_size
                    });
                    let span_color = a.color_opt.map_or(self.color, |c| {
                        Color::from_rgba(
                            c.r() as f32 / 255.0,
                            c.g() as f32 / 255.0,
                            c.b() as f32 / 255.0,
                            c.a() as f32 / 255.0,
                        )
                    });
                    let uy =
                        origin.y + (run.line_top + span_fs * LINE_HEIGHT_FACTOR + 2.0) * inv;
                    let ux = origin.x + first.x * inv;
                    let uw = (last.x + last.w - first.x) * inv;

                    if let Some(path) = build_path(|pb| {
                        pb.move_to(ux, uy);
                        pb.line_to(ux + uw, uy);
                    }) {
                        stroke_on_pixmap(
                            pixmap,
                            &path,
                            span_color,
                            1.0 / scale,
                            LineCap::Butt,
                            LineJoin::Miter,
                        );
                    }
                }

                i = j;
            }
        }
    }
}

impl ToolOperation for TextOperation {
    fn render(&self, pixmap: &mut Pixmap, _image_size: Size, scale: f32) {
        if self.spans.is_empty() {
            return;
        }
        self.render_text_to_pixmap(pixmap, scale);
    }

    fn apply(&self, image: &mut DynamicImage) {
        if self.spans.is_empty() || self.bounding_box.width < 1.0 {
            return;
        }

        let img_font = self.font_size / self.edit_scale;
        let img_metrics =
            cosmic_text::Metrics::new(img_font, img_font * LINE_HEIGHT_FACTOR);
        let default_attrs =
            cosmic_text::Attrs::new().family(cosmic_text::Family::Name(self.font_family));
        let font_scale = 1.0 / self.edit_scale;

        let mut font_sys = font_system().write().expect("Write font system");
        let mut buffer = cosmic_text::Buffer::new(font_sys.raw(), img_metrics);
        buffer.set_size(font_sys.raw(), Some(self.bounding_box.width), None);
        buffer.set_wrap(font_sys.raw(), cosmic_text::Wrap::WordOrGlyph);

        let lines = group_spans(&self.spans);
        buffer.lines.clear();
        for (line_text, line_spans, line_align) in &lines {
            let mut attrs_list = cosmic_text::AttrsList::new(&default_attrs);
            let mut offset = 0;
            for span in line_spans {
                let end = offset + span.text.len();
                attrs_list.add_span(
                    offset..end,
                    &span_attrs(default_attrs.clone(), span, font_scale),
                );
                offset = end;
            }
            buffer
                .lines
                .push(build_buffer_line(line_text, attrs_list, *line_align));
        }
        buffer.shape_until_scroll(font_sys.raw(), false);

        let rgba = image.as_mut_rgba8().expect("image should be RGBA");
        let (img_w, img_h) = (rgba.width() as i32, rgba.height() as i32);

        let fallback_color = cosmic_text::Color::rgba(
            (self.color.r * 255.0) as u8,
            (self.color.g * 255.0) as u8,
            (self.color.b * 255.0) as u8,
            (self.color.a * 255.0) as u8,
        );

        let mut swash_cache = cosmic_text::SwashCache::new();
        let origin = self.bounding_box.position();

        for run in buffer.layout_runs() {
            let line = &buffer.lines[run.line_i];
            let attrs_list = line.attrs_list();

            for glyph in run.glyphs {
                let glyph_color = attrs_list
                    .get_span(glyph.start)
                    .color_opt
                    .unwrap_or(fallback_color);

                let physical = glyph.physical((0.0, 0.0), 1.0);
                let gx = origin.x + glyph.x + glyph.x_offset;
                let gy = origin.y + run.line_y + glyph.y_offset;

                swash_cache.with_pixels(
                    font_sys.raw(),
                    physical.cache_key,
                    glyph_color,
                    |off_x, off_y, color| {
                        let px = (gx as i32) + off_x;
                        let py = (gy as i32) + off_y;

                        if px >= 0 && px < img_w && py >= 0 && py < img_h {
                            let alpha = color.a() as f32 / 255.0;
                            if alpha > 0.0 {
                                let existing = rgba.get_pixel(px as u32, py as u32);
                                let blended = Rgba([
                                    blend_channel(existing[0], color.r(), alpha),
                                    blend_channel(existing[1], color.g(), alpha),
                                    blend_channel(existing[2], color.b(), alpha),
                                    (existing[3] as f32 * (1.0 - alpha) + 255.0 * alpha)
                                        .min(255.0)
                                        as u8,
                                ]);
                                rgba.put_pixel(px as u32, py as u32, blended);
                            }
                        }
                    },
                );
            }

            // Underlines
            if run.glyphs.is_empty() {
                continue;
            }
            let mut i = 0;
            while i < run.glyphs.len() {
                let first = &run.glyphs[i];
                let a = attrs_list.get_span(first.start);
                let mut j = i + 1;
                while j < run.glyphs.len() {
                    if attrs_list.get_span(run.glyphs[j].start) != a {
                        break;
                    }
                    j += 1;
                }

                if a.metadata == 1 {
                    let last = &run.glyphs[j - 1];
                    let span_fs = a.metrics_opt.map_or(img_font, |m| {
                        let metrics: cosmic_text::Metrics = m.into();
                        metrics.font_size
                    });
                    let uy =
                        (origin.y + run.line_top + span_fs * LINE_HEIGHT_FACTOR + 2.0) as i32;
                    let x_start = (origin.x + first.x) as i32;
                    let x_end = (origin.x + last.x + last.w) as i32;

                    let color = a.color_opt.unwrap_or(fallback_color);
                    let r = color.r();
                    let g = color.g();
                    let b = color.b();
                    let ca = color.a();

                    if uy >= 0 && uy < img_h {
                        for x in x_start.max(0)..x_end.min(img_w) {
                            rgba.put_pixel(x as u32, uy as u32, Rgba([r, g, b, ca]));
                        }
                    }
                }

                i = j;
            }
        }

        *image = DynamicImage::ImageRgba8(rgba.clone());
    }

    fn commit(&self) -> Option<Box<dyn ToolOperation>> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn transform_rotate(&mut self, direction: RotateDirection, image_size: Size) {
        let (width, height) = (image_size.width, image_size.height);
        let (x, y) = (self.position.x, self.position.y);

        self.position = match direction {
            RotateDirection::Left => Point::new(y, width - x),
            RotateDirection::Right => Point::new(height - y, x),
        }
    }

    fn transform_crop(&mut self, region: Rectangle) {
        self.position.x -= region.x;
        self.position.y -= region.y;
        self.bounding_box.x -= region.x;
        self.bounding_box.y -= region.y;
    }

    fn movable(&self) -> bool {
        true
    }

    fn hit_test(&self, point: Point) -> bool {
        TextOperation::hit_test(self, point)
    }

    fn translate(&mut self, dx: f32, dy: f32) {
        TextOperation::translate(self, dx, dy);
    }

    fn bounds(&self) -> Option<Rectangle> {
        Some(TextOperation::bounds(self))
    }
}

fn blend_channel(dst: u8, src: u8, alpha: f32) -> u8 {
    ((dst as f32) * (1.0 - alpha) + (src as f32) * alpha).min(255.0) as u8
}
