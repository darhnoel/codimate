//! Formulas, pictures and SVG artwork, each loaded once per process.

use crate::*;

/// Typeset one LaTeX string into glyph outlines, centred, in em units.
///
/// Typeset **once per process**, not once per segment. `codimate-math` already
/// caches the Typst SVG on disk by content hash, but re-parsing that SVG into
/// paths for every segment of an explanation is still waste — an explanation
/// has tens of segments and the answer never changes.
///
/// Cached in em units with the fill left off, so one entry serves every size
/// and colour the same formula is ever drawn at.
pub fn formula_glyphs(latex: &str) -> Result<Arc<Vec<Path>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<Vec<Path>>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Some(hit) = cache.lock().unwrap().get(latex) {
        return Ok(hit.clone());
    }

    let block = codimate_math::formula(latex, Color::WHITE).map_err(|e| {
        Error(match e {
            codimate_math::FormulaError::TypstSpawn(_) => format!(
                "could not run `typst`, which Codimate uses to typeset {latex:?}. \
                 Install it (macOS: `brew install typst`) — like ffmpeg it is an \
                 external tool, not a Python dependency."
            ),
            codimate_math::FormulaError::Mitex(m) => {
                format!("could not read the LaTeX {latex:?}: {m}")
            }
            codimate_math::FormulaError::TypstCompile(m) => {
                format!("could not typeset {latex:?}:\n{m}")
            }
            codimate_math::FormulaError::Svg(m) => {
                format!("could not read the typeset {latex:?}: {m}")
            }
            // Only an imported file can raise these; typst's own output is
            // never read from disk and never contains text elements.
            codimate_math::FormulaError::SvgRead(m) | codimate_math::FormulaError::SvgText(m) => m,
        })
    })?;

    let paths: Vec<Path> = block.glyphs.iter().map(|g| g.resolve(0.0).path).collect();

    // Centre on the block's own bounding box. `GlyphBlock` reports width and
    // height but keeps the glyphs at their absolute page coordinates, so the
    // origin has to be recovered here.
    let (mut min_x, mut min_y) = (f32::MAX, f32::MAX);
    let (mut max_x, mut max_y) = (f32::MIN, f32::MIN);
    for path in &paths {
        if let Some((x0, y0, x1, y1)) = path.bounding_box() {
            min_x = min_x.min(x0);
            min_y = min_y.min(y0);
            max_x = max_x.max(x1);
            max_y = max_y.max(y1);
        }
    }
    if min_x > max_x {
        return Err(Error(format!("{latex:?} typeset to nothing visible")));
    }
    let (cx, cy) = ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);

    let mut centred: Vec<Path> = paths
        .iter()
        .map(|p| {
            map_path(p, |v| {
                Vec2::new((v.x - cx) / FORMULA_EM, (v.y - cy) / FORMULA_EM)
            })
        })
        .collect();

    // Left to right, so revealing part of a formula is a wipe in reading
    // order. Typst emits glyphs in layout order, which is close but not a
    // promise — and for a fraction it puts the whole numerator before the
    // denominator, which would reveal the equation in two passes.
    centred.sort_by(|a, b| {
        let key = |p: &Path| p.bounding_box().map(|(x0, ..)| x0).unwrap_or(0.0);
        key(a)
            .partial_cmp(&key(b))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let shared = Arc::new(centred);
    cache
        .lock()
        .unwrap()
        .insert(latex.to_string(), shared.clone());
    Ok(shared)
}

/// Decode, premultiply and cache a picture — once per path, for the life of
/// the process.
///
/// Cached for the reason the formula glyphs and SVG artwork are: a 1,200-frame
/// render must not decode the same megabyte 1,200 times. As with those, it
/// also means editing the file mid-session and re-rendering shows the old
/// picture.
pub(crate) fn image_pixels(file: &str) -> Result<Arc<Pixels>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<Pixels>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Some(hit) = cache.lock().unwrap().get(file) {
        return Ok(hit.clone());
    }

    let bytes = std::fs::read(file)
        .map_err(|e| Error(format!("could not read the image {file:?}: {e}")))?;

    // By content, not by extension: a `.png` that is really a JPEG is a
    // mistake worth surviving, and the magic numbers are unambiguous.
    let decoded = if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        decode_png(&bytes)
    } else if bytes.starts_with(&[0xFF, 0xD8]) {
        decode_jpeg(&bytes)
    } else {
        return Err(Error(format!(
            "{file:?} is not a PNG or a JPEG — those are the two Codimate reads"
        )));
    }
    .map_err(|m| Error(format!("could not read the image {file:?}: {m}")))?;

    let shared = Arc::new(decoded);
    cache
        .lock()
        .unwrap()
        .insert(file.to_string(), shared.clone());
    Ok(shared)
}

