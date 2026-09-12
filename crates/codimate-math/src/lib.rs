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
pub enum FormulaError {
    /// `mitex` could not translate the LaTeX into Typst markup.
    Mitex(String),
    /// The `typst` binary could not be spawned — is it installed / on `PATH`?
    TypstSpawn(std::io::Error),
    /// `typst` ran but reported a compile error.
    TypstCompile(String),
    /// The emitted SVG could not be parsed into paths.
    Svg(String),
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

/// Stage 3 — SVG -> core `PathNode`s via the `usvg` crate.
///
/// Walks the usvg tree, extracts path geometry from every visible Path node,
/// applies the node's absolute transform, and produces one `PathNode` per
/// glyph with multi-contour support (MoveTo / Close segments).
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
        if std::process::Command::new("typst").arg("--version").output().is_err() {
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
            let block = formula(latex, Color::WHITE)
                .unwrap_or_else(|e| panic!("{latex} failed: {e:?}"));
            assert!(!block.glyphs.is_empty(), "{latex} produced no glyphs");
        }
    }

    /// A fraction bar is a *stroked* line in Typst's SVG. Filling it directly
    /// draws nothing, so `\frac` used to render with no bar at all. The bar
    /// must enclose area — that is what makes it visible.
    #[test]
    fn a_fraction_has_a_visible_bar() {
        if std::process::Command::new("typst").arg("--version").output().is_err() {
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
        assert!(y1 - y0 > 0.0, "the fraction bar is a zero-height line, so it fills to nothing");
    }
}

