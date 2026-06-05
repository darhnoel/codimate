//! The `tiny-skia` CPU rasterizer — one adapter behind the `Renderer` seam.
//!
//! Turns renderer-neutral [`RenderCommand`]s into pixels ([`Bitmap`]). This is
//! the only half of the crate that knows about `tiny-skia` / `ab_glyph`; the
//! neutral model lives in [`crate::command`]. Pure and deterministic — no I/O.

use crate::command::{render_commands, RenderCommand, RenderFrame, Renderer};
use codimate_core::{Color, Segment, TextAlign};
use codimate_layout::{LayoutFrame, Viewport};

/// Bundled monospace font embedded in the binary (DejaVu Sans Mono).
static FONT_DATA: &[u8] = include_bytes!("../DejaVuSansMono.ttf");

/// A finished image in memory: straight RGBA8, 4 bytes per pixel, row-major,
/// top-left origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bitmap {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Bitmap {
    /// The `(r, g, b, a)` bytes at a pixel. Panics if out of bounds.
    pub fn pixel(&self, x: u32, y: u32) -> (u8, u8, u8, u8) {
        let i = ((y * self.width + x) * 4) as usize;
        (
            self.rgba[i],
            self.rgba[i + 1],
            self.rgba[i + 2],
            self.rgba[i + 3],
        )
    }
}

/// Paint a renderer-neutral frame into pixels. Pure and deterministic — no I/O.
///
/// Background is opaque black; top-left origin, y-down, 1 viewport unit = 1px;
/// anti-aliasing on.
///
/// ```
/// use codimate_core::Color;
/// use codimate_layout::Viewport;
/// use codimate_render::{rasterize, RenderCommand, RenderFrame};
///
/// let frame = RenderFrame {
///     name: "demo".into(),
///     elapsed_seconds: 0.0,
///     viewport: Viewport::new(64.0, 64.0),
///     commands: vec![RenderCommand::Circle { x: 32.0, y: 32.0, radius: 16.0, fill: Color::RED }],
/// };
/// let img = rasterize(&frame);
/// assert_eq!(img.pixel(32, 32), (255, 0, 0, 255)); // center of the circle is red
/// ```
pub fn rasterize(frame: &RenderFrame) -> Bitmap {
    rasterize_commands(frame.viewport, &frame.commands)
}

fn rasterize_commands(viewport: Viewport, commands: &[RenderCommand]) -> Bitmap {
    use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Rect as SkRect, Transform};

    let width = viewport.width.round().max(1.0) as u32;
    let height = viewport.height.round().max(1.0) as u32;

    let mut pixmap = Pixmap::new(width, height).expect("viewport is at least 1x1");
    pixmap.fill(tiny_skia::Color::from_rgba8(0, 0, 0, 255)); // opaque black background

    for command in commands {
        // AA on (tiny-skia default, made explicit).
        let mut paint = Paint {
            anti_alias: true,
            ..Default::default()
        };

        match command {
            RenderCommand::Circle { x, y, radius, fill } => {
                paint.set_color(to_sk_color(*fill));
                let mut builder = PathBuilder::new();
                builder.push_circle(*x, *y, *radius);
                if let Some(path) = builder.finish() {
                    pixmap.fill_path(
                        &path,
                        &paint,
                        FillRule::Winding,
                        Transform::identity(),
                        None,
                    );
                }
            }
            RenderCommand::Rect {
                x,
                y,
                width: w,
                height: h,
                fill,
            } => {
                paint.set_color(to_sk_color(*fill));
                if let Some(rect) = SkRect::from_xywh(*x, *y, *w, *h) {
                    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                }
            }
            RenderCommand::Path {
                segments,
                closed,
                fill,
                stroke_width,
                stroke_color,
            } => {
                let mut builder = PathBuilder::new();
                let mut first = true;
                let mut saw_close = false;
                for segment in segments.iter() {
                    match segment {
                        Segment::MoveTo(p) => {
                            builder.move_to(p.x, p.y);
                            first = false;
                        }
                        Segment::Close => {
                            builder.close();
                            saw_close = true;
                        }
                        Segment::Line(from, to) => {
                            if first {
                                builder.move_to(from.x, from.y);
                                first = false;
                            }
                            builder.line_to(to.x, to.y);
                        }
                        Segment::Quad(from, ctrl, to) => {
                            if first {
                                builder.move_to(from.x, from.y);
                                first = false;
                            }
                            builder.quad_to(ctrl.x, ctrl.y, to.x, to.y);
                        }
                        Segment::Cubic(from, ctrl1, ctrl2, to) => {
                            if first {
                                builder.move_to(from.x, from.y);
                                first = false;
                            }
                            builder.cubic_to(ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y);
                        }
                    }
                }
                if *closed && !saw_close {
                    builder.close();
                }
                if let Some(path) = builder.finish() {
                    // Fill
                    paint.set_color(to_sk_color(*fill));
                    pixmap.fill_path(
                        &path,
                        &paint,
                        FillRule::Winding,
                        Transform::identity(),
                        None,
                    );

                    // Stroke (when width > 0)
                    if *stroke_width > 0.0 {
                        paint.set_color(to_sk_color(*stroke_color));
                        let stroke = tiny_skia::Stroke {
                            width: *stroke_width,
                            ..Default::default()
                        };
                        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                    }
                }
            }
            RenderCommand::Text {
                x,
                y,
                text,
                font_size,
                fill,
                align,
            } => {
                render_text(&mut pixmap, *x, *y, text, *font_size, *fill, *align);
            }
        }
    }

    // tiny-skia stores premultiplied alpha; demultiply back to straight RGBA8.
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for px in pixmap.pixels() {
        let c = px.demultiply();
        rgba.push(c.red());
        rgba.push(c.green());
        rgba.push(c.blue());
        rgba.push(c.alpha());
    }

    Bitmap {
        width,
        height,
        rgba,
    }
}

