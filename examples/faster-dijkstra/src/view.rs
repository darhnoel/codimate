use crate::{
    style::*, FasterDijkstra, FasterDijkstraAction, FasterDijkstraMotion, FasterDijkstraStep,
    FasterDijkstraTiming, FasterDijkstraTrace, NODE_COUNT,
};
use codimate::Viewport;
use codimate::*;
use codimate::{animation, sequence, Animation, Playable};

const VIEW_W: f32 = 1280.0;
const VIEW_H: f32 = 720.0;
const NODE_R: f32 = 25.0;
const GRAPH_Y_OFFSET: f32 = -18.0;
const CHIP_W: f32 = 50.0;
const CHIP_H: f32 = 32.0;
const CHIP_R: f32 = 5.0;
const QUEUE_X: f32 = 1088.0;
const QUEUE_Y: f32 = 190.0;
const QUEUE_GAP: f32 = 46.0;
const BATCH_Y: f32 = 535.0;
const BATCH_GAP: f32 = 78.0;

#[derive(Clone, Copy)]
pub struct FasterDijkstraView;

#[derive(Clone, Copy, PartialEq, Eq)]
enum NodeRole {
    Dim,
    Source,
    Frontier,
    Window,
    Pivot,
    Done,
}

pub fn faster_dijkstra_view() -> FasterDijkstraView {
    FasterDijkstraView
}

pub(crate) fn build_faster_dijkstra(
    name: &'static str,
    state: FasterDijkstra,
    trace: FasterDijkstraTrace,
    motion: FasterDijkstraMotion,
    timing: FasterDijkstraTiming,
) -> (Box<dyn Playable>, Viewport) {
    let mut anims: Vec<Animation> = Vec::new();
    for step in &trace.steps {
        anims.push(animation(
            format!("faster-dijkstra-step-{:02}", step.index),
            step_duration(step, timing),
            step_scene(state, step, motion),
        ));
    }

    (
        Box::new(sequence(name, anims)),
        Viewport::new(VIEW_W, VIEW_H),
    )
}

fn step_duration(step: &FasterDijkstraStep, timing: FasterDijkstraTiming) -> f32 {
    match step.action {
        FasterDijkstraAction::Problem => timing.intro,
        FasterDijkstraAction::Result => timing.result,
        _ => timing.concept,
    }
}

fn node_positions() -> [Vec2; NODE_COUNT] {
    [
        Vec2::new(250.0, 345.0 + GRAPH_Y_OFFSET),
        Vec2::new(410.0, 225.0 + GRAPH_Y_OFFSET),
        Vec2::new(410.0, 465.0 + GRAPH_Y_OFFSET),
        Vec2::new(585.0, 215.0 + GRAPH_Y_OFFSET),
        Vec2::new(585.0, 445.0 + GRAPH_Y_OFFSET),
        Vec2::new(755.0, 270.0 + GRAPH_Y_OFFSET),
        Vec2::new(755.0, 420.0 + GRAPH_Y_OFFSET),
        Vec2::new(900.0, 345.0 + GRAPH_Y_OFFSET),
    ]
}

fn text_width(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * font_size * 0.60
}

fn label(x: f32, y: f32, content: impl Into<String>, font_size: f32, fill: Color) -> Text {
    text()
        .x(x)
        .y(y)
        .text(content.into())
        .font_size(font_size)
        .fill(fill)
}

fn centered_label(
    x: f32,
    y: f32,
    content: impl Into<String>,
    font_size: f32,
    fill: impl IntoAnimated<Color>,
) -> Text {
    let content = content.into();
    text()
        .x(x - text_width(&content, font_size) / 2.0)
        .y(y)
        .text(content)
        .font_size(font_size)
        .fill(fill)
}

