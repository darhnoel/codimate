use crate::{
    style::*, HeldKey, InsertionAction, InsertionMovement, InsertionSort, InsertionSortMotion,
    InsertionSortTiming, InsertionStep, InsertionTrace, VisualItem, DEFAULT_VALUES, N,
};
use codimate::*;
use codimate::{animation, sequence, text, Animation, Playable, Scene, Text, TextAlign, Viewport};
use std::f32::consts::PI;

const VIEW_W: f32 = 960.0;
const VIEW_H: f32 = 540.0;
const BASELINE: f32 = 430.0;
const BAR_W: f32 = 78.0;
const GAP: f32 = 20.0;
const GROUP_W: f32 = N as f32 * BAR_W + (N - 1) as f32 * GAP;
const LEFT: f32 = (VIEW_W - GROUP_W) * 0.5;
const HEIGHT_SCALE: f32 = 30.0;
const HELD_X: f32 = 24.0;
const HELD_Y: f32 = 178.0;
const HELD_W: f32 = 110.0;
const HELD_H: f32 = 264.0;
const HELD_GAP: f32 = 34.0;
const HELD_LAYOUT_LEFT: f32 = HELD_X + HELD_W + HELD_GAP;
const HELD_LABEL_Y: f32 = 150.0;

#[derive(Clone, Copy)]
pub struct InsertionSortView;

pub fn insertion_sort_view() -> InsertionSortView {
    InsertionSortView
}

pub(crate) fn build_insertion_sort(
    name: &'static str,
    _state: InsertionSort,
    trace: InsertionTrace,
    _motion: InsertionSortMotion,
    timing: InsertionSortTiming,
) -> (Box<dyn Playable>, Viewport) {
    let mut animations: Vec<Animation> = trace
        .steps
        .iter()
        .enumerate()
        .map(|(idx, step)| {
            let duration = timing.duration_for(step.action);
            animation(
                format!("insertion-step-{idx:03}"),
                duration,
                step_scene(*step),
            )
        })
        .collect();

    animations.push(animation(
        "sorted",
        timing.final_hold,
        sorted_scene(trace.sorted),
    ));

    (
        Box::new(sequence(name, animations)),
        Viewport::new(VIEW_W, VIEW_H),
    )
}

fn step_scene(step: InsertionStep) -> Scene {
    let mut sc = base_scene(step.title());
    let left = layout_left(step);
    sc = draw_slots(sc, step, left);

    if step.held.is_some() {
        sc = draw_held_area(sc);
    }

    sc = match (step.action, step.movement) {
        (InsertionAction::ChooseKey { index, .. }, None) => draw_choose_key(sc, step, index, left),
        (_, Some(InsertionMovement::ShiftRight { item, from, to })) => {
            sc = draw_static_held_key(sc, step.held, false);
            draw_shift_movement(sc, item, from, to, left)
        }
        (_, Some(InsertionMovement::InsertKey { item, from, to })) => {
            draw_insert_movement(sc, item, from, to, left)
        }
        (_, None) => draw_static_held_key(sc, step.held, step.compare_left.is_some()),
    };

    if let Some(left) = step.compare_left {
        if let Some(item) = step.slots[left] {
            sc = sc.add(compare_border_for_slot(left, item.value, layout_left(step)));
        }
    }

    sc
}

fn draw_slots(mut sc: Scene, step: InsertionStep, left: f32) -> Scene {
    for slot in 0..N {
        if should_skip_slot(step, slot) {
            continue;
        }

        if let Some(item) = step.slots[slot] {
            sc = sc.add(item_at_slot(slot, item, left));
            sc = sc.add(label_at_slot(slot, item.value, INK, left));
        }
    }
    sc
}

fn should_skip_slot(step: InsertionStep, slot: usize) -> bool {
    match step.movement {
        Some(InsertionMovement::ShiftRight { from, .. }) => slot == from,
        Some(InsertionMovement::InsertKey { .. }) => false,
        None => false,
    }
}

