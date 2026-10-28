//! `codimate-math` — typeset LaTeX math into `codimate-core` `Path` nodes.
//!
//! A [`Formula`] is a group of glyph outlines (animatable Béziers), produced by
//! shelling out to Typst once at Scene-build time. See `docs/adr/0005`.
//!
//! Pipeline:
//! ```text
//! LaTeX  --mitex-->  Typst markup  --typst(bin)-->  SVG  --usvg-->  Path nodes
//! ```
//!
//! # The One-Law boundary (load-bearing)
//! [`formula`] runs the Typst subprocess **once, at Scene-construction time**,
//! and bakes the result into static `PathNode`s. It is never called inside
//! `resolve(t)`, so `f(t) -> Scene` stays pure (Invariant 1). Treat it like
//! loading an asset at build time, not a per-frame side effect.

use std::path::PathBuf;

use codimate_core::{Color, GlyphBlock, PathNode, Segment, Vec2};

/// Why a [`formula()`] could not be produced.
#[derive(Debug)]
#[non_exhaustive]
pub enum FormulaError {
    /// `mitex` could not translate the LaTeX into Typst markup.
    Mitex(String),
    /// The `typst` binary could not be spawned — is it installed / on `PATH`?
    TypstSpawn(std::io::Error),
    /// `typst` ran but reported a compile error.
    TypstCompile(String),
    /// The emitted SVG could not be parsed into paths.
    Svg(String),
    /// An imported SVG file could not be read.
    SvgRead(String),
    /// An imported SVG draws text, which this build cannot render.
    SvgText(String),
}

/// Typeset a LaTeX **math** string (e.g. `r"\frac{Q_{enc}}{\epsilon_0}"`) into a
/// group of glyph `Path` nodes filled with `fill`.
///
/// Runs at build time only — see the crate-level One-Law note.
pub fn formula(latex: &str, fill: Color) -> Result<GlyphBlock, FormulaError> {
    let typst_src = latex_to_typst(latex)?;
    let svg = typst_compile(&typst_src)?;
    let glyphs = svg_to_paths(&svg, fill)?;
    Ok(GlyphBlock::from_glyphs(glyphs))
}

// --- pipeline stages ------------------------------------------------------
// Each stage is a clean seam.

/// Stage 1 — LaTeX -> Typst markup via the `mitex` crate.
fn latex_to_typst(latex: &str) -> Result<String, FormulaError> {
    match mitex::convert_math(latex, None) {
        Ok(typst) => Ok(typst),
        Err(e) => Err(FormulaError::Mitex(e)),
    }
}

/// Helpers `mitex` emits but does not define.
///
/// `mitex` translates some LaTeX into calls on its own Typst package rather
/// than into plain Typst — `\sqrt{x}` becomes `mitexsqrt(x)`, for instance.
/// The package would have to be fetched from Typst's registry at compile time,
/// so the four helpers it actually reaches for are defined here instead. That
/// keeps the pipeline offline, which is the same reason `typst` is a binary
/// rather than a crate (ADR 0005).
///
/// Without this, `\sqrt` fails — which is most of the formulas anyone wants.
const MITEX_PRELUDE: &str = r#"
#let textmath(body) = text(body)
#let mitexsqrt(..args) = if args.pos().len() == 1 { math.sqrt(..args) } else { math.root(..args) }
#let mitexmathbf(x) = math.bold(math.upright(x))
#let negthinspace = h(-(3/18) * 1em)
#let bmatrix(..args) = math.mat(delim: "[", ..args)
#let pmatrix(..args) = math.mat(delim: "(", ..args)
#let vmatrix(..args) = math.mat(delim: "|", ..args)
#let Bmatrix(..args) = math.mat(delim: "{", ..args)
"#;