/// Straight RGBA to premultiplied, which is what tiny-skia blits.
pub(crate) fn premultiplied(width: u32, height: u32, rgba: Vec<u8>) -> Pixels {
    let mut rgba = rgba;
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3] as u32;
        if a == 255 {
            continue;
        }
        for c in 0..3 {
            px[c] = ((px[c] as u32 * a + 127) / 255) as u8;
        }
    }
    Pixels {
        width,
        height,
        rgba,
    }
}

pub(crate) fn decode_png(bytes: &[u8]) -> std::result::Result<Pixels, String> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    // Normalise greyscale, palette and 16-bit down to 8-bit colour, and expand
    // transparency into a real alpha channel, so the rest of this sees one
    // layout instead of six.
    decoder.set_transformations(
        png::Transformations::normalize_to_color8() | png::Transformations::ALPHA,
    );
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buffer = vec![0; reader.output_buffer_size().unwrap_or(0)];
    let info = reader.next_frame(&mut buffer).map_err(|e| e.to_string())?;
    buffer.truncate(info.buffer_size());

    let rgba = match info.color_type {
        png::ColorType::Rgba => buffer,
        png::ColorType::GrayscaleAlpha => buffer
            .chunks_exact(2)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        other => return Err(format!("unsupported PNG colour type {other:?}")),
    };
    Ok(premultiplied(info.width, info.height, rgba))
}

pub(crate) fn decode_jpeg(bytes: &[u8]) -> std::result::Result<Pixels, String> {
    let mut decoder = jpeg_decoder::Decoder::new(std::io::Cursor::new(bytes));
    let pixels = decoder.decode().map_err(|e| e.to_string())?;
    let info = decoder.info().ok_or("no JPEG header")?;

    // JPEG has no alpha, so every pixel is opaque and premultiplying is a
    // no-op — the conversion is only about layout.
    let rgba: Vec<u8> = match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => pixels
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        jpeg_decoder::PixelFormat::L8 => pixels.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        other => return Err(format!("unsupported JPEG pixel format {other:?}")),
    };
    Ok(Pixels {
        width: info.width as u32,
        height: info.height as u32,
        rgba,
    })
}

/// An imported SVG, ready to draw.
///
/// Paths are centred on the artwork's own bounding box and scaled so its
/// longest side is 1.0, which makes `size` mean the same thing whatever units
/// the file was drawn in — a 24-unit icon and a 1000-unit diagram both arrive
/// the same size (ADR 0014).
pub(crate) struct SvgArt {
    pub(crate) paths: Vec<(Path, Option<Color>)>,
    /// Labels, in the same normalised space as the paths. Not glyph outlines:
    /// these become `Geometry::Text` and are shaped at draw time by the call
    /// that shapes every other label in the frame (ADR 0014).
    pub(crate) texts: Vec<SvgLabel>,
    /// Normalised extent. The longer side is 1.0.
    pub(crate) size: (f32, f32),
}

pub(crate) struct SvgLabel {
    pub(crate) content: String,
    /// The baseline anchor, normalised. SVG places text on its baseline and so
    /// does the Engine, so this needs no conversion.
    pub(crate) at: Vec2,
    pub(crate) size: f32,
    pub(crate) fill: Option<Color>,
    pub(crate) align: TextAlign,
    /// True when the file said `text-anchor="end"`. The Engine has no right
    /// alignment, so the label is drawn left-aligned from a point shifted back
    /// by its own measured width.
    pub(crate) from_right: bool,
}

/// How much to scale normalised artwork so it fits inside `box`.
///
/// Aspect is always preserved: the drawing fits inside the box the author
/// named, whichever way round it is. A zero or missing box falls back to the
/// artwork's own normalised size, which keeps a forgotten `size=` visible
/// rather than invisible.
pub(crate) fn fit_scale(art: (f32, f32), fit: (f32, f32)) -> f32 {
    let (w, h) = fit;
    if w <= 0.0 && h <= 0.0 {
        return 1.0;
    }
    let by_w = if w > 0.0 { w / art.0 } else { f32::MAX };
    let by_h = if h > 0.0 { h / art.1 } else { f32::MAX };
    by_w.min(by_h)
}

/// The size a picture is actually drawn at, fitted inside the box the author
/// named with its aspect kept — the same rule `scene.svg` follows, so the two
/// imports size the same way.
pub(crate) fn fitted_box(pixels: &Pixels, fit: (f32, f32)) -> (f32, f32) {
    let natural = (pixels.width.max(1) as f32, pixels.height.max(1) as f32);
    if fit.0 <= 0.0 && fit.1 <= 0.0 {
        return natural;
    }
    let by_w = if fit.0 > 0.0 {
        fit.0 / natural.0
    } else {
        f32::MAX
    };
    let by_h = if fit.1 > 0.0 {
        fit.1 / natural.1
    } else {
        f32::MAX
    };
    let scale = by_w.min(by_h);
    (natural.0 * scale, natural.1 * scale)
}

