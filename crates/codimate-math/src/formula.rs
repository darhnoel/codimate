use crate::{svg_to_paths, FormulaError};
use codimate_core::{Color, GlyphBlock};
use std::path::PathBuf;

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
