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

use codimate_core::{Color, PathNode, Segment, Vec2};

/// A typeset formula: positioned glyph outlines plus its bounding box, in a
/// local space whose origin is the formula's top-left corner.
pub struct Formula {
    /// One `PathNode` per glyph, already positioned relative to the origin.
    pub glyphs: Vec<PathNode>,
    pub width: f32,
    pub height: f32,
}

/// Why a [`formula`] could not be produced.
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
pub fn formula(latex: &str, fill: Color) -> Result<Formula, FormulaError> {
    let typst_src = latex_to_typst(latex)?;
    let svg = typst_compile(&typst_src)?;
    let glyphs = svg_to_paths(&svg, fill)?;
    Ok(Formula::from_glyphs(glyphs))
}

impl Formula {
    fn from_glyphs(glyphs: Vec<PathNode>) -> Self {
        let (mut min_x, mut min_y, mut max_x, mut max_y) =
            (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for g in &glyphs {
            let resolved = g.resolve(0.0);
            if let Some((xmin, ymin, xmax, ymax)) = resolved.path.bounding_box() {
                min_x = min_x.min(xmin);
                min_y = min_y.min(ymin);
                max_x = max_x.max(xmax);
                max_y = max_y.max(ymax);
            }
        }
        Formula {
            glyphs,
            width: if max_x > min_x { max_x - min_x } else { 0.0 },
            height: if max_y > min_y { max_y - min_y } else { 0.0 },
        }
    }
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
         #let textmath(body) = text(body)\n\
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
    use usvg::tiny_skia_path::PathSegment;

    let transformed = match path.data().clone().transform(path.abs_transform()) {
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
