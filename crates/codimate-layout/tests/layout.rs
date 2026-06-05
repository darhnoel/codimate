use codimate_core::{
    circle, scene, tween, AnchorKind, Color, ConcreteNode, Style, TextAlign, Vec2,
};
use codimate_layout::{box_at, box_in, centered_text, column, layout_scene, row, Viewport};

#[test]
fn layout_scene_pairs_concrete_scene_with_viewport() {
    let concrete = scene().node(circle().radius(10.0)).resolve(0.5);
    let viewport = Viewport::new(1920.0, 1080.0);

    let frame = layout_scene(concrete.clone(), viewport);

    assert_eq!(frame.viewport, viewport);
    assert_eq!(frame.scene, concrete);
}

#[test]
fn row_computes_slot_positions_from_origin_size_gap_and_count() {
    let slots = row()
        .origin(Vec2::new(120.0, 80.0))
        .cell_size(Vec2::new(100.0, 40.0))
        .gap(12.0)
        .count(3);

    assert_eq!(slots.len(), 3);
    assert_eq!(slots[0].top_left().resolve(0.0), Vec2::new(120.0, 80.0));
    assert_eq!(slots[1].top_left().resolve(0.0), Vec2::new(232.0, 80.0));
    assert_eq!(slots[2].top_left().resolve(0.0), Vec2::new(344.0, 80.0));
}

#[test]
fn column_computes_slot_positions_from_origin_size_gap_and_count() {
    let slots = column()
        .origin(Vec2::new(120.0, 80.0))
        .cell_size(Vec2::new(100.0, 40.0))
        .gap(12.0)
        .count(3);

    assert_eq!(slots[0].top_left().resolve(0.0), Vec2::new(120.0, 80.0));
    assert_eq!(slots[1].top_left().resolve(0.0), Vec2::new(120.0, 132.0));
    assert_eq!(slots[2].top_left().resolve(0.0), Vec2::new(120.0, 184.0));
}

#[test]
fn slot_anchor_helpers_are_derived_from_slot_bounds() {
    let slots = row()
        .origin(Vec2::new(120.0, 80.0))
        .cell_size(Vec2::new(100.0, 40.0))
        .gap(12.0)
        .count(1);

    assert_eq!(slots[0].center().resolve(0.0), Vec2::new(170.0, 100.0));
    assert_eq!(slots[0].top().resolve(0.0), Vec2::new(170.0, 80.0));
    assert_eq!(slots[0].bottom().resolve(0.0), Vec2::new(170.0, 120.0));
    assert_eq!(slots[0].left().resolve(0.0), Vec2::new(120.0, 100.0));
    assert_eq!(slots[0].right().resolve(0.0), Vec2::new(220.0, 100.0));
    assert_eq!(
        slots[0].anchor(AnchorKind::Right).resolve(0.0),
        Vec2::new(220.0, 100.0)
    );
}

#[test]
fn viewport_slot_represents_the_whole_screen() {
    let slot = Viewport::new(960.0, 540.0).slot();

    assert_eq!(slot.top_left().resolve(0.0), Vec2::new(0.0, 0.0));
    assert_eq!(slot.center().resolve(0.0), Vec2::new(480.0, 270.0));
    assert_eq!(slot.right().resolve(0.0), Vec2::new(960.0, 270.0));
    assert_eq!(slot.bottom().resolve(0.0), Vec2::new(480.0, 540.0));
}

#[test]
fn centered_child_derives_a_slot_centered_inside_parent_bounds() {
    let parent = Viewport::new(960.0, 540.0).slot();
    let child = parent.centered_child(Vec2::new(302.0, 86.0));

    assert_eq!(child.top_left().resolve(0.0), Vec2::new(329.0, 227.0));
    assert_eq!(child.center().resolve(0.0), Vec2::new(480.0, 270.0));
}

#[test]
fn slot_row_and_column_lay_out_children_inside_parent_slot() {
    let parent = row()
        .origin(Vec2::new(120.0, 80.0))
        .cell_size(Vec2::new(300.0, 100.0))
        .count(1)
        .remove(0);

    let row_children = parent.row(Vec2::new(60.0, 20.0), 12.0, 2);
    let column_children = parent.column(Vec2::new(60.0, 20.0), 12.0, 2);

    assert_eq!(
        row_children[0].top_left().resolve(0.0),
        Vec2::new(120.0, 80.0)
    );
    assert_eq!(
        row_children[1].top_left().resolve(0.0),
        Vec2::new(192.0, 80.0)
    );
    assert_eq!(
        column_children[0].top_left().resolve(0.0),
        Vec2::new(120.0, 80.0)
    );
    assert_eq!(
        column_children[1].top_left().resolve(0.0),
        Vec2::new(120.0, 112.0)
    );
}

#[test]
fn centered_text_places_text_at_slot_center_with_center_alignment() {
    let slot = Viewport::new(960.0, 540.0)
        .slot()
        .centered_child(Vec2::new(300.0, 100.0));

    let label = centered_text(&slot, "hello", 20.0, Color::WHITE).resolve(0.0);

    assert_eq!(label.x, 480.0);
    assert!((label.y - 276.4).abs() < 0.001);
    assert_eq!(label.text, "hello");
    assert_eq!(label.font_size, 20.0);
    assert_eq!(label.fill, Color::WHITE);
    assert_eq!(label.align, TextAlign::Center);
}

