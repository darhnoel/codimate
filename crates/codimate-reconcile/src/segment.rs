//! The diff — motion is implied by identity.

use crate::*;

/// How much of a many-outlined Item is showing, and whether a pen is drawing
/// it.
///
/// Shared by `formula` and `svg`, which are the same animation over different
/// artwork: a list of outlines, revealed left to right, optionally traced.
pub(crate) struct Reveal {
    /// The reveal edge at t=0 and t=1, in outline counts.
    edge: (f32, f32),
    /// Opacity at t=0 and t=1.
    fade: (f32, f32),
    /// Pen width. Zero means the outlines simply fade in.
    pen: f32,
}

impl Reveal {
    /// `pen` is passed rather than read off the Shape because the two kinds
    /// keep it in different fields: a formula in `w`, an imported SVG in
    /// `size`, because `w` there is already the fit box.
    fn of(before: &Shape, after: &Shape, count: usize, pen: f32) -> Self {
        // `r` is the revealed fraction. Scaled to an outline count it becomes a
        // moving edge: outlines behind it are solid, the one under it is
        // part-faded, the rest are not there yet — so a drawing assembles
        // itself instead of appearing all at once. The edge travels past the
        // last outline by the overlap so the tail finishes fading rather than
        // snapping on.
        let span = count as f32 + REVEAL_OVERLAP;
        Reveal {
            edge: (before.r * span, after.r * span),
            fade: (before.opacity, after.opacity),
            pen,
        }
    }
}

/// Draw outlines as one Item, swept by the reveal edge and optionally traced
/// by a pen.
pub(crate) fn revealed(
    outlines: &[(Path, Animated<Style>)],
    show: Reveal,
    pen_ink: Color,
    path: Animated<Vec2>,
) -> Vec<Primitive> {
    let (edge_from, edge_to) = show.edge;
    let (fade_from, fade_to) = show.fade;
    let pen = show.pen;

    // How far this outline has got, 0 to 1. The edge is eased; a linear sweep
    // starts and stops abruptly, which is most of what makes a reveal look
    // mechanical. Plain opacity stays linear, matching `tween`.
    let progress = move |index: f32, t: f32| {
        let edge = edge_from + (edge_to - edge_from) * ease_in_out(t);
        ((edge - index) / REVEAL_OVERLAP).clamp(0.0, 1.0)
    };
    let fade = move |t: f32| fade_from + (fade_to - fade_from) * t;

    let mut out = Vec::with_capacity(outlines.len() * if pen > 0.0 { 2 } else { 1 });
    for (index, (outline, look)) in outlines.iter().enumerate() {
        let index = index as f32;

        if pen <= 0.0 {
            out.push(
                Primitive::new(Geometry::path(outline.clone().into_animated()))
                    .pos(path.clone())
                    .style(look.clone())
                    .opacity(Animated::new(move |t| fade(t) * progress(index, t))),
            );
            continue;
        }

        // The solid outline, arriving only at the end of its own trace.
        out.push(
            Primitive::new(Geometry::path(outline.clone().into_animated()))
                .pos(path.clone())
                .style(look.clone())
                .opacity(Animated::new(move |t| {
                    let u = progress(index, t);
                    fade(t) * ((u - (1.0 - FILL_TAKEOVER)) / FILL_TAKEOVER).clamp(0.0, 1.0)
                })),
        );

        // The pen, drawing that outline and then lifting.
        let traced = flatten(outline);
        out.push(
            Primitive::new(Geometry::path(Animated::new(move |t| {
                trim(&traced, progress(index, t))
            })))
            .pos(path.clone())
            .style(
                Style::new()
                    .fill(Color::TRANSPARENT)
                    .stroke(pen, pen_ink)
                    .into_animated(),
            )
            .opacity(Animated::new(move |t| {
                let u = progress(index, t);
                let lifting = ((1.0 - u) / FILL_TAKEOVER).clamp(0.0, 1.0);
                fade(t) * lifting
            })),
        );
    }
    out
}

