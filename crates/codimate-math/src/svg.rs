use crate::FormulaError;
use codimate_core::{Color, PathNode, Segment, Vec2};

/// One path out of an imported SVG, with the colour it was authored in.
///
/// `fill` is `None` when the file gave the path no paint this build can read —
/// a pattern, say. The caller decides what to do about it.
pub struct SvgPath {
    pub path: codimate_core::Path,
    pub fill: Option<Color>,
}

/// Theme paints use a deliberately small, deterministic SVG extension:
/// six-digit hexadecimal RGB colors in `data-dark-fill`.
pub(crate) fn parse_hex_color(value: &str) -> Result<Color, FormulaError> {
    let digits = value.strip_prefix('#').filter(|digits| digits.len() == 6);
    let Some(digits) = digits else {
        return Err(FormulaError::Svg(format!(
            "data-dark-fill must be a six-digit hex color, got {value:?}"
        )));
    };
    let rgb = u32::from_str_radix(digits, 16)
        .map_err(|_| FormulaError::Svg(format!("invalid data-dark-fill color {value:?}")))?;
    Ok(Color {
        r: ((rgb >> 16) & 0xff) as f32 / 255.0,
        g: ((rgb >> 8) & 0xff) as f32 / 255.0,
        b: (rgb & 0xff) as f32 / 255.0,
        a: 1.0,
    })
}

/// Read an SVG's geometry, keeping each path's own colour (ADR 0014).
///
/// This is the same stage the formula pipeline ends with — LaTeX becomes SVG
/// and then paths — with two differences: the source is a file rather than
/// typst's output, and the artwork's own fills are kept instead of being
/// replaced by one ink.
///
/// Geometry only. `usvg` is built without its `text` feature and drops
/// `<text>` while parsing, so labels are read separately by
/// [`crate::import_svg_text`].
pub fn import_svg(svg: &str) -> Result<Vec<SvgPath>, FormulaError> {
    let opts = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &opts).map_err(|e| FormulaError::Svg(e.to_string()))?;
    let mut out = Vec::new();
    collect_svg(tree.root(), &mut out);
    Ok(out)
}

/// An attribute on this element or the nearest ancestor that sets it — SVG
/// presentation attributes inherit, and `font-size` is usually on the root.
pub(crate) fn inherited_str(node: &roxmltree::Node, name: &str) -> Option<String> {
    node.ancestors()
        .find_map(|n| n.attribute(name))
        .map(|v| v.trim().to_string())
}

pub(crate) fn inherited(node: &roxmltree::Node, name: &str) -> Option<f32> {
    let raw = inherited_str(node, name)?;
    // `12`, `12px` and `12pt` all appear; anything with other units is left to
    // the default rather than guessed at.
    let digits: String = raw
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    digits.parse().ok()
}

/// `#rgb`, `#rrggbb` or `none`. Anything else — a named colour, a gradient
/// reference — falls back to the caller's choice rather than being invented.
pub(crate) fn parse_css_colour(raw: &str) -> Option<Color> {
    let hex = raw.trim().strip_prefix('#')?;
    let (r, g, b) = match hex.len() {
        3 => {
            let d = |i: usize| u8::from_str_radix(&hex[i..i + 1].repeat(2), 16).ok();
            (d(0)?, d(1)?, d(2)?)
        }
        6 => {
            let d = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
            (d(0)?, d(2)?, d(4)?)
        }
        _ => return None,
    };
    Some(Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    })
}

fn collect_svg(group: &usvg::Group, out: &mut Vec<SvgPath>) {
    for node in group.children() {
        match node {
            usvg::Node::Path(path) => {
                if !path.is_visible() {
                    continue;
                }
                let segments = extract_segments(path);
                if segments.is_empty() {
                    continue;
                }
                out.push(SvgPath {
                    path: codimate_core::Path {
                        segments,
                        closed: false,
                    },
                    fill: authored_colour(path),
                });
            }
            usvg::Node::Group(g) => collect_svg(g, out),
            _ => {}
        }
    }
}

