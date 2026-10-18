//! The camera: what to look at, aimed by name.

use crate::*;

/// What to look at. Aimed by name, never by coordinate (ADR 0009).
#[derive(Clone, Debug, Default)]
pub struct Focus {
    /// Items to frame. Empty means the whole canvas.
    pub names: Vec<String>,
    /// Breathing room around them, in canvas units.
    pub pad: f32,
    /// Never frame tighter than this, so a small shape is not magnified into
    /// abstraction. A statement about the picture, not about the arithmetic.
    pub min_size: f32,
    /// Shapes that ignore the camera, by name — the overlay.
    pub fixed: Vec<String>,
}

/// How far a shape reaches from its anchor, as `(left, top, right, bottom)`.
pub(crate) fn bounds(shape: &Shape) -> (f32, f32, f32, f32) {
    let (w, h) = match shape.kind.as_str() {
        "circle" => (shape.r * 2.0, shape.r * 2.0),
        "text" => codimate_render::measure_text(&shape.text, shape.size, &shape.font),
        "formula" => formula_size(&shape.text, shape.size).unwrap_or((0.0, 0.0)),
        // The fitted box, not the box asked for: a wide drawing in a square
        // `size` does not fill the square, and framing the square would leave
        // the camera looking at empty air above and below.
        "image" => image_pixels(&shape.text)
            .map(|p| fitted_box(&p, (shape.w, shape.h)))
            .unwrap_or((0.0, 0.0)),
        "svg" => svg_art(&shape.text)
            .map(|art| {
                let s = fit_scale(art.size, (shape.w, shape.h));
                (art.size.0 * s, art.size.1 * s)
            })
            .unwrap_or((0.0, 0.0)),
        // A Catmull-Rom curve can bulge a little past its samples between
        // them, but never far, and framing by the samples is what an author
        // means by "frame this curve".
        "polygon" | "curve" => {
            let xs: Vec<f32> = shape.points.iter().step_by(2).copied().collect();
            let ys: Vec<f32> = shape.points.iter().skip(1).step_by(2).copied().collect();
            if xs.is_empty() {
                return (shape.x, shape.y, shape.x, shape.y);
            }
            return (
                xs.iter().cloned().fold(f32::MAX, f32::min),
                ys.iter().cloned().fold(f32::MAX, f32::min),
                xs.iter().cloned().fold(f32::MIN, f32::max),
                ys.iter().cloned().fold(f32::MIN, f32::max),
            );
        }
        // The whole ellipse, not the swept part: an arc that grows would
        // otherwise resize the camera as it sweeps.
        "arc" => (shape.w, shape.h),
        "line" => {
            let (x0, x1) = (shape.x.min(shape.x2), shape.x.max(shape.x2));
            let (y0, y1) = (shape.y.min(shape.y2), shape.y.max(shape.y2));
            return (x0, y0, x1, y1);
        }
        _ => (shape.w, shape.h),
    };
    (
        shape.x - w / 2.0,
        shape.y - h / 2.0,
        shape.x + w / 2.0,
        shape.y + h / 2.0,
    )
}

/// Where the camera sits and how far in, for one Scene.
///
/// Returns the centre to look at and the zoom that frames it. Zoom is capped
/// by `min_size`, because `focus` on a three-pixel dot is a two-hundred-times
/// magnification of nothing.
pub(crate) fn aim(
    shapes: &[Shape],
    focus: &Focus,
    viewport: (f32, f32),
) -> Result<(f32, f32, f32)> {
    if focus.names.is_empty() {
        return Ok((viewport.0 / 2.0, viewport.1 / 2.0, 1.0));
    }

    let (mut x0, mut y0) = (f32::MAX, f32::MAX);
    let (mut x1, mut y1) = (f32::MIN, f32::MIN);
    let mut found = 0;
    for shape in shapes.iter().filter(|s| focus.names.contains(&s.item)) {
        let (a, b, c, d) = bounds(shape);
        x0 = x0.min(a);
        y0 = y0.min(b);
        x1 = x1.max(c);
        y1 = y1.max(d);
        found += 1;
    }

    if found == 0 {
        // A camera aimed at nothing is the failure this design exists to
        // prevent, so it is an error rather than a silent wide shot.
        return Err(Error(format!(
            "focus({}) — no shape by that name in this scene",
            focus.names.join(", ")
        )));
    }

    let want_w = (x1 - x0 + focus.pad * 2.0).max(focus.min_size);
    let want_h = (y1 - y0 + focus.pad * 2.0).max(focus.min_size);
    // Fit the whole box: the tighter of the two axes decides.
    let zoom = (viewport.0 / want_w).min(viewport.1 / want_h).max(1.0);

    Ok(((x0 + x1) / 2.0, (y0 + y1) / 2.0, zoom))
}

/// Move a Scene's shapes into the camera's view.
///
/// Applied to the flat payload rather than to built primitives, which is only
/// possible because the payload is a flat union: one pass over a handful of
/// floats covers every kind, sizes and positions alike.
///
/// Each Scene is framed with its own camera *before* the diff sees it, so a
/// camera that moves between two moments is simply two sets of coordinates
/// that differ — and the existing tween animates it. Camera movement needs no
/// machinery of its own.
pub(crate) fn framed(
    shapes: Vec<Shape>,
    focus: Option<&Focus>,
    viewport: (f32, f32),
) -> Result<Vec<Shape>> {
    let Some(focus) = focus else {
        return Ok(shapes);
    };
    let (cx, cy, zoom) = aim(&shapes, focus, viewport)?;
    let (sx, sy) = (viewport.0 / 2.0, viewport.1 / 2.0);

    Ok(shapes
        .into_iter()
        .map(|mut s| {
            if focus.fixed.iter().any(|f| s.item.starts_with(f.as_str())) {
                return s; // the overlay: pinned to the screen
            }
            s.x = (s.x - cx) * zoom + sx;
            s.y = (s.y - cy) * zoom + sy;
            s.x2 = (s.x2 - cx) * zoom + sx;
            s.y2 = (s.y2 - cy) * zoom + sy;
            for (i, value) in s.points.iter_mut().enumerate() {
                *value = if i % 2 == 0 {
                    (*value - cx) * zoom + sx
                } else {
                    (*value - cy) * zoom + sy
                };
            }
            s.w *= zoom;
            s.h *= zoom;
            s.r *= zoom;
            s.size *= zoom;
            s
        })
        .collect())
}