fn style(fill: Color, stroke_width: f32, stroke_color: Color) -> Style {
    Style::new().fill(fill).stroke(stroke_width, stroke_color)
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

fn fade_in(color: Color, start: f32) -> Animated<Color> {
    Animated::new(move |t| {
        let alpha = ((t - start) / (1.0 - start).max(0.01)).clamp(0.0, 1.0);
        Color { a: alpha, ..color }
    })
}

fn fade_out(color: Color, end: f32) -> Animated<Color> {
    Animated::new(move |t| {
        let alpha = (1.0 - (t / (end * 0.45).max(0.01))).clamp(0.0, 1.0);
        Color { a: alpha, ..color }
    })
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

fn chip_node(
    center: impl IntoAnimated<Vec2>,
    fill: impl IntoAnimated<Color>,
    stroke: impl IntoAnimated<Color>,
    stroke_width: impl IntoAnimated<f32>,
) -> Primitive {
    let center = center.into_animated();
    primitive_path(Animated::new(move |t| {
        let c = center.resolve(t);
        rounded_rect_path(
            c.x - CHIP_W / 2.0,
            c.y - CHIP_H / 2.0,
            CHIP_W,
            CHIP_H,
            CHIP_R,
        )
    }))
    .fill(fill)
    .stroke(stroke_width, stroke)
}

fn add_background(mut sc: Scene) -> Scene {
    sc = sc.add(primitive_path(rect_path(0.0, 0.0, VIEW_W, VIEW_H)).style(style(BG, 0.0, BG)));
    sc = sc.add(centered_label(
        VIEW_W / 2.0,
        52.0,
        "Faster Dijkstra: break the sorting barrier",
        28.0,
        INK,
    ));
    sc = sc.add(centered_label(
        VIEW_W / 2.0,
        82.0,
        "BMSSP: bounded multi-source shortest paths",
        14.0,
        MUTED,
    ));
    sc
}

fn add_color_key(mut sc: Scene) -> Scene {
    let items = [
        (250.0, SOURCE, "source"),
        (360.0, FRONTIER, "frontier"),
        (486.0, WINDOW, "inside B"),
        (620.0, PIVOT, "pivot"),
        (730.0, DONE, "complete"),
        (858.0, RELAX, "answer path"),
        (1000.0, DANGER, "Dijkstra cost"),
    ];
    for (x, color, label_text) in items {
        sc = sc.add(circle().x(x).y(112.0).radius(5.0).fill(color));
        sc = sc.add(label(x + 12.0, 117.0, label_text, 11.0, MUTED));
    }
    sc
}

fn step_scene(
    state: FasterDijkstra,
    step: &FasterDijkstraStep,
    motion: FasterDijkstraMotion,
) -> Scene {
    let pos = node_positions();
    let mut sc = add_background(scene());
    if matches!(step.action, FasterDijkstraAction::Result) {
        sc = add_structure(sc, step, motion, &pos);
        return add_subtitle(sc, step);
    }

    sc = add_color_key(sc);
    sc = add_bound_region(sc, step);
    sc = add_graph(sc, state, step, &pos);
    sc = add_structure(sc, step, motion, &pos);
    add_subtitle(sc, step)
}

fn add_graph(
    mut sc: Scene,
    state: FasterDijkstra,
    step: &FasterDijkstraStep,
    pos: &[Vec2; NODE_COUNT],
) -> Scene {
    for edge in state.edges {
        sc = add_edge(sc, edge.from, edge.to, edge.weight, pos, step);
    }
    for (index, p) in pos.iter().copied().enumerate() {
        sc = add_node(sc, p, state.labels[index], node_role(index, step));
    }
    sc
}

fn add_bound_region(mut sc: Scene, step: &FasterDijkstraStep) -> Scene {
    if !matches!(
        step.action,
        FasterDijkstraAction::QuestionShift
            | FasterDijkstraAction::BoundedWindow
            | FasterDijkstraAction::MultiSourceBatch
            | FasterDijkstraAction::KStepRelaxation
            | FasterDijkstraAction::FindPivots
            | FasterDijkstraAction::RecursiveBmssp
            | FasterDijkstraAction::BatchDataStructure
    ) {
        return sc;
    }

    sc = sc.add(
        primitive_path(rounded_rect_path(330.0, 150.0, 500.0, 355.0, 12.0)).style(style(
            with_alpha(WINDOW, 0.08),
            1.8,
            with_alpha(WINDOW, 0.65),
        )),
    );
    sc.add(label(350.0, 178.0, "bound B", 14.0, WINDOW))
}

fn add_edge(
    sc: Scene,
    from: usize,
    to: usize,
    weight: u32,
    pos: &[Vec2; NODE_COUNT],
    step: &FasterDijkstraStep,
) -> Scene {
    let start = edge_endpoint(pos[from], pos[to], NODE_R + 5.0);
    let end = edge_endpoint(pos[to], pos[from], NODE_R + 7.0);
    let mid = Vec2::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
    let color = if settled_edge(from, to, step) {
        DONE
    } else if active_edge(from, to, step) {
        RELAX
    } else {
        DIM
    };
    let width = if settled_edge(from, to, step) {
        3.0
    } else if active_edge(from, to, step) {
        2.6
    } else {
        1.3
    };

    let mut sc = sc.add(connection(start, end).stroke(width, color).arrow(3.0));
    sc = sc.add(centered_label(
        mid.x,
        mid.y - 7.0,
        weight.to_string(),
        12.0,
        if active_edge(from, to, step) {
            RELAX
        } else {
            MUTED
        },
    ));
    sc
}

fn edge_endpoint(from: Vec2, to: Vec2, inset: f32) -> Vec2 {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    Vec2::new(from.x + dx / len * inset, from.y + dy / len * inset)
}

fn active_edge(from: usize, to: usize, step: &FasterDijkstraStep) -> bool {
    match step.action {
        FasterDijkstraAction::Problem => from == 0 && (to == 1 || to == 2),
        FasterDijkstraAction::KStepRelaxation => {
            matches!((from, to), (1, 3) | (2, 4) | (4, 5) | (4, 6))
        }
        FasterDijkstraAction::RecursiveBmssp => matches!((from, to), (3, 5) | (5, 7) | (6, 7)),
        FasterDijkstraAction::BatchDataStructure => matches!((from, to), (4, 5) | (4, 6)),
        _ => false,
    }
}

fn settled_edge(from: usize, to: usize, step: &FasterDijkstraStep) -> bool {
    match step.action {
        FasterDijkstraAction::RecursiveBmssp => matches!((from, to), (0, 1) | (1, 3)),
        FasterDijkstraAction::BatchDataStructure => matches!((from, to), (0, 1) | (1, 3) | (3, 5)),
        FasterDijkstraAction::Result => {
            matches!((from, to), (0, 1) | (1, 3) | (3, 5) | (5, 7))
        }
        _ => false,
    }
}

fn node_role(index: usize, step: &FasterDijkstraStep) -> NodeRole {
    match step.action {
        FasterDijkstraAction::Problem => {
            if index == 0 {
                NodeRole::Source
            } else {
                NodeRole::Dim
            }
        }
        FasterDijkstraAction::SortingBarrier => match index {
            1..=5 => NodeRole::Frontier,
            0 => NodeRole::Source,
            _ => NodeRole::Dim,
        },
        FasterDijkstraAction::QuestionShift => match index {
            1 | 2 => NodeRole::Source,
            3..=5 => NodeRole::Window,
            0 => NodeRole::Done,
            _ => NodeRole::Dim,
        },
        FasterDijkstraAction::BoundedWindow => match index {
            1..=4 => NodeRole::Window,
            0 => NodeRole::Source,
            _ => NodeRole::Dim,
        },
        FasterDijkstraAction::MultiSourceBatch => match index {
            1 | 2 => NodeRole::Source,
            3..=5 => NodeRole::Window,
            _ => NodeRole::Dim,
        },
        FasterDijkstraAction::KStepRelaxation => match index {
            1 | 2 => NodeRole::Source,
            3..=6 => NodeRole::Window,
            _ => NodeRole::Dim,
        },
        FasterDijkstraAction::FindPivots => match index {
            3 | 4 => NodeRole::Pivot,
            1 | 2 | 5 | 6 => NodeRole::Window,
            _ => NodeRole::Dim,
        },
        FasterDijkstraAction::RecursiveBmssp => match index {
            0 | 1 | 3 | 5 => NodeRole::Done,
            4 | 6 => NodeRole::Pivot,
            _ => NodeRole::Dim,
        },
        FasterDijkstraAction::BatchDataStructure => match index {
            0 | 1 | 3 | 5 => NodeRole::Done,
            4 | 6 => NodeRole::Frontier,
            _ => NodeRole::Dim,
        },
        FasterDijkstraAction::Result => NodeRole::Done,
    }
}

fn node_fill(role: NodeRole) -> Color {
    match role {
        NodeRole::Dim => PANEL_2,
        NodeRole::Source => SOURCE,
        NodeRole::Frontier => FRONTIER,
        NodeRole::Window => WINDOW,
        NodeRole::Pivot => PIVOT,
        NodeRole::Done => DONE,
    }
}

fn add_node(mut sc: Scene, pos: Vec2, label_text: &str, role: NodeRole) -> Scene {
    let stroke = match role {
        NodeRole::Dim => PANEL_BORDER,
        NodeRole::Done => DONE,
        _ => INK,
    };
    sc = sc.add(
        circle()
            .x(pos.x)
            .y(pos.y)
            .radius(NODE_R)
            .fill(node_fill(role)),
    );
    sc = sc.add(
        primitive_path(circle_path(pos.x, pos.y, NODE_R))
            .fill(Color::TRANSPARENT)
            .stroke(if role == NodeRole::Dim { 1.2 } else { 2.2 }, stroke),
    );
    sc.add(centered_label(pos.x, pos.y + 6.0, label_text, 17.0, INK))
}

fn add_structure(
    sc: Scene,
    step: &FasterDijkstraStep,
    motion: FasterDijkstraMotion,
    pos: &[Vec2; NODE_COUNT],
) -> Scene {
    match step.action {
        FasterDijkstraAction::Problem => add_source_fan(sc, pos),
        FasterDijkstraAction::SortingBarrier => add_dijkstra_queue(sc, motion),
        FasterDijkstraAction::QuestionShift => add_queue_to_batch(sc, motion),
        FasterDijkstraAction::BoundedWindow => add_bounded_batch(sc),
        FasterDijkstraAction::MultiSourceBatch => add_sources_enter_bound(sc, motion),
        FasterDijkstraAction::KStepRelaxation => add_relaxation_layers(sc, motion, pos),
        FasterDijkstraAction::FindPivots => add_pivot_shrink(sc, motion),
        FasterDijkstraAction::RecursiveBmssp => add_recursive_regions(sc),
        FasterDijkstraAction::BatchDataStructure => add_batch_pull(sc, motion),
        FasterDijkstraAction::Result => add_result_compare(sc),
    }
}

fn add_source_fan(mut sc: Scene, pos: &[Vec2; NODE_COUNT]) -> Scene {
    sc = sc.add(label(166.0, 510.0, "source", 14.0, SOURCE));
    sc = sc.add(connection(Vec2::new(220.0, 505.0), pos[0]).stroke(1.2, SOURCE));
    sc
}

fn queue_position(i: usize) -> Vec2 {
    Vec2::new(QUEUE_X, QUEUE_Y + i as f32 * QUEUE_GAP)
}

fn batch_position(i: usize) -> Vec2 {
    Vec2::new(470.0 + i as f32 * BATCH_GAP, BATCH_Y)
}

fn add_chip(
    mut sc: Scene,
    center: impl IntoAnimated<Vec2>,
    text_value: &'static str,
    fill: impl IntoAnimated<Color>,
    stroke: impl IntoAnimated<Color>,
    text_fill: impl IntoAnimated<Color>,
) -> Scene {
    let center = center.into_animated();
    let label_center_x = center.clone();
    let label_center_y = center.clone();
    sc = sc.add(chip_node(center, fill, stroke, 1.4));
    sc.add(
        text()
            .x(Animated::new(move |t| {
                label_center_x.resolve(t).x - text_width(text_value, 14.0) / 2.0
            }))
            .y(Animated::new(move |t| label_center_y.resolve(t).y + 5.0))
            .text(text_value)
            .font_size(14.0)
            .fill(text_fill),
    )
}

fn add_dijkstra_queue(mut sc: Scene, motion: FasterDijkstraMotion) -> Scene {
    sc = sc.add(centered_label(
        QUEUE_X,
        143.0,
        "frontier order",
        14.0,
        MUTED,
    ));
    let labels = ["a", "b", "c", "d", "e"];
    for (i, value) in labels.iter().enumerate() {
        let from = queue_position(i);
        let to = if i == 0 {
            queue_position(5)
        } else {
            queue_position(i - 1)
        };
        let center = motion.ease(from, to);
        let fill = if i == 0 { DONE } else { FRONTIER };
        sc = add_chip(sc, center, value, fill, INK, INK);
    }
    sc = sc.add(
        connection(
            Vec2::new(QUEUE_X, queue_position(4).y + 24.0),
            Vec2::new(QUEUE_X, queue_position(5).y - 24.0),
        )
        .stroke(1.3, MUTED)
        .arrow(3.0),
    );
    sc.add(centered_label(
        QUEUE_X,
        queue_position(5).y + 46.0,
        "extract min",
        13.0,
        MUTED,
    ))
}

fn add_queue_to_batch(mut sc: Scene, motion: FasterDijkstraMotion) -> Scene {
    let labels = ["a", "b", "c", "d", "e"];
    for (i, value) in labels.iter().enumerate() {
        let target = if i < 2 {
            batch_position(i)
        } else {
            Vec2::new(650.0 + (i - 2) as f32 * 62.0, BATCH_Y)
        };
        sc = add_chip(
            sc,
            motion.ease(queue_position(i), target),
            value,
            if i < 2 { SOURCE } else { WINDOW },
            INK,
            INK,
        );
    }
    sc = sc.add(label(438.0, 581.0, "S", 15.0, SOURCE));
    sc.add(label(675.0, 581.0, "inside B", 15.0, WINDOW))
}

fn add_bounded_batch(mut sc: Scene) -> Scene {
    sc = sc.add(label(438.0, 581.0, "source set S", 15.0, SOURCE));
    for (i, value) in ["a", "b"].iter().enumerate() {
        sc = add_chip(sc, batch_position(i), value, SOURCE, INK, INK);
    }
    for (i, value) in ["c", "d", "e"].iter().enumerate() {
        sc = add_chip(
            sc,
            Vec2::new(650.0 + i as f32 * 62.0, BATCH_Y),
            value,
            WINDOW,
            INK,
            INK,
        );
    }
    sc
}

fn add_sources_enter_bound(mut sc: Scene, motion: FasterDijkstraMotion) -> Scene {
    let targets = [
        Vec2::new(410.0, 225.0 + GRAPH_Y_OFFSET),
        Vec2::new(410.0, 465.0 + GRAPH_Y_OFFSET),
    ];
    for (i, value) in ["a", "b"].iter().enumerate() {
        sc = add_chip(
            sc,
            motion.ease(batch_position(i), targets[i]),
            value,
            SOURCE,
            INK,
            INK,
        );
    }
    sc
}

fn expanding_circle(center: Vec2, from: f32, to: f32, color: Color) -> Primitive {
    primitive_path(Animated::new(move |t| {
        circle_path(center.x, center.y, f32::lerp(from, to, t))
    }))
    .fill(Color::TRANSPARENT)
    .stroke(
        2.0,
        Animated::new(move |t| with_alpha(color, 0.75 * (1.0 - t))),
    )
}

fn add_relaxation_layers(
    mut sc: Scene,
    _motion: FasterDijkstraMotion,
    pos: &[Vec2; NODE_COUNT],
) -> Scene {
    sc = sc.add(expanding_circle(pos[1], 35.0, 175.0, RELAX));
    sc = sc.add(expanding_circle(pos[2], 35.0, 145.0, RELAX));
    sc.add(label(702.0, 535.0, "k-step relaxation", 15.0, RELAX))
}

fn add_pivot_shrink(mut sc: Scene, motion: FasterDijkstraMotion) -> Scene {
    let candidates = ["c", "d", "e", "f"];
    let starts = [
        Vec2::new(470.0, 535.0),
        Vec2::new(536.0, 535.0),
        Vec2::new(602.0, 535.0),
        Vec2::new(668.0, 535.0),
    ];
    let targets = [
        Vec2::new(520.0, 535.0),
        Vec2::new(612.0, 535.0),
        Vec2::new(520.0, 535.0),
        Vec2::new(612.0, 535.0),
    ];
    for (i, value) in candidates.iter().enumerate() {
        let is_pivot = i < 2;
        sc = add_chip(
            sc,
            motion.ease(starts[i], targets[i]),
            value,
            if is_pivot {
                PIVOT.into_animated()
            } else {
                fade_out(WINDOW, 0.85)
            },
            if is_pivot {
                INK.into_animated()
            } else {
                fade_out(INK, 0.85)
            },
            if is_pivot {
                INK.into_animated()
            } else {
                fade_out(INK, 0.85)
            },
        );
    }
    sc.add(label(494.0, 581.0, "pivots P", 15.0, PIVOT))
}

fn add_recursive_regions(mut sc: Scene) -> Scene {
    sc = sc.add(
        primitive_path(rounded_rect_path(514.0, 165.0, 300.0, 150.0, 12.0)).style(style(
            with_alpha(PIVOT, 0.06),
            1.6,
            with_alpha(PIVOT, 0.65),
        )),
    );
    sc = sc.add(
        primitive_path(rounded_rect_path(520.0, 355.0, 305.0, 125.0, 12.0)).style(style(
            with_alpha(PIVOT, 0.06),
            1.6,
            with_alpha(PIVOT, 0.65),
        )),
    );
    sc = sc.add(label(822.0, 229.0, "smaller BMSSP", 14.0, PIVOT));
    sc.add(label(830.0, 430.0, "smaller BMSSP", 14.0, PIVOT))
}

fn add_batch_pull(mut sc: Scene, motion: FasterDijkstraMotion) -> Scene {
    for (i, value) in ["d", "f"].iter().enumerate() {
        sc = add_chip(
            sc,
            motion.ease(
                Vec2::new(640.0 + i as f32 * 62.0, 535.0),
                Vec2::new(520.0 + i as f32 * 70.0, 535.0),
            ),
            value,
            FRONTIER,
            INK,
            INK,
        );
    }
    sc.add(label(492.0, 581.0, "pull smallest batch", 15.0, FRONTIER))
}

fn add_result_compare(mut sc: Scene) -> Scene {
    sc = sc.add(centered_label(360.0, 166.0, "Dijkstra", 20.0, INK));
    sc = sc.add(centered_label(
        360.0,
        196.0,
        "asks for the exact next minimum",
        14.0,
        MUTED,
    ));
    sc = add_sorted_queue_recapped(sc, Vec2::new(360.0, 285.0));
    sc = sc.add(centered_label(
        360.0,
        512.0,
        "full frontier order",
        15.0,
        DANGER,
    ));
    sc = sc.add(centered_label(
        360.0,
        540.0,
        "sorting cost repeats",
        14.0,
        MUTED,
    ));

    sc = sc.add(centered_label(840.0, 166.0, "Faster method", 20.0, INK));
    sc = sc.add(centered_label(
        840.0,
        196.0,
        "asks which vertices finish inside B",
        14.0,
        MUTED,
    ));
    sc = add_bmssp_recapped(sc, Vec2::new(840.0, 305.0));
    sc = sc.add(centered_label(840.0, 512.0, "batch inside B", 15.0, WINDOW));
    sc = sc.add(centered_label(
        840.0,
        540.0,
        "then recurse from pivots",
        14.0,
        PIVOT,
    ));

    sc = sc.add(centered_label(
        VIEW_W / 2.0,
        586.0,
        "Color meaning here: red = exact ordering cost, green = sources, purple = inside B, orange = pivots.",
        13.0,
        fade_in(MUTED, 0.45),
    ));
    sc.add(centered_label(
        VIEW_W / 2.0,
        610.0,
        "That is the shift: avoid building the entire sorted order when a bounded batch is enough.",
        13.0,
        fade_in(INK, 0.58),
    ))
}

fn add_sorted_queue_recapped(mut sc: Scene, origin: Vec2) -> Scene {
    let x = origin.x;
    let y = origin.y;
    sc =
        sc.add(
            primitive_path(rounded_rect_path(x - 100.0, y - 75.0, 200.0, 160.0, 12.0)).style(
                style(with_alpha(DANGER, 0.05), 1.4, with_alpha(DANGER, 0.55)),
            ),
        );
    for (i, value) in ["b", "c", "d", "e"].iter().enumerate() {
        sc = add_chip(
            sc,
            Vec2::new(x, y - 42.0 + i as f32 * 38.0),
            value,
            FRONTIER,
            INK,
            INK,
        );
    }
    sc = sc.add(centered_label(x, y + 128.0, "extract one", 13.0, MUTED));
    sc.add(
        connection(Vec2::new(x, y + 86.0), Vec2::new(x, y + 110.0))
            .stroke(1.2, MUTED)
            .arrow(3.0),
    )
}

fn add_bmssp_recapped(mut sc: Scene, origin: Vec2) -> Scene {
    let x = origin.x;
    let y = origin.y;
    sc =
        sc.add(
            primitive_path(rounded_rect_path(x - 142.0, y - 88.0, 284.0, 130.0, 12.0)).style(
                style(with_alpha(WINDOW, 0.08), 1.5, with_alpha(WINDOW, 0.62)),
            ),
        );
    sc = sc.add(label(x - 126.0, y - 62.0, "bound B", 13.0, WINDOW));
    for (i, value) in ["a", "b"].iter().enumerate() {
        sc = add_chip(
            sc,
            Vec2::new(x - 72.0 + i as f32 * 56.0, y - 10.0),
            value,
            SOURCE,
            INK,
            INK,
        );
    }
    for (i, value) in ["c", "d", "e"].iter().enumerate() {
        sc = add_chip(
            sc,
            Vec2::new(x + 34.0 + i as f32 * 56.0, y - 10.0),
            value,
            WINDOW,
            INK,
            INK,
        );
    }
    for (i, value) in ["p1", "p2"].iter().enumerate() {
        sc = add_chip(
            sc,
            Vec2::new(x - 30.0 + i as f32 * 72.0, y + 72.0),
            value,
            PIVOT,
            INK,
            INK,
        );
    }
    sc = sc.add(
        connection(Vec2::new(x, y + 44.0), Vec2::new(x, y + 56.0))
            .stroke(1.2, MUTED)
            .arrow(3.0),
    );
    sc.add(centered_label(
        x,
        y + 114.0,
        "smaller subproblems",
        13.0,
        MUTED,
    ))
}

fn add_subtitle(mut sc: Scene, step: &FasterDijkstraStep) -> Scene {
    let (question, answer, accent) = qa(step);
    let q = format!("Q: {question}");
    let a = format!("A: {answer}");
    sc = sc.add(centered_label(VIEW_W / 2.0, 642.0, q, 18.0, accent));
    sc.add(centered_label(
        VIEW_W / 2.0,
        674.0,
        a,
        18.0,
        fade_in(INK, 0.48),
    ))
}

fn qa(step: &FasterDijkstraStep) -> (&'static str, &'static str, Color) {
    match step.action {
        FasterDijkstraAction::Problem => (
            "What is the shortest-path task?",
            "Find every distance from source s in a directed non-negative graph.",
            SOURCE,
        ),
        FasterDijkstraAction::SortingBarrier => (
            "Why does Dijkstra pay a sorting cost?",
            "It repeatedly needs the exact next minimum frontier node.",
            DANGER,
        ),
        FasterDijkstraAction::QuestionShift => (
            "What question does the faster method ask instead?",
            "Which vertices can be completed inside distance bound B?",
            WINDOW,
        ),
        FasterDijkstraAction::BoundedWindow => (
            "What does the bound B buy us?",
            "Far vertices stop competing in the current decision.",
            WINDOW,
        ),
        FasterDijkstraAction::MultiSourceBatch => (
            "Why start from many sources S?",
            "A bounded batch can be processed together instead of one-by-one.",
            SOURCE,
        ),
        FasterDijkstraAction::KStepRelaxation => (
            "How are nearby vertices completed?",
            "A few relaxation layers finish paths that stay close to S.",
            RELAX,
        ),
        FasterDijkstraAction::FindPivots => (
            "Why do pivots matter?",
            "Many candidates collapse into a few useful roots.",
            PIVOT,
        ),
        FasterDijkstraAction::RecursiveBmssp => (
            "What happens after pivots are found?",
            "Each pivot opens a smaller bounded shortest-path problem.",
            PIVOT,
        ),
        FasterDijkstraAction::BatchDataStructure => (
            "How is the next work chosen?",
            "Pull a small batch, then prepend improved vertices together.",
            FRONTIER,
        ),
        FasterDijkstraAction::Result => (
            "What do the colors mean in the final takeaway?",
            "Red is ordering cost; green starts the batch, purple stays inside B, orange becomes pivots.",
            MUTED,
        ),
    }
}
