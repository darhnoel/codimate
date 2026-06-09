//! Layer 2 tests: Scene resolves a tree of Nodes into ConcreteScene.

use codimate_core::{
    circle, primitive_circle, rect, scene, tween, Color, ConcreteCircle, ConcreteGeometry,
    ConcreteNode, ConcreteRect, Node, Transformable,
};

/// Golden test: a Scene resolves each child Node at the same `t`, preserving
/// order and producing only plain concrete data.
#[test]
fn scene_resolves_children_in_order() {
    let s = scene()
        .node(
            circle()
                .x(tween(0.0, 20.0))
                .y(20.0)
                .radius(5.0)
                .fill(Color::RED),
        )
        .node(
            rect()
                .x(0.0)
                .y(0.0)
                .width(tween(50.0, 150.0))
                .height(40.0)
                .fill(Color::RED),
        );

    let resolved = s.resolve(0.5);

    assert_eq!(
        resolved.children,
        vec![
            ConcreteNode::Circle(ConcreteCircle {
                x: 10.0,
                y: 20.0,
                radius: 5.0,
                fill: Color::RED,
            }),
            ConcreteNode::Rect(ConcreteRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 40.0,
                fill: Color::RED,
            }),
        ]
    );
}

/// Circle, Rect, and Scene share the same public Node contract: `resolve(t)`.
#[test]
fn node_trait_resolves_primitives_and_scene() {
    fn resolve_node<N: Node>(node: &N, t: f32) -> N::Concrete {
        node.resolve(t)
    }

    let c = circle().x(tween(0.0, 10.0)).radius(2.0);
    let r = rect().width(tween(10.0, 20.0)).height(5.0);
    let s = scene().node(c.clone()).node(r.clone());

    assert_eq!(resolve_node(&c, 0.5).x, 5.0);
    assert_eq!(resolve_node(&r, 0.5).width, 15.0);
    assert_eq!(resolve_node(&s, 0.5).children.len(), 2);
}

#[test]
fn scene_add_is_alias_for_node() {
    let s = scene().add(circle().radius(10.0).fill(Color::RED));
    assert!(matches!(
        s.resolve(0.0).children[0],
        ConcreteNode::Circle(_)
    ));
}

#[test]
fn primitive_constructor_resolves_to_primitive_node() {
    let s = scene().add(primitive_circle(12.0).fill(Color::CYAN).x(40.0));
    match &s.resolve(0.0).children[0] {
        ConcreteNode::Primitive(p) => {
            assert_eq!(p.transform.pos.x, 40.0);
            assert_eq!(p.style.fill, Color::CYAN);
            assert_eq!(p.geometry, ConcreteGeometry::Circle { radius: 12.0 });
        }
        other => panic!("expected Primitive, got {other:?}"),
    }
}