/// Stage 2 — Typst markup -> SVG via the external `typst` binary.
///
/// Wraps the math in a minimal page, writes to a hash-keyed temp file,
/// shells out to `typst compile --format svg`, and caches the result.
fn typst_compile(typst_src: &str) -> Result<String, FormulaError> {
    use std::collections::hash_map::DefaultHasher;
    use std::fs;
    use std::hash::{Hash, Hasher};
    use std::io::Write;
    use std::process::Command;

    let doc = format!(
        "#set page(width: auto, height: auto, margin: 0pt, fill: none)\n\
         {MITEX_PRELUDE}\n\
         $ {typst_src} $\n"
    );

    let mut hasher = DefaultHasher::new();
    doc.hash(&mut hasher);
    let hash = hasher.finish();

    let cache = cache_dir();
    let svg_path = cache.join(format!("{hash:x}.svg"));

    if svg_path.exists() {
        return fs::read_to_string(&svg_path).map_err(|e| FormulaError::Svg(e.to_string()));
    }

    fs::create_dir_all(&cache).map_err(|e| FormulaError::Svg(e.to_string()))?;

    let typ_path = cache.join(format!("{hash:x}.typ"));
    {
        let mut f = fs::File::create(&typ_path).map_err(|e| FormulaError::Svg(e.to_string()))?;
        f.write_all(doc.as_bytes())
            .map_err(|e| FormulaError::Svg(e.to_string()))?;
    }

    let output = Command::new("typst")
        .args(["compile", "--format", "svg"])
        .arg(&typ_path)
        .arg(&svg_path)
        .output()
        .map_err(FormulaError::TypstSpawn)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(FormulaError::TypstCompile(stderr));
    }

    fs::read_to_string(&svg_path).map_err(|e| FormulaError::Svg(e.to_string()))
}

fn cache_dir() -> PathBuf {
    std::env::temp_dir().join("codimate-math")
}

/// One path out of an imported SVG, with the colour it was authored in.
///
/// `fill` is `None` when the file gave the path no paint this build can read —
/// a pattern, say. The caller decides what to do about it.
pub struct SvgPath {
    pub path: codimate_core::Path,
    pub fill: Option<Color>,
}

/// A named joint and its pivot, transformed into the SVG's rendered space.
#[derive(Clone, Debug, PartialEq)]
pub struct SvgJoint {
    pub id: String,
    pub pivot: Vec2,
}

/// One outline from a rigged SVG, with the joints and expression group that
/// own it. Joint order is root-to-leaf.
pub struct SvgRigPath {
    pub path: codimate_core::Path,
    pub fill: Option<Color>,
    /// Optional replacement paint when the character uses its dark palette.
    pub dark_fill: Option<Color>,
    pub joints: Vec<SvgJoint>,
    pub expression: Option<String>,
}

/// Art and metadata from an explicitly marked Codimate character SVG.
pub struct SvgRig {
    pub paths: Vec<SvgRigPath>,
    pub default_expression: Option<String>,
    pub expressions: Vec<String>,
}

#[derive(Clone)]
struct RigGroupMetadata {
    joint: Option<(String, f32, f32)>,
    expression: Option<String>,
    dark_fill: Option<Color>,
}

