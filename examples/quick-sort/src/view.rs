use crate::{
    style::*, QuickAction, QuickSort, QuickSortMotion, QuickSortTiming, QuickStep, QuickTrace, N,
};
use codimate::Viewport;
use codimate::*;
use codimate::{animation, sequence, Animation, Playable};
use std::f32::consts::PI;

const VIEW_W: f32 = 1000.0;
const VIEW_H: f32 = 560.0;
const TILE_W: f32 = 58.0;
const TILE_H: f32 = 42.0;
const TILE_GAP: f32 = 18.0;
const TILE_FONT: f32 = 17.0;
const TILE_START_X: f32 = (VIEW_W - (N as f32 * TILE_W + (N - 1) as f32 * TILE_GAP)) / 2.0;
const TILE_RADIUS: f32 = 7.0;

const HEADER_Y: f32 = 42.0;
const ARRAY_Y: f32 = 260.0;
const INDEX_Y: f32 = ARRAY_Y + TILE_H + 32.0;

#[derive(Clone, Copy)]
pub struct QuickSortView;

pub fn quick_sort_view() -> QuickSortView {
    QuickSortView
}

pub(crate) fn build_quick_sort(
    name: &'static str,
    state: QuickSort,
    trace: QuickTrace,
    motion: QuickSortMotion,
    timing: QuickSortTiming,
) -> (Box<dyn Playable>, Viewport) {
    let mut anims: Vec<Animation> = Vec::new();

    anims.push(animation(
        "overview",
        timing.overview,
        overview_scene(&state.values()),
    ));
    for (i, step) in trace.steps.iter().enumerate() {
        anims.push(animation(
            format!("quick-step-{i:02}"),
            step_duration(step, timing),
            step_scene(step, motion),
        ));
    }
    anims.push(animation(
        "sorted",
        timing.final_hold,
        final_scene(&trace.sorted),
    ));

    (
        Box::new(sequence(name, anims)),
        Viewport::new(VIEW_W, VIEW_H),
    )
}

fn step_duration(step: &QuickStep, timing: QuickSortTiming) -> f32 {
    match step.action {
        QuickAction::ChoosePivot => timing.choose_pivot,
        QuickAction::Compare { .. } => timing.compare,
        QuickAction::Swap { .. } => timing.swap,
        QuickAction::PlacePivot { .. } => timing.place_pivot,
    }
}

fn tile_x(index: usize) -> f32 {
    TILE_START_X + index as f32 * (TILE_W + TILE_GAP)
}

fn text_width(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * font_size * 0.30
}

fn rounded_rect_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> Path {
    let r = r.min(w / 2.0).min(h / 2.0);
    let k = 0.552_284_8 * r;
    Path {
        segments: vec![
            Segment::Line(Vec2::new(x + r, y), Vec2::new(x + w - r, y)),
            Segment::Cubic(
                Vec2::new(x + w - r, y),
                Vec2::new(x + w - r + k, y),
                Vec2::new(x + w, y + r - k),
                Vec2::new(x + w, y + r),
            ),
            Segment::Line(Vec2::new(x + w, y + r), Vec2::new(x + w, y + h - r)),
            Segment::Cubic(
                Vec2::new(x + w, y + h - r),
                Vec2::new(x + w, y + h - r + k),
                Vec2::new(x + w - r + k, y + h),
                Vec2::new(x + w - r, y + h),
            ),
            Segment::Line(Vec2::new(x + w - r, y + h), Vec2::new(x + r, y + h)),
            Segment::Cubic(
                Vec2::new(x + r, y + h),
                Vec2::new(x + r - k, y + h),
                Vec2::new(x, y + h - r + k),
                Vec2::new(x, y + h - r),
            ),
            Segment::Line(Vec2::new(x, y + h - r), Vec2::new(x, y + r)),
            Segment::Cubic(
                Vec2::new(x, y + r),
                Vec2::new(x, y + r - k),
                Vec2::new(x + r - k, y),
                Vec2::new(x + r, y),
            ),
        ],
        closed: true,
    }
}

fn style(fill: Color, stroke_width: f32, stroke_color: Color) -> Style {
    Style::new().fill(fill).stroke(stroke_width, stroke_color)
}

