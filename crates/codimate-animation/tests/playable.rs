//! Layer 3 tests: Playable is the shared contract for sampled compositions.

use codimate_animation::{animation, parallel, sequence, stagger, Playable};
use codimate_core::{circle, rect, scene, tween, ConcreteNode};

fn child_count(playable: &impl Playable) -> usize {
    playable.resolve(0.5).children.len()
}

/// Animation, Sequence, Parallel, and Stagger all expose the same public
/// sampling contract for future preview/export code.
#[test]
fn playable_resolves_all_layer_3_types() {
    let single = animation("single", 2.0, scene().node(circle().radius(10.0)));
    assert_eq!(single.name(), "single");
    assert_eq!(child_count(&single), 1);

    let sequence = sequence(
        "steps",
        [
            animation("first", 1.0, scene().node(circle().radius(10.0))),
            animation("second", 1.0, scene().node(rect().width(20.0))),
        ],
    );
    assert_eq!(sequence.name(), "steps");
    assert_eq!(child_count(&sequence), 1);

    let parallel = parallel(
        "together",
        [
            animation("a", 1.0, scene().node(circle().radius(10.0))),
            animation("b", 1.0, scene().node(rect().width(20.0))),
        ],
    );
    assert_eq!(parallel.name(), "together");
    assert_eq!(child_count(&parallel), 2);

    let stagger = stagger(
        "cascade",
        0.5,
        [
            animation("a", 1.0, scene().node(circle().radius(10.0))),
            animation("b", 1.0, scene().node(rect().width(20.0))),
        ],
    );
    assert_eq!(stagger.name(), "cascade");
    assert_eq!(child_count(&stagger), 2);
}

/// Elapsed-time sampling maps seconds into normalized `t` without owning a
/// clock. Values outside the animation range clamp to the nearest endpoint.
#[test]
fn playable_resolves_at_elapsed_seconds() {
    let single = animation(
        "single",
        2.0,
        scene().node(circle().x(tween(0.0, 100.0)).radius(10.0)),
    );

    match &single.resolve_at(1.0).children[0] {
        ConcreteNode::Circle(circle) => assert_eq!(circle.x, 50.0),
        other => panic!("expected circle at midpoint, got {other:?}"),
    }
    match &single.resolve_at(-1.0).children[0] {
        ConcreteNode::Circle(circle) => assert_eq!(circle.x, 0.0),
        other => panic!("expected circle at start, got {other:?}"),
    }
    match &single.resolve_at(3.0).children[0] {
        ConcreteNode::Circle(circle) => assert_eq!(circle.x, 100.0),
        other => panic!("expected circle at end, got {other:?}"),
    }
}