/// Read a rigged SVG. Only content beneath `<g data-codimate-rig="1">` is
/// character art; ordinary SVG import remains a separate, whole-art operation.
/// Joint pivots are authored in the root `viewBox` coordinate system.
pub fn import_svg_rig(svg: &str) -> Result<SvgRig, FormulaError> {
    use std::collections::{BTreeSet, HashMap};

    let doc = roxmltree::Document::parse(svg).map_err(|e| FormulaError::Svg(e.to_string()))?;
    let roots: Vec<_> = doc
        .descendants()
        .filter(|n| n.has_tag_name("g") && n.attribute("data-codimate-rig") == Some("1"))
        .collect();
    if roots.len() != 1 {
        return Err(FormulaError::Svg(
            "a character SVG needs exactly one <g data-codimate-rig=\"1\">".into(),
        ));
    }
    let rig_root = roots[0];
    let root_id = rig_root
        .attribute("id")
        .filter(|id| !id.is_empty())
        .ok_or_else(|| FormulaError::Svg("the character rig root needs an id".into()))?;

    let tree = usvg::Tree::from_str(svg, &usvg::Options::default())
        .map_err(|e| FormulaError::Svg(e.to_string()))?;

    let mut groups = HashMap::new();
    let mut expressions = BTreeSet::new();
    for node in rig_root.descendants().filter(|n| n.has_tag_name("g")) {
        let id = node.attribute("id").filter(|id| !id.is_empty());
        let pivot = node.attribute("data-pivot");
        let expression = node.attribute("data-expression").filter(|s| !s.is_empty());
        let dark_fill = node
            .attribute("data-dark-fill")
            .map(parse_hex_color)
            .transpose()?;
        if pivot.is_some() && id.is_none() {
            return Err(FormulaError::Svg(
                "every joint needs an id as well as data-pivot".into(),
            ));
        }
        let joint = match (id, pivot) {
            (Some(id), Some(value)) => {
                let values: Vec<f32> = value
                    .split(|c: char| c.is_ascii_whitespace() || c == ',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse::<f32>())
                    .collect::<std::result::Result<_, _>>()
                    .map_err(|_| {
                        FormulaError::Svg(format!("joint {id:?} has an invalid data-pivot"))
                    })?;
                if values.len() != 2 || values.iter().any(|v| !v.is_finite()) {
                    return Err(FormulaError::Svg(format!(
                        "joint {id:?} data-pivot must be two finite viewBox coordinates"
                    )));
                }
                Some((id.to_owned(), values[0], values[1]))
            }
            _ => None,
        };
        if let Some(name) = expression {
            if id.is_none() {
                return Err(FormulaError::Svg(
                    "every expression group needs an id".into(),
                ));
            }
            expressions.insert(name.to_owned());
        }
        if let Some(id) = id {
            if groups
                .insert(
                    id.to_owned(),
                    RigGroupMetadata {
                        joint,
                        expression: expression.map(str::to_owned),
                        dark_fill,
                    },
                )
                .is_some()
            {
                return Err(FormulaError::Svg(format!(
                    "character group id {id:?} is repeated"
                )));
            }
        }
    }

    let default_expression = rig_root
        .attribute("data-default-expression")
        .filter(|s| !s.is_empty())
        .map(str::to_owned);
    if let Some(default) = &default_expression {
        if !expressions.contains(default) {
            return Err(FormulaError::Svg(format!(
                "default character expression {default:?} has no matching data-expression group"
            )));
        }
    }

    if rig_root.descendants().any(|n| n.has_tag_name("text")) {
        return Err(FormulaError::Svg(
            "character SVG text must be converted to paths".into(),
        ));
    }

    let view_box_transform = tree
        .root()
        .children()
        .iter()
        .find_map(|node| match node {
            usvg::Node::Group(group) if group.id().is_empty() => Some(group.transform()),
            _ => None,
        })
        .unwrap_or_default();

    let root = find_group(tree.root(), root_id).ok_or_else(|| {
        FormulaError::Svg(format!("could not find character rig group {root_id:?}"))
    })?;
    let mut paths = Vec::new();
    collect_rig_paths(
        root,
        &groups,
        view_box_transform,
        &mut Vec::new(),
        None,
        None,
        &mut paths,
    )?;
    if paths.is_empty() {
        return Err(FormulaError::Svg(
            "the character rig has no visible paths".into(),
        ));
    }

    Ok(SvgRig {
        paths,
        default_expression,
        expressions: expressions.into_iter().collect(),
    })
}

fn find_group<'a>(group: &'a usvg::Group, id: &str) -> Option<&'a usvg::Group> {
    for node in group.children() {
        if let usvg::Node::Group(child) = node {
            if child.id() == id {
                return Some(child);
            }
            if let Some(found) = find_group(child, id) {
                return Some(found);
            }
        }
    }
    None
}