/// Read, parse and normalise an SVG file — once per path, for the life of the
/// process.
///
/// Cached for the same reason formula glyphs are: a 1,200-frame render must
/// not re-parse the same file 1,200 times. It also means editing the file
/// mid-session and re-rendering shows the old drawing.
pub(crate) fn svg_art(file: &str) -> Result<Arc<SvgArt>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<SvgArt>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Some(hit) = cache.lock().unwrap().get(file) {
        return Ok(hit.clone());
    }

    let source = std::fs::read_to_string(file)
        .map_err(|e| Error(format!("could not read the SVG {file:?}: {e}")))?;
    let imported = codimate_math::import_svg(&source).map_err(|e| {
        Error(match e {
            codimate_math::FormulaError::SvgText(m) => format!("{file:?}: {m}"),
            other => format!("could not read the SVG {file:?}: {other:?}"),
        })
    })?;

    let (mut min_x, mut min_y) = (f32::MAX, f32::MAX);
    let (mut max_x, mut max_y) = (f32::MIN, f32::MIN);
    for item in &imported {
        if let Some((x0, y0, x1, y1)) = item.path.bounding_box() {
            min_x = min_x.min(x0);
            min_y = min_y.min(y0);
            max_x = max_x.max(x1);
            max_y = max_y.max(y1);
        }
    }
    if min_x > max_x {
        return Err(Error(format!("{file:?} has nothing visible in it")));
    }

    let (cx, cy) = ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
    let longest = (max_x - min_x).max(max_y - min_y).max(1e-6);

    let labels = codimate_math::import_svg_text(&source).map_err(|e| match e {
        codimate_math::FormulaError::SvgText(m) => Error(format!("{file:?}: {m}")),
        other => Error(format!("could not read the SVG {file:?}: {other:?}")),
    })?;

    let texts: Vec<SvgLabel> = labels
        .into_iter()
        .map(|l| SvgLabel {
            content: l.content,
            at: Vec2::new((l.x - cx) / longest, (l.y - cy) / longest),
            size: l.size / longest,
            fill: l.fill,
            align: if l.anchor == "middle" {
                TextAlign::Center
            } else {
                TextAlign::Left
            },
            from_right: l.anchor == "end",
        })
        .collect();

    let mut paths: Vec<(Path, Option<Color>)> = imported
        .iter()
        .map(|item| {
            (
                map_path(&item.path, |v| {
                    Vec2::new((v.x - cx) / longest, (v.y - cy) / longest)
                }),
                item.fill,
            )
        })
        .collect();

    // Left to right, so the pen draws across the picture. Document order was
    // the alternative and is better for artwork a person built stroke by
    // stroke, but most files come out of a tool where the saved order is
    // arbitrary — a Mermaid diagram emits arrows before boxes (ADR 0014).
    paths.sort_by(|a, b| {
        let key = |p: &Path| p.bounding_box().map(|(x0, ..)| x0).unwrap_or(0.0);
        key(&a.0)
            .partial_cmp(&key(&b.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let art = Arc::new(SvgArt {
        paths,
        texts,
        size: ((max_x - min_x) / longest, (max_y - min_y) / longest),
    });
    cache.lock().unwrap().insert(file.to_string(), art.clone());
    Ok(art)
}

/// How wide and tall a typeset formula is at `size`.
///
/// Lives here rather than in a frontend because it reads the same cached glyph
/// outlines the renderer draws, and because framing a formula — the thing a
/// camera would need — is scene geometry, not marshalling.
pub fn formula_size(latex: &str, size: f32) -> Result<(f32, f32)> {
    let glyphs = formula_glyphs(latex)?;
    let (mut x0, mut y0) = (f32::MAX, f32::MAX);
    let (mut x1, mut y1) = (f32::MIN, f32::MIN);
    for path in glyphs.iter() {
        if let Some((a, b, c, d)) = path.bounding_box() {
            x0 = x0.min(a);
            y0 = y0.min(b);
            x1 = x1.max(c);
            y1 = y1.max(d);
        }
    }
    if x0 > x1 {
        return Ok((0.0, 0.0));
    }
    // The cache holds glyphs centred and in em units, so scaling by `size` is
    // the same arithmetic the renderer does.
    Ok(((x1 - x0) * size, (y1 - y0) * size))
}
