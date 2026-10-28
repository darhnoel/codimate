//! `codimate-math` — typeset LaTeX math into `codimate-core` `Path` nodes.
//!
//! A formula is a group of glyph outlines (animatable Béziers), produced by
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

mod error;
mod formula;
mod rig;
mod svg;
mod svg_text;

pub use error::*;
pub use formula::*;
pub use rig::*;
pub use svg::*;
pub use svg_text::*;

#[cfg(test)]
mod tests {
    use super::*;
    use codimate_core::Color;

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
