//! Reconciliation — two pictures in, movement out.
//!
//! Given consecutive Scenes whose shapes carry stable names, pair them by name
//! and turn the differences into `Animated<T>`. A shape present in both moved;
//! one that appears enters; one that disappears leaves. That is the whole idea
//! Codimate rests on (ADR 0008), and it lives here rather than in the bindings
//! so that it can be tested, reused by another frontend, and read without a
//! Python toolchain in the way.
//!
//! The input is a flat `Shape` — a data structure, not an API — so nothing
//! here knows that Python exists.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};

use codimate_animation::Playable;
use codimate_core::{
    scene::AnchorKind, scene::Transformable, tween, Animated, Color, ConcreteScene, Geometry,
    IntoAnimated, Path, Pixels, Primitive, Scene, Segment, Style, TextAlign, TimeCurve, Vec2,
};

/// Why a Scene could not be built.
///
/// Every one of these is an authoring mistake — an unknown shape kind, a
/// motion path that does not exist, LaTeX that will not typeset. The frontend
/// decides how to present it; this crate only says what went wrong.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Error(pub String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

mod shape;
pub use shape::*;
mod motion;
pub use motion::*;
mod pen;
pub use pen::*;
mod media;
pub use media::*;
mod segment;
pub use segment::*;
mod camera;
pub use camera::*;
mod explanation;
pub use explanation::*;

#[cfg(test)]
mod tests;
