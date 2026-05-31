//! codimate-render — renderer boundary.
//!
//! The real `skia-safe` backend belongs here. This first slice establishes a
//! renderer-neutral command stream and the `Renderer` trait.

use codimate_core::{Color, ConcreteNode};
use codimate_layout::LayoutFrame;

/// Renderer-neutral drawing command produced from a laid-out frame.
#[derive(Clone, Copy, Debug, PartialEq)]
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
}

/// Convert a laid-out frame into renderer-neutral commands.
pub fn render_commands(frame: &LayoutFrame) -> Vec<RenderCommand> {
    frame
        .scene
        .children
        .iter()
        .map(|node| match node {
            ConcreteNode::Circle(circle) => RenderCommand::Circle {
                x: circle.x,
                y: circle.y,
                radius: circle.radius,
                fill: circle.fill,
            },
            ConcreteNode::Rect(rect) => RenderCommand::Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
                fill: rect.fill,
            },
        })
        .collect()
}

/// Backend trait implemented by concrete renderers.
pub trait Renderer {
    type Error;

    fn render(&mut self, frame: &LayoutFrame) -> Result<(), Self::Error>;
}

/// Placeholder backend type for the future `skia-safe` implementation.
#[derive(Default)]
pub struct SkiaRenderer {
    last_commands: Vec<RenderCommand>,
}

impl SkiaRenderer {
    pub fn last_commands(&self) -> &[RenderCommand] {
        &self.last_commands
    }
}

impl Renderer for SkiaRenderer {
    type Error = core::convert::Infallible;

    fn render(&mut self, frame: &LayoutFrame) -> Result<(), Self::Error> {
        self.last_commands = render_commands(frame);
        Ok(())
    }
}
