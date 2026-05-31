use codimate_core::{circle, scene};
use codimate_layout::{layout_scene, Viewport};

#[test]
fn layout_scene_pairs_concrete_scene_with_viewport() {
    let concrete = scene().node(circle().radius(10.0)).resolve(0.5);
    let viewport = Viewport::new(1920.0, 1080.0);

    let frame = layout_scene(concrete.clone(), viewport);

    assert_eq!(frame.viewport, viewport);
    assert_eq!(frame.scene, concrete);
}
