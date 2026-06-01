//! codimate-render — the render pipeline, in two halves either side of the
//! [`Renderer`] seam (ADR 0001):
//!
//! - [`command`] — the renderer-neutral model: a laid-out `ConcreteScene`
//!   becomes a flat list of `RenderCommand`s in a `RenderFrame`. No backend.
//! - [`raster`] — the `tiny-skia` CPU adapter: `RenderCommand`s become pixels
//!   (`Bitmap`). The only half that depends on `tiny-skia`.
//!
//! A future GPU/Skia backend is a new adapter alongside [`raster`], behind the
//! same [`Renderer`] seam. Everything is re-exported flat.

pub mod command;
pub mod raster;

pub use command::*;
pub use raster::*;