fn collect_rig_paths(
    group: &usvg::Group,
    metadata: &std::collections::HashMap<String, RigGroupMetadata>,
    view_box_transform: usvg::Transform,
    joints: &mut Vec<SvgJoint>,
    expression: Option<String>,
    dark_fill: Option<Color>,
    out: &mut Vec<SvgRigPath>,
) -> Result<(), FormulaError> {
    let mut joints = joints.clone();
    let mut expression = expression;
    let mut dark_fill = dark_fill;
    if let Some(metadata) = metadata.get(group.id()) {
        let group_expression = &metadata.expression;
        if let Some((id, x, y)) = &metadata.joint {
            let mut point = usvg::tiny_skia_path::Point::from_xy(*x, *y);
            view_box_transform.map_point(&mut point);
            joints.push(SvgJoint {
                id: id.clone(),
                pivot: Vec2::new(point.x, point.y),
            });
        }
        if let Some(name) = group_expression {
            if expression.as_ref().is_some_and(|parent| parent != name) {
                return Err(FormulaError::Svg(
                    "expression groups cannot be nested inside different expressions".into(),
                ));
            }
            expression = Some(name.clone());
        }
        if metadata.dark_fill.is_some() {
            dark_fill = metadata.dark_fill;
        }
    }

    for node in group.children() {
        match node {
            usvg::Node::Path(path) if path.is_visible() => {
                let segments = extract_segments(path);
                if !segments.is_empty() {
                    out.push(SvgRigPath {
                        path: codimate_core::Path {
                            segments,
                            closed: false,
                        },
                        fill: authored_colour(path),
                        dark_fill,
                        joints: joints.clone(),
                        expression: expression.clone(),
                    });
                }
            }
            usvg::Node::Group(child) => collect_rig_paths(
                child,
                metadata,
                view_box_transform,
                &mut joints,
                expression.clone(),
                dark_fill,
                out,
            )?,
            _ => {}
        }
    }
    Ok(())
}

/// Theme paints use a deliberately small, deterministic SVG extension:
/// six-digit hexadecimal RGB colors in `data-dark-fill`.
fn parse_hex_color(value: &str) -> Result<Color, FormulaError> {
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
/// [`import_svg_text`].
pub fn import_svg(svg: &str) -> Result<Vec<SvgPath>, FormulaError> {
    let opts = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &opts).map_err(|e| FormulaError::Svg(e.to_string()))?;
    let mut out = Vec::new();
    collect_svg(tree.root(), &mut out);
    Ok(out)
}

/// A line of text from an imported SVG, in the file's own coordinates.
///
/// Not shaped here. It becomes a `Geometry::Text` primitive and is shaped at
/// draw time by the same call that draws every `scene.text` — one text stack
/// rather than two (ADR 0014).
pub struct SvgText {
    pub content: String,
    /// Where the anchor sits, after `text-anchor` has been applied.
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub fill: Option<Color>,
    /// `start`, `middle` or `end`, as written.
    pub anchor: String,
}

/// Read an imported SVG's `<text>` elements.
///
/// A second pass over the source rather than a walk of the parsed tree,
/// because `usvg` is built without its `text` feature and has already thrown
/// these away by the time it hands back a tree. Enabling that feature was the
/// alternative and was rejected: it brings a second font database and a second
/// shaper, so the same string in the same font would be shaped one way from
/// `scene.text` and another from an import, inside one frame.
///
/// Deliberately narrow. A `<tspan>`, a `<textPath>`, or text under a
/// transformed group is refused rather than guessed at — this reads the common
/// case, which is a label placed at an absolute point.
pub fn import_svg_text(svg: &str) -> Result<Vec<SvgText>, FormulaError> {
    let doc = roxmltree::Document::parse(svg).map_err(|e| FormulaError::Svg(e.to_string()))?;

    for node in doc.descendants() {
        if node.has_tag_name("tspan") || node.has_tag_name("textPath") {
            return Err(FormulaError::SvgText(
                "this SVG lays out text in a way Codimate cannot read \
                 (tspan or textPath) — export it with text converted to outlines"
                    .into(),
            ));
        }
    }

    let mut out = Vec::new();
    for node in doc.descendants().filter(|n| n.has_tag_name("text")) {
        // Text nodes only. `descendants()` yields the element as well, and
        // `Node::text()` on an element returns its first text child — so
        // collecting from everything counts the content twice.
        let content: String = node
            .descendants()
            .filter(|n| n.is_text())
            .filter_map(|n| n.text())
            .collect();
        if content.trim().is_empty() {
            continue;
        }

        // A transform on the label, or on anything above it, is nearly always
        // a rotation — and the renderer cannot turn glyphs at all, the same
        // limitation `.turn()` has on `scene.text`. So this is not a shortcut:
        // there is no way to draw the label correctly, and drawing it upright
        // in the wrong place would be a silently wrong picture. Named, so the
        // author knows which label to outline.
        if node.ancestors().any(|a| a.attribute("transform").is_some()) {
            return Err(FormulaError::SvgText(format!(
                "this SVG turns the label {:?}, and Codimate cannot draw \
                 turned text — export it with text converted to outlines",
                content.trim()
            )));
        }

        out.push(SvgText {
            content: content.trim().to_string(),
            x: inherited(&node, "x").unwrap_or(0.0),
            y: inherited(&node, "y").unwrap_or(0.0),
            size: inherited(&node, "font-size").unwrap_or(16.0),
            fill: inherited_str(&node, "fill")
                .as_deref()
                .and_then(parse_css_colour),
            anchor: inherited_str(&node, "text-anchor").unwrap_or_else(|| "start".into()),
        });
    }
    Ok(out)
}