/// One Item becomes one Primitive — except a formula or an imported SVG, which
/// become one per outline. They share a position and opacity, so they move as
/// a unit.
pub(crate) fn primitives(before: &Shape, after: &Shape, rules: &[Rule]) -> Result<Vec<Primitive>> {
    if before.kind != after.kind {
        return Err(Error(format!(
            "item {:?} changed kind from {:?} to {:?} — an Item keeps one shape kind",
            after.item, before.kind, after.kind
        )));
    }

    // A font that will not load is an authoring mistake, said here rather
    // than drawn quietly in the fallback font.
    for font in [&before.font, &after.font] {
        if !font.is_empty() {
            codimate_fonts::FontRegistry::global()
                .load(font)
                .map_err(|e| Error(format!("item {:?}: {e}", after.item)))?;
        }
    }

    let path = motion_path(before.anchor(), after.anchor(), rules, &after.item)?;
    let style = tween(before.style()?, after.style()?);
    let opacity = tween(before.opacity, after.opacity);

    // The rest of the Transform. Tweened like everything else, so a shape can
    // grow or turn between two moments without the author saying how.
    let scale = tween(
        Vec2::new(before.scale_x, before.scale_y),
        Vec2::new(after.scale_x, after.scale_y),
    );
    let spin = tween(before.rotate, after.rotate);
    let pivot = pivot_of(&after.pivot)?;

    if before.kind == "formula" {
        // `before`'s LaTeX, matching how text takes its value from the start of
        // the segment. An author who wants a formula read before it changes
        // emits a move and then a hold, exactly as they already do for text.
        let size = before.size;
        let ink = parse_color(&before.color)?;
        let outlines: Vec<(Path, Animated<Style>)> = formula_glyphs(&before.text)?
            .iter()
            .map(|g| {
                (
                    map_path(g, |v| Vec2::new(v.x * size, v.y * size)),
                    style.clone(),
                )
            })
            .collect();
        return Ok(revealed(
            &outlines,
            Reveal::of(before, after, outlines.len(), before.w.max(after.w)),
            ink,
            path,
        ));
    }

    // One Geometry, not an expansion — an image is a single thing. It is here
    // rather than in `geometry()` only because decoding can fail, and that
    // signature has no way to say so.
    if before.kind == "image" {
        let pixels = image_pixels(&before.text)?;
        let from = fitted_box(&pixels, (before.w, before.h));
        let to = fitted_box(&pixels, (after.w, after.h));
        return Ok(vec![Primitive::new(Geometry::Image {
            pixels,
            width: tween(from.0, to.0),
            height: tween(from.1, to.1),
        })
        .pos(path)
        .scale_xy(scale)
        .rotate(spin)
        .pivot(pivot)
        .style(style)
        .opacity(opacity)]);
    }

    if before.kind == "svg" {
        // Same shape as a formula: one Item, many outlines, revealed left to
        // right. The difference is that the artwork brought its own colours
        // (ADR 0014), so each outline carries a Style instead of sharing one.
        let art = svg_art(&before.text)?;
        let scale = fit_scale(art.size, (before.w, before.h));
        let authored = before.color.is_empty();
        let ink = if authored {
            Color::WHITE
        } else {
            parse_color(&before.color)?
        };

        let outlines: Vec<(Path, Animated<Style>)> = art
            .paths
            .iter()
            .map(|(p, own)| {
                let geometry = map_path(p, |v| Vec2::new(v.x * scale, v.y * scale));
                // An empty `color` means "as authored"; anything else is an
                // override the author asked for, and flattens the drawing.
                let look = match (authored, own) {
                    (true, Some(c)) => Style::new().fill(*c).into_animated(),
                    _ => style.clone(),
                };
                (geometry, look)
            })
            .collect();
        let pen = before.size.max(after.size);
        let mut out = revealed(
            &outlines,
            Reveal::of(before, after, outlines.len(), pen),
            ink,
            path.clone(),
        );

        // Labels are translated, not traced: each becomes a `Geometry::Text`
        // and is shaped at draw time by the same call that shapes every other
        // label in the frame. They cannot be pen-drawn, which is exactly true
        // of `scene.text` too, so it is consistent rather than surprising.
        let fade = tween(before.opacity, after.opacity);
        for label in &art.texts {
            let size = label.size * scale;
            // The Engine aligns left or centre; `text-anchor="end"` becomes a
            // left-aligned label starting one measured width earlier.
            let shift = if label.from_right {
                -codimate_render::measure_text(&label.content, size, "").0
            } else {
                0.0
            };
            let offset = Vec2::new(label.at.x * scale + shift, label.at.y * scale);
            let anchored = path.clone();
            let colour = match (authored, label.fill) {
                (true, Some(c)) => Style::new().fill(c).into_animated(),
                _ => style.clone(),
            };
            out.push(
                Primitive::new(Geometry::Text {
                    text: label.content.clone().into_animated(),
                    font_size: size.into_animated(),
                    align: label.align,
                    font: String::new(),
                })
                .pos(Animated::new(move |t| {
                    let base = anchored.resolve(t);
                    Vec2::new(base.x + offset.x, base.y + offset.y)
                }))
                .style(colour)
                .opacity(fade.clone()),
            );
        }
        return Ok(out);
    }

    Ok(vec![Primitive::new(before.geometry(after))
        .pos(path)
        .scale_xy(scale)
        .rotate(spin)
        .pivot(pivot)
        .style(style)
        .opacity(opacity)])
}

/// One segment's Scene: every Item that exists in `before` or `after`, tweened.
pub fn build_segment(before: &[Shape], after: &[Shape], rules: &[Rule]) -> Result<Scene> {
    let by_item = |s: &[Shape]| -> HashMap<String, Shape> {
        s.iter().map(|sh| (sh.item.clone(), sh.clone())).collect()
    };
    let (b, a) = (by_item(before), by_item(after));

    // Draw order: layer, then Item — stable and independent of dict ordering.
    let mut items: Vec<&Shape> = after.iter().chain(before.iter()).collect();
    items.sort_by(|x, y| (x.layer, &x.item).cmp(&(y.layer, &y.item)));
    items.dedup_by(|x, y| x.item == y.item);

    let changing: Vec<(&Shape, bool)> = items
        .iter()
        .map(|s| {
            let moved = match (b.get(&s.item), a.get(&s.item)) {
                (Some(from), Some(to)) => from != to,
                _ => true, // entering or leaving
            };
            (*s, moved)
        })
        .collect();
    let clocks = clocks(&changing, rules)?;

    let mut scene = Scene::new();

    for shape in items {
        let prims = match (b.get(&shape.item), a.get(&shape.item)) {
            // Present in both — the ordinary case.
            (Some(from), Some(to)) => primitives(from, to, rules)?,

            // Entering — fade in where it lands, no travel.
            (None, Some(to)) => {
                let mut faded = to.clone();
                faded.opacity = 0.0;
                primitives(&faded, to, rules)?
            }

            // Exiting — fade out where it was.
            (Some(from), None) => {
                let mut faded = from.clone();
                faded.opacity = 0.0;
                primitives(from, &faded, rules)?
            }

            (None, None) => unreachable!("item came from one of the two lists"),
        };
        let clock = clocks.get(&shape.item);
        for prim in prims {
            scene = scene.add(match clock {
                Some(c) => prim.ease(c.clone()),
                None => prim,
            });
        }
    }

    Ok(scene)
}
