//! The renderer-neutral command model and the `Renderer` seam.
//!
//! This is the backend-agnostic half of the crate: a `ConcreteScene` (already
//! laid out) is projected into a flat list of [`RenderCommand`]s packaged in a
//! [`RenderFrame`]. No pixels, no `tiny-skia` — any backend (the CPU
//! [`crate::raster`] adapter today, a GPU one tomorrow) consumes this behind
//! the [`Renderer`] seam (ADR 0001).

use codimate_core::{
    circle_path, rect_path, Color, ConcreteGeometry, ConcreteNode, ConcretePrimitive,
    ConcreteTransform, Path, Segment, TextAlign, Vec2,
};
use codimate_layout::{LayoutFrame, Viewport};

/// Renderer-neutral drawing command produced from a laid-out frame.
#[derive(Clone, Debug, PartialEq)]
pub enum RenderCommand {
    Circle {
        x: f32,
        y: f32,
        radius: f32,
        fill: Color,
    },
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        fill: Color,
    },
    Path {
        segments: Vec<Segment>,
        closed: bool,
        fill: Color,
        stroke_width: f32,
        stroke_color: Color,
    },
    Text {
        x: f32,
        y: f32,
        text: String,
        font_size: f32,
        fill: Color,
        align: TextAlign,
    },
}

/// Convert a laid-out frame into renderer-neutral commands.
pub fn render_commands(frame: &LayoutFrame) -> Vec<RenderCommand> {
    frame
        .scene
        .children
        .iter()
        .flat_map(|node| match node {
            ConcreteNode::Primitive(primitive) => render_primitive_commands(primitive),
            ConcreteNode::Circle(circle) => vec![RenderCommand::Circle {
                x: circle.x,
                y: circle.y,
                radius: circle.radius,
                fill: circle.fill,
            }],
            ConcreteNode::Rect(rect) => vec![RenderCommand::Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
                fill: rect.fill,
            }],
            ConcreteNode::Path(path) => vec![RenderCommand::Path {
                segments: path.path.segments.clone(),
                closed: path.path.closed,
                fill: path.fill,
                stroke_width: path.stroke_width,
                stroke_color: path.stroke_color,
            }],
            ConcreteNode::Text(text) => vec![RenderCommand::Text {
                x: text.x,
                y: text.y,
                text: text.text.clone(),
                font_size: text.font_size,
                fill: text.fill,
                align: text.align,
            }],
        })
        .collect()
}

fn render_primitive_commands(primitive: &ConcretePrimitive) -> Vec<RenderCommand> {
    let opacity = primitive.transform.opacity;
    let mut fill = primitive.style.fill;
    let mut stroke = primitive.style.stroke_color;
    fill.a *= opacity;
    stroke.a *= opacity;

    let scale_factor = average_scale(&primitive.transform).max(0.0);

    match &primitive.geometry {
        ConcreteGeometry::Circle { radius } => {
            let local = circle_path(0.0, 0.0, *radius);
            let world = transform_path(&local, &primitive.transform);
            vec![RenderCommand::Path {
                segments: world.segments,
                closed: world.closed,
                fill,
                stroke_width: primitive.style.stroke_width * scale_factor,
                stroke_color: stroke,
            }]
        }
        ConcreteGeometry::Rect { width, height } => {
            let local = rect_path(-width / 2.0, -height / 2.0, *width, *height);
            let world = transform_path(&local, &primitive.transform);
            vec![RenderCommand::Path {
                segments: world.segments,
                closed: world.closed,
                fill,
                stroke_width: primitive.style.stroke_width * scale_factor,
                stroke_color: stroke,
            }]
        }
        ConcreteGeometry::Path { path } => {
            let world = transform_path(path, &primitive.transform);
            vec![RenderCommand::Path {
                segments: world.segments,
                closed: world.closed,
                fill,
                stroke_width: primitive.style.stroke_width * scale_factor,
                stroke_color: stroke,
            }]
        }
        ConcreteGeometry::Text {
            text,
            font_size,
            align,
        } => {
            let origin = transform_point(Vec2::new(0.0, 0.0), &primitive.transform);
            vec![RenderCommand::Text {
                x: origin.x,
                y: origin.y,
                text: text.clone(),
                font_size: *font_size * scale_factor.max(0.01),
                fill,
                align: *align,
            }]
        }
    }
}

fn average_scale(transform: &ConcreteTransform) -> f32 {
    (transform.scale.x.abs() + transform.scale.y.abs()) * 0.5
}

fn transform_path(path: &Path, transform: &ConcreteTransform) -> Path {
    Path {
        segments: path
            .segments
            .iter()
            .copied()
            .map(|segment| transform_segment(segment, transform))
            .collect(),
        closed: path.closed,
    }
}

fn transform_segment(segment: Segment, transform: &ConcreteTransform) -> Segment {
    match segment {
        Segment::MoveTo(p) => Segment::MoveTo(transform_point(p, transform)),
        Segment::Line(a, b) => {
            Segment::Line(transform_point(a, transform), transform_point(b, transform))
        }
        Segment::Quad(a, c, b) => Segment::Quad(
            transform_point(a, transform),
            transform_point(c, transform),
            transform_point(b, transform),
        ),
        Segment::Cubic(a, c1, c2, b) => Segment::Cubic(
            transform_point(a, transform),
            transform_point(c1, transform),
            transform_point(c2, transform),
            transform_point(b, transform),
        ),
        Segment::Close => Segment::Close,
    }
}

fn transform_point(point: Vec2, transform: &ConcreteTransform) -> Vec2 {
    let dx = (point.x - transform.pivot.x) * transform.scale.x;
    let dy = (point.y - transform.pivot.y) * transform.scale.y;
    let rad = transform.rotation_deg.to_radians();
    let (sin, cos) = rad.sin_cos();
    Vec2::new(
        transform.pos.x + transform.pivot.x + dx * cos - dy * sin,
        transform.pos.y + transform.pivot.y + dx * sin + dy * cos,
    )
}

/// Renderer-neutral frame ready for a backend or export pipeline.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderFrame {
    pub name: String,
    pub elapsed_seconds: f32,
    pub viewport: Viewport,
    pub commands: Vec<RenderCommand>,
}

/// Package a laid-out frame and metadata into renderer-neutral commands.
pub fn render_frame(
    name: impl Into<String>,
    elapsed_seconds: f32,
    frame: &LayoutFrame,
) -> RenderFrame {
    RenderFrame {
        name: name.into(),
        elapsed_seconds,
        viewport: frame.viewport,
        commands: render_commands(frame),
    }
}

/// Inject debug metadata into a rendered frame.
///
/// Appends a green overlay [`RenderCommand::Text`] with the animation name and
/// elapsed seconds so rendered frames can be traced back to their source during
/// testing or AI feedback.
pub fn inject_debug_metadata(frame: &mut RenderFrame) {
    let text = format!("{}  {:.2}s", frame.name, frame.elapsed_seconds);
    frame.commands.push(RenderCommand::Text {
        x: 8.0,
        y: 20.0,
        text,
        font_size: 12.0,
        fill: Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
        align: TextAlign::Left,
    });
}

/// Backend trait implemented by concrete renderers — the seam where a CPU,
/// GPU, or Skia adapter plugs in (ADR 0001).
pub trait Renderer {
    type Error;

    fn render(&mut self, frame: &LayoutFrame) -> Result<(), Self::Error>;
}