/// An attribute on this element or the nearest ancestor that sets it — SVG
/// presentation attributes inherit, and `font-size` is usually on the root.
fn inherited_str(node: &roxmltree::Node, name: &str) -> Option<String> {
    node.ancestors()
        .find_map(|n| n.attribute(name))
        .map(|v| v.trim().to_string())
}

fn inherited(node: &roxmltree::Node, name: &str) -> Option<f32> {
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
fn parse_css_colour(raw: &str) -> Option<Color> {
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
fn authored_colour(path: &usvg::Path) -> Option<Color> {
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

fn svg_to_paths(svg: &str, fill: Color) -> Result<Vec<PathNode>, FormulaError> {
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

fn extract_segments(path: &usvg::Path) -> Vec<Segment> {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// `mitex` compiles some LaTeX into calls on its own Typst package, so a
    /// preamble missing those helpers fails on ordinary formulas — `\sqrt`
    /// most of all. See `MITEX_PRELUDE`.
    ///
    /// Skipped when `typst` is not installed (ADR 0005: it is an external
    /// binary, not a build dependency).
    #[test]
    fn mitex_helpers_are_defined() {
        if std::process::Command::new("typst")
            .arg("--version")
            .output()
            .is_err()
        {
            eprintln!("skipping: typst not installed");
            return;
        }
        for latex in [
            r"\sqrt{x}",
            r"\sqrt[3]{x}",
            r"\frac{QK^{T}}{\sqrt{d_k}}",
            r"\mathbf{W}",
            r"a \! b",
            r"\begin{bmatrix} a & b \\ c & d \end{bmatrix}",
        ] {
            let block =
                formula(latex, Color::WHITE).unwrap_or_else(|e| panic!("{latex} failed: {e:?}"));
            assert!(!block.glyphs.is_empty(), "{latex} produced no glyphs");
        }
    }

    /// A fraction bar is a *stroked* line in Typst's SVG. Filling it directly
    /// draws nothing, so `\frac` used to render with no bar at all. The bar
    /// must enclose area — that is what makes it visible.
    #[test]
    fn a_fraction_has_a_visible_bar() {
        if std::process::Command::new("typst")
            .arg("--version")
            .output()
            .is_err()
        {
            eprintln!("skipping: typst not installed");
            return;
        }
        let block = formula(r"\frac{a}{b}", Color::WHITE).unwrap();
        let bar = block
            .glyphs
            .iter()
            .filter_map(|g| g.resolve(0.0).path.bounding_box())
            // The bar is the wide, flat one; `a` and `b` are roughly square.
            .find(|(x0, y0, x1, y1)| (x1 - x0) > 3.0 * (y1 - y0).max(0.001));
        let (_, y0, _, y1) = bar.expect("no fraction bar in the output at all");
        assert!(
            y1 - y0 > 0.0,
            "the fraction bar is a zero-height line, so it fills to nothing"
        );
    }
}

#[cfg(test)]
mod svg_import_tests {
    use super::*;

    // `r##` because the artwork contains `"#` in every hex colour, which
    // would close an `r#` string early.
    const ART: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
        <rect x="0" y="0" width="40" height="40" fill="#ff0000"/>
        <path d="M60 10 L90 10" stroke="#00ff00" stroke-width="4" fill="none"/>
    </svg>"##;

    /// The whole point of importing rather than pasting a picture: each path
    /// arrives with the colour it was drawn in (ADR 0014).
    #[test]
    fn each_path_keeps_the_colour_it_was_authored_in() {
        let paths = import_svg(ART).expect("should import");
        assert_eq!(paths.len(), 2, "a rect and a line");

        let red = paths[0].fill.expect("the rect has a fill");
        assert!((red.r - 1.0).abs() < 0.01 && red.g < 0.01, "{red:?}");

        // A stroke-only path reports its stroke: `extract_segments` turned the
        // stroke into a fillable outline, so the outline should be the colour
        // the stroke was, not the fill it never had.
        let green = paths[1].fill.expect("the line reports its stroke");
        assert!(green.g > 0.99 && green.r < 0.01, "{green:?}");
    }

    /// Labels come back as text to shape later, not as outlines — that is
    /// what keeps one text stack rather than two (ADR 0014).
    #[test]
    fn labels_are_read_with_their_placement() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" font-size="12">
            <text x="44" y="340" text-anchor="end" fill="#888">0.0</text>
        </svg>"##;
        let labels = import_svg_text(svg).expect("should read");
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].content, "0.0");
        assert_eq!((labels[0].x, labels[0].y), (44.0, 340.0));
        assert_eq!(labels[0].anchor, "end");
        // `font-size` is inherited from the root, which is where files put it.
        assert_eq!(labels[0].size, 12.0);
        assert!(labels[0].fill.is_some(), "#888 should parse");
    }

    /// Anything whose placement this does not implement is refused rather
    /// than drawn in the wrong spot.
    #[test]
    fn layout_we_cannot_read_is_refused() {
        for svg in [
            r#"<svg xmlns="http://www.w3.org/2000/svg"><text><tspan>a</tspan></text></svg>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><g transform="rotate(90)"><text x="1" y="2">a</text></g></svg>"#,
        ] {
            assert!(
                matches!(import_svg_text(svg), Err(FormulaError::SvgText(_))),
                "{svg}"
            );
        }
    }

    #[test]
    fn a_rig_keeps_joint_ancestry_and_viewbox_pivots() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100" viewBox="0 0 100 100">
          <g id="guide" data-codimate-rig="1" data-default-expression="smile">
            <g id="body" data-pivot="50 50">
              <path d="M40 40h20v20H40z" fill="#ff0000"/>
              <g id="arm" data-pivot="75,50" data-dark-fill="#44aaee">
                <path d="M70 45h20v10H70z" fill="#0000ff"/>
              </g>
              <g id="smile-face" data-expression="smile">
                <path d="M45 70h10" stroke="#000000" stroke-width="2"/>
              </g>
            </g>
          </g>
        </svg>"##;

        let rig = import_svg_rig(svg).expect("should import a character rig");
        assert_eq!(rig.default_expression.as_deref(), Some("smile"));
        assert_eq!(rig.expressions, ["smile"]);
        assert_eq!(rig.paths.len(), 3);

        let arm = rig
            .paths
            .iter()
            .find(|path| path.fill.is_some_and(|c| c.b > 0.9))
            .expect("blue arm outline");
        let dark_arm = arm.dark_fill.expect("dark-theme arm paint");
        assert!(dark_arm.r < 0.3 && dark_arm.g > 0.6 && dark_arm.b > 0.8);
        assert_eq!(
            arm.joints
                .iter()
                .map(|joint| joint.id.as_str())
                .collect::<Vec<_>>(),
            ["body", "arm"]
        );
        assert_eq!(
            (arm.joints[0].pivot.x, arm.joints[0].pivot.y),
            (100.0, 50.0)
        );
        assert_eq!(
            (arm.joints[1].pivot.x, arm.joints[1].pivot.y),
            (125.0, 50.0)
        );

        let face = rig
            .paths
            .iter()
            .find(|path| path.expression.as_deref() == Some("smile"))
            .expect("expression outline");
        assert_eq!(
            face.joints
                .iter()
                .map(|joint| joint.id.as_str())
                .collect::<Vec<_>>(),
            ["body"]
        );
    }

    #[test]
    fn a_rig_rejects_invalid_dark_palette_colors() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10">
          <g id="guide" data-codimate-rig="1">
            <g id="hand" data-pivot="5 5" data-dark-fill="blue">
              <path d="M1 1h8v8H1z" fill="#102A50"/>
            </g>
          </g>
        </svg>"##;
        assert!(matches!(
            import_svg_rig(svg),
            Err(FormulaError::Svg(message)) if message.contains("data-dark-fill")
        ));
    }
}
