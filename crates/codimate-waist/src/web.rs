//! The WebAssembly transport: the JavaScript Skin's crossing (ADR 0032).
//!
//! Python builds a film's scenes and saves them (`Explanation.export_web`); this
//! crate reads that file and draws any moment of it to RGBA pixels for a
//! `<canvas>`. It translates and nothing else: the same `codimate-reconcile`,
//! layout and renderer draw the film here as on the server.

use codimate_animation::Playable;
use codimate_fonts::FontRegistry;
use codimate_layout::Viewport;
use codimate_reconcile::{self as reconcile, Focus, Rule, Shape};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

/// The saved film, as `Explanation.export_web` writes it.
#[derive(Deserialize)]
struct Timeline {
    version: u32,
    width: f32,
    height: f32,
    durations: Vec<f32>,
    scenes: Vec<Vec<WebShape>>,
    cameras: Vec<Option<WebFocus>>,
    rules: Vec<Rule>,
}

/// One shape, field for field the payload `codimate-py` reads: the boundary is
/// a data structure, so a second frontend restates it rather than sharing it.
#[derive(Deserialize)]
#[serde(default)]
struct WebShape {
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
    font: String,
    layer: i32,
    opacity: f32,
    scale_x: f32,
    scale_y: f32,
    rotate: f32,
    pivot: String,
}

impl Default for WebShape {
    fn default() -> Self {
        WebShape {
            item: String::new(),
            kind: String::new(),
            x: 0.0,
            y: 0.0,
            x2: 0.0,
            y2: 0.0,
            w: 0.0,
            h: 0.0,
            r: 0.0,
            color: String::new(),
            edge: String::new(),
            edge_w: 0.0,
            points: Vec::new(),
            text: String::new(),
            size: 0.0,
            font: String::new(),
            layer: 0,
            opacity: 1.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotate: 0.0,
            pivot: String::new(),
        }
    }
}

impl From<WebShape> for Shape {
    fn from(s: WebShape) -> Self {
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
            font: s.font,
            layer: s.layer,
            opacity: s.opacity,
            scale_x: s.scale_x,
            scale_y: s.scale_y,
            rotate: s.rotate,
            pivot: s.pivot,
        }
    }
}

#[derive(Deserialize)]
struct WebFocus {
    names: Vec<String>,
    pad: f32,
    min_size: f32,
    fixed: Vec<String>,
}

impl From<WebFocus> for Focus {
    fn from(f: WebFocus) -> Self {
        Focus {
            names: f.names,
            pad: f.pad,
            min_size: f.min_size,
            fixed: f.fixed,
        }
    }
}

/// The Waist (ADR 0032): one authoring operation, data in and data out, the
/// same function every Skin reaches the Authoring Core through.
#[wasm_bindgen]
pub fn call(op: &str, data: &str) -> String {
    codimate_author::call(op, data)
}

/// Register a font's bytes under the name the film's scenes use for it (its
/// `font` field, usually the path it had on the machine that saved the film).
/// Call it for each of a film's fonts before building the [`Film`].
#[wasm_bindgen(js_name = provideFont)]
pub fn provide_font(spec: &str, bytes: Vec<u8>) -> Result<(), JsError> {
    FontRegistry::global()
        .provide(spec, bytes)
        .map(|_| ())
        .map_err(|e| JsError::new(&e))
}

/// The font names a saved film's scenes use, read without building the film,
/// so a page knows which files to fetch for [`provide_font`].
#[wasm_bindgen(js_name = filmFonts)]
pub fn film_fonts(json: &str) -> Result<Vec<String>, JsError> {
    #[derive(Deserialize)]
    struct Fonts {
        fonts: Vec<String>,
    }
    serde_json::from_str::<Fonts>(json)
        .map(|f| f.fonts)
        .map_err(|e| JsError::new(&e.to_string()))
}

/// A film built once, to draw any moment of on request.
#[wasm_bindgen]
pub struct Film {
    explanation: reconcile::Explanation,
    viewport: Viewport,
    duration: f32,
    /// The last frame drawn, kept so the page reads it in place: one buffer for
    /// the whole film, not a new 8 MB copy for every frame at 60 fps.
    pixels: Vec<u8>,
}

#[wasm_bindgen]
impl Film {
    /// Build the film from the JSON `Explanation.export_web` wrote.
    #[wasm_bindgen(constructor)]
    pub fn new(json: &str) -> Result<Film, JsError> {
        let t: Timeline = serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        if t.version != 1 {
            return Err(JsError::new(&format!("unknown film version {}", t.version)));
        }
        let duration = t.durations.iter().sum();
        let scenes: Vec<Vec<Shape>> = t
            .scenes
            .into_iter()
            .map(|scene| scene.into_iter().map(Shape::from).collect())
            .collect();
        let cameras: Vec<Option<Focus>> =
            t.cameras.into_iter().map(|c| c.map(Focus::from)).collect();
        let explanation = reconcile::explanation(
            scenes,
            &cameras,
            &t.rules,
            &t.durations,
            (t.width, t.height),
        )
        .map_err(|e| JsError::new(&e.0))?;
        Ok(Film {
            explanation,
            viewport: Viewport::new(t.width, t.height),
            duration,
            pixels: Vec::new(),
        })
    }

    /// The film's length, in seconds.
    #[wasm_bindgen(getter)]
    pub fn duration(&self) -> f32 {
        self.duration
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> f32 {
        self.viewport.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> f32 {
        self.viewport.height
    }

    /// Draw the moment at `seconds`, `scale` times the canvas size, with the
    /// same arithmetic the video and `frame_at` use. Returns where the RGBA
    /// pixels are in this module's memory (`pixelsLength` bytes long): read
    /// them there, in place, until the next `draw`.
    pub fn draw(&mut self, seconds: f32, scale: f32) -> *const u8 {
        let scene = self.explanation.resolve_at(seconds);
        let layout = codimate_layout::layout_scene(scene, self.viewport);
        let frame = codimate_render::render_frame(self.explanation.name(), seconds, &layout);
        self.pixels = codimate_render::rasterize_scaled(&frame, scale.max(1.0)).rgba;
        self.pixels.as_ptr()
    }

    /// How many bytes the last `draw` wrote: width × height × 4 at its scale.
    #[wasm_bindgen(getter, js_name = pixelsLength)]
    pub fn pixels_length(&self) -> usize {
        self.pixels.len()
    }
}