fn translucent(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

fn background_style() -> Style {
    style(BG, 0.0, BG)
}

fn tile_style(fill: Color) -> Style {
    style(fill, 1.8, TILE_STROKE)
}

fn slot_style() -> Style {
    style(PANEL, 1.0, PANEL_BORDER)
}

fn range_style() -> Style {
    style(
        translucent(RANGE_FILL, 0.35),
        1.2,
        translucent(ACCENT, 0.50),
    )
}

fn label(x: f32, y: f32, content: impl Into<String>, font_size: f32, fill: Color) -> Text {
    text()
        .x(x)
        .y(y)
        .text(content.into())
        .font_size(font_size)
        .fill(fill)
}

fn centered_label(x: f32, y: f32, content: impl Into<String>, font_size: f32, fill: Color) -> Text {
    let content = content.into();
    label(
        x - text_width(&content, font_size) / 2.0,
        y,
        content,
        font_size,
        fill,
    )
}

fn box_path(x: impl IntoAnimated<f32>, y: impl IntoAnimated<f32>) -> Animated<Path> {
    let x = x.into_animated();
    let y = y.into_animated();
    Animated::new(move |t| {
        rounded_rect_path(x.resolve(t), y.resolve(t), TILE_W, TILE_H, TILE_RADIUS)
    })
}

fn arced_y(base: f32, height: f32, direction: f32) -> Animated<f32> {
    Animated::new(move |t| base + direction * height * (PI * t).sin())
}

fn add_panel(sc: Scene) -> Scene {
    sc.add(primitive_path(rect_path(0.0, 0.0, VIEW_W, VIEW_H)).style(background_style()))
}

fn add_header(mut sc: Scene, subtitle: impl Into<String>) -> Scene {
    sc = sc.add(label(54.0, HEADER_Y, "Quick Sort", 28.0, INK));
    sc.add(label(54.0, 76.0, subtitle, 16.0, MUTED))
}

fn add_slots(mut sc: Scene) -> Scene {
    for i in 0..N {
        sc = sc.add(
            primitive_path(rounded_rect_path(
                tile_x(i),
                ARRAY_Y,
                TILE_W,
                TILE_H,
                TILE_RADIUS,
            ))
            .style(slot_style()),
        );
        sc = sc.add(centered_label(
            tile_x(i) + TILE_W / 2.0,
            INDEX_Y,
            i.to_string(),
            12.0,
            translucent(MUTED, 0.72),
        ));
    }
    sc
}

fn add_range_band(sc: Scene, low: usize, high: usize, depth: usize) -> Scene {
    if low > high {
        return sc;
    }
    let pad_x = 6.0;
    let pad_y = 9.0;
    let depth_offset = depth as f32 * 7.0;
    let x = tile_x(low) - pad_x;
    let y = ARRAY_Y - pad_y - depth_offset;
    let w = tile_x(high) + TILE_W - x + pad_x;
    let h = TILE_H + pad_y * 2.0 + depth_offset;
    sc.add(primitive_path(rounded_rect_path(x, y, w, h, TILE_RADIUS + 2.0)).style(range_style()))
}

fn tile_fill(step: Option<&QuickStep>, index: usize, final_state: bool) -> Color {
    if final_state {
        return SORTED_FILL;
    }
    let Some(step) = step else {
        return PANEL;
    };
    match step.action {
        QuickAction::Compare { index: candidate } if index == candidate => CANDIDATE_FILL,
        QuickAction::PlacePivot { to, .. } if index == to => PIVOT_FILL,
        _ if index == step.pivot_index => PIVOT_FILL,
        _ if index >= step.low && index < step.store_index => SMALL_FILL,
        _ if index >= step.low && index <= step.high => PANEL,
        _ => translucent(PANEL, 0.55),
    }
}

fn add_tile(
    sc: Scene,
    value: i32,
    x: impl IntoAnimated<f32>,
    y: impl IntoAnimated<f32>,
    tile_style: impl IntoAnimated<Style>,
) -> Scene {
    add_tile_with_text(sc, value, x, y, tile_style, INK)
}

fn add_tile_with_text(
    mut sc: Scene,
    value: i32,
    x: impl IntoAnimated<f32>,
    y: impl IntoAnimated<f32>,
    tile_style: impl IntoAnimated<Style>,
    text_fill: Color,
) -> Scene {
    let x = x.into_animated();
    let y = y.into_animated();
    let text_x = x.clone();
    let text_y = y.clone();
    let content = value.to_string();
    let offset_x = (TILE_W - text_width(&content, TILE_FONT)) / 2.0;
    let offset_y = (TILE_H + TILE_FONT * 0.45) / 2.0;

    sc = sc.add(primitive_path(box_path(x, y)).style(tile_style));
    sc.add(
        text()
            .x(Animated::new(move |t| text_x.resolve(t) + offset_x))
            .y(Animated::new(move |t| text_y.resolve(t) + offset_y))
            .text(content)
            .font_size(TILE_FONT)
            .fill(text_fill),
    )
}

fn add_array(
    mut sc: Scene,
    values: &[i32; N],
    step: Option<&QuickStep>,
    hidden: &[usize],
    final_state: bool,
) -> Scene {
    for (i, value) in values.iter().enumerate() {
        if hidden.contains(&i) {
            continue;
        }
        sc = add_tile(
            sc,
            *value,
            tile_x(i),
            ARRAY_Y,
            tile_style(tile_fill(step, i, final_state)),
        );
    }
    sc
}

fn add_legend(mut sc: Scene) -> Scene {
    let y = 408.0;
    for (i, (name, fill)) in [
        ("pivot", PIVOT_FILL),
        ("candidate", CANDIDATE_FILL),
        ("< pivot", SMALL_FILL),
    ]
    .into_iter()
    .enumerate()
    {
        let x = 346.0 + i as f32 * 120.0;
        sc = sc
            .add(primitive_path(rounded_rect_path(x, y, 18.0, 18.0, 5.0)).style(tile_style(fill)));
        sc = sc.add(label(x + 28.0, y + 14.0, name, 13.0, MUTED));
    }
    sc
}

fn overview_scene(values: &[i32; N]) -> Scene {
    let mut sc = add_panel(scene());
    sc = add_header(sc, "Partition around a pivot, then recurse on both sides");
    sc = add_slots(sc);
    sc = add_array(sc, values, None, &[], false);
    add_legend(sc)
}

fn step_scene(step: &QuickStep, motion: QuickSortMotion) -> Scene {
    match step.action {
        QuickAction::Swap { left, right } => swap_scene(step, motion, left, right),
        QuickAction::PlacePivot { from, to } if from != to => swap_scene(step, motion, from, to),
        _ => static_step_scene(step),
    }
}

fn static_step_scene(step: &QuickStep) -> Scene {
    let mut sc = add_panel(scene());
    sc = add_header(sc, step.title());
    sc = add_range_band(sc, step.low, step.high, step.depth);
    sc = add_slots(sc);
    sc = add_array(sc, &step.values_before, Some(step), &[], false);
    add_step_markers(sc, step)
}

fn swap_scene(step: &QuickStep, motion: QuickSortMotion, left: usize, right: usize) -> Scene {
    let mut sc = add_panel(scene());
    sc = add_header(sc, step.title());
    sc = add_range_band(sc, step.low, step.high, step.depth);
    sc = add_slots(sc);
    sc = add_array(sc, &step.values_before, Some(step), &[left, right], false);
    sc = add_step_markers(sc, step);

    let left_value = step.values_before[left];
    let right_value = step.values_before[right];
    let moving_style = tile_style(MOVING_FILL);
    sc = add_tile(
        sc,
        left_value,
        motion.move_value(tile_x(left), tile_x(right)),
        arced_y(ARRAY_Y, motion.lift_height, -1.0),
        moving_style,
    );
    add_tile(
        sc,
        right_value,
        motion.move_value(tile_x(right), tile_x(left)),
        arced_y(ARRAY_Y, motion.lift_height, 1.0),
        moving_style,
    )
}

fn add_step_markers(mut sc: Scene, step: &QuickStep) -> Scene {
    sc = sc.add(centered_label(
        tile_x(step.pivot_index) + TILE_W / 2.0,
        ARRAY_Y - 24.0,
        "pivot",
        12.0,
        PIVOT_FILL,
    ));
    sc = sc.add(centered_label(
        tile_x(step.store_index) + TILE_W / 2.0,
        ARRAY_Y + TILE_H + 56.0,
        "store",
        12.0,
        SMALL_FILL,
    ));
    if let QuickAction::Compare { index } = step.action {
        sc = sc.add(centered_label(
            tile_x(index) + TILE_W / 2.0,
            ARRAY_Y - 48.0,
            "candidate",
            12.0,
            CANDIDATE_FILL,
        ));
    }
    sc
}

fn final_scene(sorted: &[i32; N]) -> Scene {
    let mut sc = add_panel(scene());
    sc = add_header(sc, "Sorted output");
    sc = add_slots(sc);
    sc = add_array(sc, sorted, None, &[], true);
    sc
}