fn draw_held_area(sc: Scene) -> Scene {
    sc.add(
        primitive_path(rect_path(HELD_X, HELD_Y, HELD_W, HELD_H))
            .style(Style::new().fill(Color::TRANSPARENT).stroke(2.0, HELD)),
    )
    .add(centered_label(
        HELD_X + HELD_W * 0.5,
        HELD_LABEL_Y,
        "តម្លៃចំណាំបណ្ដោះអាសន្ន",
        16.0,
        HELD,
    ))
}

fn draw_choose_key(mut sc: Scene, step: InsertionStep, index: usize, left: f32) -> Scene {
    if let Some(held) = step.held {
        let item = held.item;
        let x = tween(slot_x(index, left), held_bar_x()).ease(ease_in_out);
        let y = bar_top(item.value).into_animated();
        sc = sc.add(bar_node(x.clone(), y.clone(), item.value, item_color(item)));
        sc = sc.add(value_label(x, y, item.value, INK));
    }
    sc
}

fn draw_static_held_key(mut sc: Scene, held: Option<HeldKey>, compare_glow: bool) -> Scene {
    if let Some(held) = held {
        sc = sc.add(held_key_node(held.item));
        sc = sc.add(held_key_label(held.item));
        if compare_glow {
            sc = sc.add(held_key_border(held.item, ACTIVE));
        }
    }
    sc
}

fn draw_shift_movement(
    mut sc: Scene,
    item: VisualItem,
    from: usize,
    to: usize,
    left: f32,
) -> Scene {
    let x = straight_x(from, to, left);
    let y = bar_top(item.value).into_animated();
    sc = sc.add(bar_node(x.clone(), y.clone(), item.value, item_color(item)));
    sc = sc.add(moving_border(x.clone(), y.clone(), item.value));
    sc.add(value_label(x, y, item.value, INK))
}

fn draw_insert_movement(
    mut sc: Scene,
    item: VisualItem,
    _from: usize,
    to: usize,
    left: f32,
) -> Scene {
    let x = held_to_slot_x(to, left);
    let y = arc_y_to_slot(item.value);
    sc = sc.add(bar_node(x.clone(), y.clone(), item.value, item_color(item)));
    sc = sc.add(moving_border(x.clone(), y.clone(), item.value));
    sc.add(value_label(x, y, item.value, INK))
}

fn sorted_scene(values: [i32; N]) -> Scene {
    let mut sc = base_scene("តម្រៀបបានបញ្ចប់".to_string());
    for (idx, value) in values.iter().enumerate() {
        sc = sc.add(bar_at_slot(idx, *value, color_for_value(*value), LEFT));
        sc = sc.add(label_at_slot(idx, *value, INK, LEFT));
    }
    sc
}

fn base_scene(title: String) -> Scene {
    scene()
        .add(primitive_path(rect_path(0.0, 0.0, VIEW_W, VIEW_H)).style(Style::new().fill(BG)))
        .add(centered_label(
            VIEW_W * 0.5,
            58.0,
            "តម្រៀបដោយប្រើវិធីបញ្ចូល",
            34.0,
            INK,
        ))
        .add(centered_label(VIEW_W * 0.5, 96.0, title, 22.0, INK))
}

fn bar_node(x: Animated<f32>, y: Animated<f32>, value: i32, fill: Color) -> Rect {
    let h = bar_height(value);
    rect().x(x).y(y).width(BAR_W).height(h).fill(fill)
}

fn value_label(x: Animated<f32>, y: Animated<f32>, value: i32, fill: Color) -> Text {
    text()
        .x(x.map(|v| v + BAR_W * 0.5))
        .y(y.map(|v| v - 24.0))
        .text(value.to_string())
        .font_size(20.0)
        .fill(fill)
        .align(TextAlign::Center)
}

fn centered_label(x: f32, y: f32, content: impl Into<String>, font_size: f32, fill: Color) -> Text {
    text()
        .x(x)
        .y(y)
        .text(content.into())
        .font_size(font_size)
        .fill(fill)
        .align(TextAlign::Center)
}