/// The colour a path was drawn in.
///
/// A stroke-only path reports its stroke, because `extract_segments` turned
/// that stroke into a fillable outline and the outline should be the colour
/// the stroke was. A gradient collapses to its first stop: `Style` carries one
/// flat colour, and one of the real colours beats refusing the file.
pub(crate) fn authored_colour(path: &usvg::Path) -> Option<Color> {
    fn from_paint(paint: &usvg::Paint, opacity: f32) -> Option<Color> {
        let (r, g, b) = match paint {
            usvg::Paint::Color(c) => (c.red, c.green, c.blue),
            usvg::Paint::LinearGradient(g) => {
                let s = g.stops().first()?.color();
                (s.red, s.green, s.blue)
            }
            usvg::Paint::RadialGradient(g) => {
                let s = g.stops().first()?.color();
                (s.red, s.green, s.blue)
            }
            usvg::Paint::Pattern(_) => return None,
        };
        Some(Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: opacity,
        })
    }

    if let Some(fill) = path.fill() {
        return from_paint(fill.paint(), fill.opacity().get());
    }
    path.stroke()
        .and_then(|s| from_paint(s.paint(), s.opacity().get()))
}

pub(crate) fn svg_to_paths(svg: &str, fill: Color) -> Result<Vec<PathNode>, FormulaError> {
    let opts = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &opts).map_err(|e| FormulaError::Svg(e.to_string()))?;

    let mut nodes = Vec::new();
    collect_nodes(tree.root(), &mut nodes, fill);
    Ok(nodes)
}

fn collect_nodes(group: &usvg::Group, nodes: &mut Vec<PathNode>, fill: Color) {
    for node in group.children() {
        match node {
            usvg::Node::Path(path) => {
                if !path.is_visible() {
                    continue;
                }
                let segments = extract_segments(path);
                if segments.is_empty() {
                    continue;
                }
                nodes.push(
                    PathNode::new()
                        .path(codimate_core::Path {
                            segments,
                            closed: false,
                        })
                        .fill(fill),
                );
            }
            usvg::Node::Group(g) => collect_nodes(g, nodes, fill),
            _ => {}
        }
    }
}

pub(crate) fn extract_segments(path: &usvg::Path) -> Vec<Segment> {
    use usvg::tiny_skia_path::{PathSegment, Stroke};

    // A fraction bar, a radical's overbar and `\overline` come out of Typst as
    // *stroked* lines with no fill. Filling one directly draws nothing — a line
    // encloses no area — so outline the stroke and fill the outline instead.
    // Without this every `\frac` renders as a numerator and a denominator with
    // no bar between them.
    let data = match (path.fill(), path.stroke()) {
        (None, Some(s)) => {
            let mut stroke = Stroke::default();
            stroke.width = s.width().get();
            path.data().stroke(&stroke, 1.0)
        }
        _ => None,
    };
    let data = data.as_ref().unwrap_or(path.data());

    let transformed = match data.clone().transform(path.abs_transform()) {
        Some(p) => p,
        None => return Vec::new(),
    };

    let mut segments = Vec::new();
    let mut current = Vec2::new(0.0, 0.0);

    for seg in transformed.segments() {
        match seg {
            PathSegment::MoveTo(p) => {
                let pos = Vec2::new(p.x, p.y);
                segments.push(Segment::MoveTo(pos));
                current = pos;
            }
            PathSegment::LineTo(p) => {
                let pos = Vec2::new(p.x, p.y);
                segments.push(Segment::Line(current, pos));
                current = pos;
            }
            PathSegment::QuadTo(c, p) => {
                let ctrl = Vec2::new(c.x, c.y);
                let pos = Vec2::new(p.x, p.y);
                segments.push(Segment::Quad(current, ctrl, pos));
                current = pos;
            }
            PathSegment::CubicTo(c1, c2, p) => {
                let ctrl1 = Vec2::new(c1.x, c1.y);
                let ctrl2 = Vec2::new(c2.x, c2.y);
                let pos = Vec2::new(p.x, p.y);
                segments.push(Segment::Cubic(current, ctrl1, ctrl2, pos));
                current = pos;
            }
            PathSegment::Close => {
                segments.push(Segment::Close);
            }
        }
    }

    segments
}