#[test]
fn box_in_uses_slot_bounds_and_style() {
    let slot = row()
        .origin(Vec2::new(120.0, 80.0))
        .cell_size(Vec2::new(100.0, 40.0))
        .count(1)
        .remove(0);
    let node = box_in(&slot)
        .radius(8.0)
        .style(Style::new().fill(Color::RED).stroke(2.0, Color::CYAN))
        .into_node()
        .resolve(0.0);

    assert_eq!(node.path.bounding_box(), Some((120.0, 80.0, 220.0, 120.0)));
    assert_eq!(node.fill, Color::RED);
    assert_eq!(node.stroke_width, 2.0);
    assert_eq!(node.stroke_color, Color::CYAN);
}

#[test]
fn box_at_uses_center_position_and_animated_radius() {
    let node = box_at(
        tween(Vec2::new(100.0, 80.0), Vec2::new(200.0, 120.0)),
        Vec2::new(60.0, 40.0),
    )
    .radius(tween(0.0, 8.0))
    .into_node();

    let start = node.resolve(0.0);
    let mid = node.resolve(0.5);

    assert_eq!(start.path.bounding_box(), Some((70.0, 60.0, 130.0, 100.0)));
    assert_eq!(mid.path.bounding_box(), Some((120.0, 80.0, 180.0, 120.0)));
    assert_eq!(start.path.segments.len(), 8);
    assert_eq!(mid.path.segments.len(), 8);
}

#[test]
fn box_style_obeys_builder_order_overrides() {
    let slot = Viewport::new(100.0, 50.0).slot();
    let resolved = box_in(&slot)
        .style(Style::new().fill(Color::RED).stroke(1.0, Color::CYAN))
        .fill(Color::WHITE)
        .stroke(3.0, Color::BLACK)
        .into_node()
        .resolve(0.0);

    assert_eq!(resolved.fill, Color::WHITE);
    assert_eq!(resolved.stroke_width, 3.0);
    assert_eq!(resolved.stroke_color, Color::BLACK);
}

#[test]
fn box_can_be_added_to_scene_without_becoming_a_new_node_variant() {
    let slot = Viewport::new(100.0, 50.0).slot();
    let concrete = scene().node(box_in(&slot).fill(Color::RED)).resolve(0.0);

    assert!(matches!(concrete.children[0], ConcreteNode::Path(_)));
}

#[test]
fn slot_direction_helpers_derive_centered_neighbor_slots_from_parent_bounds() {
    let slots = row()
        .origin(Vec2::new(120.0, 80.0))
        .cell_size(Vec2::new(100.0, 40.0))
        .count(1);

    let above = slots[0].above(Vec2::new(60.0, 20.0), 12.0);
    let below = slots[0].below(Vec2::new(60.0, 20.0), 12.0);
    let left = slots[0].left_of(Vec2::new(60.0, 20.0), 12.0);
    let right = slots[0].right_of(Vec2::new(60.0, 20.0), 12.0);

    assert_eq!(above.top_left().resolve(0.0), Vec2::new(140.0, 48.0));
    assert_eq!(below.top_left().resolve(0.0), Vec2::new(140.0, 132.0));
    assert_eq!(left.top_left().resolve(0.0), Vec2::new(48.0, 90.0));
    assert_eq!(right.top_left().resolve(0.0), Vec2::new(232.0, 90.0));

    assert_eq!(above.center().resolve(0.0), Vec2::new(170.0, 58.0));
    assert_eq!(below.center().resolve(0.0), Vec2::new(170.0, 142.0));
    assert_eq!(left.center().resolve(0.0), Vec2::new(78.0, 100.0));
    assert_eq!(right.center().resolve(0.0), Vec2::new(262.0, 100.0));
}

#[test]
fn animated_origin_composes_with_local_slot_positions() {
    let slots = row()
        .origin(tween(Vec2::new(100.0, 80.0), Vec2::new(200.0, 120.0)))
        .cell_size(Vec2::new(100.0, 40.0))
        .gap(12.0)
        .count(2);

    assert_eq!(slots[0].top_left().resolve(0.0), Vec2::new(100.0, 80.0));
    assert_eq!(slots[1].top_left().resolve(0.5), Vec2::new(262.0, 100.0));
    assert_eq!(slots[0].center().resolve(1.0), Vec2::new(250.0, 140.0));
}

#[test]
fn derived_slots_follow_animated_parent_origin() {
    let slots = row()
        .origin(tween(Vec2::new(100.0, 80.0), Vec2::new(200.0, 120.0)))
        .cell_size(Vec2::new(100.0, 40.0))
        .count(1);

    let label_slot = slots[0].below(Vec2::new(60.0, 20.0), 12.0);
    let side_slot = slots[0].right_of(Vec2::new(60.0, 20.0), 12.0);

    assert_eq!(label_slot.top_left().resolve(0.5), Vec2::new(170.0, 152.0));
    assert_eq!(side_slot.top_left().resolve(0.5), Vec2::new(262.0, 110.0));
}

#[test]
fn centered_child_follows_animated_parent_origin() {
    let parent = row()
        .origin(tween(Vec2::new(100.0, 80.0), Vec2::new(200.0, 120.0)))
        .cell_size(Vec2::new(300.0, 100.0))
        .count(1)
        .remove(0);

    let child = parent.centered_child(Vec2::new(100.0, 40.0));

    assert_eq!(child.top_left().resolve(0.5), Vec2::new(250.0, 130.0));
    assert_eq!(child.center().resolve(1.0), Vec2::new(350.0, 170.0));
}

#[test]
fn layout_does_not_clamp_offscreen_slots() {
    let slots = row()
        .origin(Vec2::new(1200.0, 100.0))
        .cell_size(Vec2::new(100.0, 40.0))
        .gap(20.0)
        .count(2);

    assert_eq!(slots[0].right().resolve(0.0), Vec2::new(1300.0, 120.0));
    assert_eq!(slots[1].top_left().resolve(0.0), Vec2::new(1320.0, 100.0));
}