fn render_text(
    pixmap: &mut tiny_skia::Pixmap,
    x: f32,
    y: f32,
    text: &str,
    font_size: f32,
    fill: Color,
    align: TextAlign,
) {
    use ab_glyph::{point, Font, FontRef, PxScale, ScaleFont};

    let font = match FontRef::try_from_slice(FONT_DATA) {
        Ok(f) => f,
        Err(_) => return,
    };
    let scale = PxScale::from(font_size);
    let scaled = font.as_scaled(scale);

    let fill_r = (fill.r * 255.0) as u32;
    let fill_g = (fill.g * 255.0) as u32;
    let fill_b = (fill.b * 255.0) as u32;
    let fill_a = (fill.a * 255.0) as u32;

    let w = pixmap.width();
    let h = pixmap.height();
    let data = pixmap.data_mut();
    let width = text_width(&scaled, &font, text);
    let mut cursor_x = match align {
        TextAlign::Left => x,
        TextAlign::Center => x - width / 2.0,
    };

    for ch in text.chars() {
        let gid = font.glyph_id(ch);
        let glyph = gid.with_scale_and_position(scale, point(cursor_x, y));

        if let Some(outline) = scaled.outline_glyph(glyph) {
            let bounds = outline.px_bounds();
            let origin_x = bounds.min.x as i32;
            let origin_y = bounds.min.y as i32;

            outline.draw(|gx: u32, gy: u32, coverage: f32| {
                let px = origin_x + gx as i32;
                let py = origin_y + gy as i32;
                if px < 0 || py < 0 {
                    return;
                }
                let (px, py) = (px as u32, py as u32);
                if px >= w || py >= h {
                    return;
                }

                let cov = (coverage * 255.0) as u32;
                if cov == 0 {
                    return;
                }

                let src_a = (fill_a * cov) / 255;
                let src_r = (fill_r * cov) / 255;
                let src_g = (fill_g * cov) / 255;
                let src_b = (fill_b * cov) / 255;

                let i = ((py * w + px) * 4) as usize;
                let dst_r = data[i] as u32;
                let dst_g = data[i + 1] as u32;
                let dst_b = data[i + 2] as u32;
                let dst_a = data[i + 3] as u32;

                let inv_a = 255 - src_a;
                data[i] = (src_r + (dst_r * inv_a + 128) / 255).min(255) as u8;
                data[i + 1] = (src_g + (dst_g * inv_a + 128) / 255).min(255) as u8;
                data[i + 2] = (src_b + (dst_b * inv_a + 128) / 255).min(255) as u8;
                data[i + 3] = (src_a + (dst_a * inv_a + 128) / 255).min(255) as u8;
            });
        }

        cursor_x += scaled.h_advance(gid);
    }
}

fn text_width(
    scaled: &ab_glyph::PxScaleFont<&ab_glyph::FontRef<'_>>,
    font: &ab_glyph::FontRef<'_>,
    text: &str,
) -> f32 {
    use ab_glyph::{Font, ScaleFont};

    text.chars()
        .map(|ch| scaled.h_advance(font.glyph_id(ch)))
        .sum()
}

fn to_sk_color(color: Color) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba(color.r, color.g, color.b, color.a)
        .unwrap_or(tiny_skia::Color::BLACK)
}

/// CPU rasterizer backend (`tiny-skia`). Stores the last rendered `Bitmap`.
/// Replaces the earlier `skia-safe` placeholder (see ADR 0001).
#[derive(Default)]
pub struct RasterRenderer {
    last: Option<Bitmap>,
}

impl RasterRenderer {
    pub fn last(&self) -> Option<&Bitmap> {
        self.last.as_ref()
    }
}

impl Renderer for RasterRenderer {
    type Error = core::convert::Infallible;

    fn render(&mut self, frame: &LayoutFrame) -> Result<(), Self::Error> {
        self.last = Some(rasterize_commands(frame.viewport, &render_commands(frame)));
        Ok(())
    }
}
