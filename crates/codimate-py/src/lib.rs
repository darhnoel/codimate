//! PyO3 bindings — the Python Authoring Surface's one crossing (ADR 0008).
//!
//! This crate translates and nothing else. Python hands over a list of flat
//! Scenes, one per Trace Event; those become `codimate_reconcile::Shape`, the
//! reconciler turns consecutive Scenes into movement, and the result goes to
//! `codimate-export`.
//!
//! Deliberately thin. The diff used to live here, which put the semantic
//! centre of the product inside a bindings crate — unreachable by any other
//! frontend, and behind a PyO3 build for anyone wanting to work on it.
//! Anything that decides what an animation *means* belongs in
//! `codimate-reconcile`; the only thing that belongs here is turning Python
//! values into Rust ones.

use codimate_export::{export_mp4, write_png, ExportConfig};
use codimate_layout::Viewport;
use codimate_reconcile::{self as reconcile, Focus, Shape};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// One shape as Python sends it: a dict of flat fields.
///
/// A near-duplicate of `reconcile::Shape`, and deliberately so — that struct
/// must not know PyO3 exists. Restating the fields here is what "the boundary
/// is a data structure, not an API" costs, and it is worth paying: the payload
/// stays serializable, testable without Python, and open to another frontend.
#[derive(FromPyObject)]
#[pyo3(from_item_all)]
struct PyShape {
    item: String,
    kind: String,
    x: f32,
    y: f32,
    x2: f32,
    y2: f32,
    w: f32,
    h: f32,
    r: f32,
    color: String,
    edge: String,
    edge_w: f32,
    points: Vec<f32>,
    text: String,
    size: f32,
    layer: i32,
    opacity: f32,
}

impl From<PyShape> for Shape {
    fn from(s: PyShape) -> Self {
        Shape {
            item: s.item,
            kind: s.kind,
            x: s.x,
            y: s.y,
            x2: s.x2,
            y2: s.y2,
            w: s.w,
            h: s.h,
            r: s.r,
            color: s.color,
            edge: s.edge,
            edge_w: s.edge_w,
            points: s.points,
            text: s.text,
            size: s.size,
            layer: s.layer,
            opacity: s.opacity,
        }
    }
}

/// What one Scene's camera is aimed at, as Python sends it.
#[derive(FromPyObject)]
#[pyo3(from_item_all)]
struct PyFocus {
    names: Vec<String>,
    pad: f32,
    min_size: f32,
    fixed: Vec<String>,
}

impl From<PyFocus> for Focus {
    fn from(f: PyFocus) -> Self {
        Focus {
            names: f.names,
            pad: f.pad,
            min_size: f.min_size,
            fixed: f.fixed,
        }
    }
}

/// Authoring mistakes reach Python as `ValueError`, which is what they are.
fn py(error: reconcile::Error) -> PyErr {
    PyValueError::new_err(error.0)
}

/// Render an explanation to `output`.
///
/// `scenes` has one entry per Trace Event plus the opening Scene, so it is
/// always `durations.len() + 1` long.
#[pyfunction]
#[pyo3(signature = (scenes, cameras, rules, durations, output, width=1280.0, height=720.0, fps=30.0, scale=1.0))]
#[allow(clippy::too_many_arguments)]
fn render(
    scenes: Vec<Vec<PyShape>>,
    cameras: Vec<Option<PyFocus>>,
    rules: Vec<reconcile::Rule>,
    durations: Vec<f32>,
    output: String,
    width: f32,
    height: f32,
    fps: f32,
    scale: f32,
) -> PyResult<()> {
    if scale <= 0.0 {
        return Err(PyValueError::new_err("scale must be positive"));
    }

    let scenes: Vec<Vec<Shape>> = scenes
        .into_iter()
        .map(|scene| scene.into_iter().map(Shape::from).collect())
        .collect();

    let cameras: Vec<Option<Focus>> = cameras.into_iter().map(|c| c.map(Focus::from)).collect();
    let explanation =
        reconcile::explanation(&scenes, &cameras, &rules, &durations, (width, height))
            .map_err(py)?;

    // `pixel_scale` rasterizes at the larger size rather than upscaling
    // afterwards, so 1080p is genuinely drawn at 1080p.
    let config = ExportConfig::new(fps, Viewport::new(width, height)).pixel_scale(scale);

    export_mp4(&explanation, &config, &output)
        .map_err(|e| PyValueError::new_err(format!("export failed: {e:?}")))
}

/// Rasterize a single moment to a PNG.
///
/// Same scenes, same timing, same arithmetic as `render` — only one frame of
/// it. Debugging a frame by rendering the whole video and seeking into it
/// costs a minute for a look at one second.
#[pyfunction]
#[pyo3(signature = (scenes, cameras, rules, durations, seconds, output, width=1280.0, height=720.0, scale=1.0))]
#[allow(clippy::too_many_arguments)]
fn render_frame_png(
    scenes: Vec<Vec<PyShape>>,
    cameras: Vec<Option<PyFocus>>,
    rules: Vec<reconcile::Rule>,
    durations: Vec<f32>,
    seconds: f32,
    output: String,
    width: f32,
    height: f32,
    scale: f32,
) -> PyResult<()> {
    use codimate_animation::Playable;

    let scenes: Vec<Vec<Shape>> = scenes
        .into_iter()
        .map(|scene| scene.into_iter().map(Shape::from).collect())
        .collect();
    let cameras: Vec<Option<Focus>> = cameras.into_iter().map(|c| c.map(Focus::from)).collect();
    let explanation =
        reconcile::explanation(&scenes, &cameras, &rules, &durations, (width, height))
            .map_err(py)?;

    let viewport = Viewport::new(width, height);
    let scene = explanation.resolve_at(seconds);
    let layout = codimate_layout::layout_scene(scene, viewport);
    let frame = codimate_render::render_frame(explanation.name(), seconds, &layout);
    // Scaled the same way the video is, so a debug frame is not a different
    // picture from the one that ships — text placement in particular used to
    // differ, which is exactly the kind of bug this is for finding.
    let bitmap = codimate_render::rasterize_scaled(&frame, scale.max(1.0));

    write_png(&output, &bitmap).map_err(|e| PyValueError::new_err(format!("{e}")))
}

/// The easing the Engine applies between two moments.
///
/// Exposed so an author can draw or reason about pacing without writing a
/// second copy of the curve — a second copy can drift, and then a diagram
/// about Codimate stops being about Codimate.
#[pyfunction]
fn ease(t: f32) -> f32 {
    reconcile::ease(t)
}

/// How wide and tall a string will be when drawn — see `codimate-render`.
///
/// Exposed because an author has no canvas to ask, and the alternative is
/// rendering a frame and measuring the picture by hand.
#[pyfunction]
fn measure(text: &str, size: f32) -> (f32, f32) {
    codimate_render::measure_text(text, size)
}

/// How wide and tall a typeset formula will be at `size`.
///
/// The text counterpart of `measure`. Without it a formula cannot be laid out
/// beside words — which is why maths inside a caption had to be spelled in
/// ASCII, and looked it.
#[pyfunction]
fn measure_formula(latex: &str, size: f32) -> PyResult<(f32, f32)> {
    reconcile::formula_size(latex, size).map_err(py)
}

#[pymodule]
fn _codimate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(render, m)?)?;
    m.add_function(wrap_pyfunction!(render_frame_png, m)?)?;
    m.add_function(wrap_pyfunction!(ease, m)?)?;
    m.add_function(wrap_pyfunction!(measure, m)?)?;
    m.add_function(wrap_pyfunction!(measure_formula, m)?)?;
    Ok(())
}
