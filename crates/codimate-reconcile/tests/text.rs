//! A text's font: named by file or family, checked when the film is built.

use codimate_core::{ConcreteGeometry, ConcreteNode};
use codimate_reconcile::*;

fn label(font: &str) -> Shape {
    Shape {
        item: "t".into(),
        kind: "text".into(),
        text: "Hello".into(),
        size: 30.0,
        color: "white".into(),
        opacity: 1.0,
        scale_x: 1.0,
        scale_y: 1.0,
        font: font.into(),
        ..Default::default()
    }
}

fn khmer_file() -> String {
    format!(
        "{}/../codimate-fonts/fonts/NotoSansKhmer-Regular.ttf",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[test]
fn a_text_carries_its_font_to_the_frame() {
    let scene = build_segment(&[label(&khmer_file())], &[label(&khmer_file())], &[]).unwrap();
    let ConcreteNode::Primitive(p) = &scene.resolve(0.5).children[0] else {
        panic!("a text is a primitive");
    };
    let ConcreteGeometry::Text { font, .. } = &p.geometry else {
        panic!("a text has text geometry");
    };
    assert_eq!(font, &khmer_file());
}

#[test]
fn a_font_that_will_not_load_is_an_error_naming_the_shape() {
    let err = build_segment(
        &[label("fonts/missing.ttf")],
        &[label("fonts/missing.ttf")],
        &[],
    )
    .err()
    .expect("a missing font file is refused");
    assert!(
        err.0.contains("\"t\"") && err.0.contains("missing.ttf"),
        "{err}"
    );
}