fn layout_left(step: InsertionStep) -> f32 {
    if step.held.is_some() {
        HELD_LAYOUT_LEFT
    } else {
        LEFT
    }
}

fn slot_x(idx: usize, left: f32) -> f32 {
    left + idx as f32 * (BAR_W + GAP)
}

fn bar_height(value: i32) -> f32 {
    (value.max(1) as f32) * HEIGHT_SCALE
}

fn bar_top(value: i32) -> f32 {
    BASELINE - bar_height(value)
}

fn bar_at_slot(slot: usize, value: i32, fill: Color, left: f32) -> Rect {
    bar_node(
        slot_x(slot, left).into_animated(),
        bar_top(value).into_animated(),
        value,
        fill,
    )
}

fn item_at_slot(slot: usize, item: VisualItem, left: f32) -> Rect {
    bar_at_slot(slot, item.value, item_color(item), left)
}

fn label_at_slot(slot: usize, value: i32, fill: Color, left: f32) -> Text {
    value_label(
        slot_x(slot, left).into_animated(),
        bar_top(value).into_animated(),
        value,
        fill,
    )
}

fn straight_x(from: usize, to: usize, left: f32) -> Animated<f32> {
    if from == to {
        slot_x(from, left).into_animated()
    } else {
        tween(slot_x(from, left), slot_x(to, left)).ease(ease_in_out)
    }
}

fn held_bar_x() -> f32 {
    HELD_X + (HELD_W - BAR_W) * 0.5
}

fn held_to_slot_x(to: usize, left: f32) -> Animated<f32> {
    tween(held_bar_x(), slot_x(to, left)).ease(ease_in_out)
}

fn arc_y_to_slot(value: i32) -> Animated<f32> {
    let start = bar_top(value);
    let end = bar_top(value);
    let lift = 46.0;
    Animated::new(move |t| {
        let linear = start + (end - start) * t;
        linear - lift * (PI * t).sin()
    })
}

fn held_key_node(item: VisualItem) -> Rect {
    bar_node(
        held_bar_x().into_animated(),
        bar_top(item.value).into_animated(),
        item.value,
        item_color(item),
    )
}

fn item_color(item: VisualItem) -> Color {
    color_for_index(item.id)
}

fn color_for_value(value: i32) -> Color {
    let idx = DEFAULT_VALUES
        .iter()
        .position(|candidate| *candidate == value)
        .unwrap_or(0);
    color_for_index(idx)
}

fn color_for_index(idx: usize) -> Color {
    ITEM_COLORS[idx % ITEM_COLORS.len()]
}

fn held_key_label(item: VisualItem) -> Text {
    value_label(
        held_bar_x().into_animated(),
        bar_top(item.value).into_animated(),
        item.value,
        INK,
    )
}

fn held_key_border(item: VisualItem, color: Color) -> Primitive {
    primitive_path(rect_path(
        held_bar_x() - 4.0,
        bar_top(item.value) - 4.0,
        BAR_W + 8.0,
        bar_height(item.value) + 8.0,
    ))
    .style(Style::new().fill(Color::TRANSPARENT).stroke(3.0, color))
}

fn moving_border(x: Animated<f32>, y: Animated<f32>, value: i32) -> Primitive {
    primitive_path(Animated::new(move |t| {
        rect_path(
            x.resolve(t) - 4.0,
            y.resolve(t) - 4.0,
            BAR_W + 8.0,
            bar_height(value) + 8.0,
        )
    }))
    .style(Style::new().fill(Color::TRANSPARENT).stroke(3.0, ACTIVE))
}

fn compare_border_for_slot(slot: usize, value: i32, left: f32) -> Primitive {
    primitive_path(rect_path(
        slot_x(slot, left) - 4.0,
        bar_top(value) - 4.0,
        BAR_W + 8.0,
        bar_height(value) + 8.0,
    ))
    .style(Style::new().fill(Color::TRANSPARENT).stroke(3.0, ACTIVE))
}
