use crate::{inherited, inherited_str, parse_css_colour, FormulaError};
use codimate_core::Color;

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
