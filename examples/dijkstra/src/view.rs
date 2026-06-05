use crate::{
    style::*, Dijkstra as DijkstraState, DijkstraAction, DijkstraMotion, DijkstraStep,
    DijkstraTiming, DijkstraTrace, NODE_COUNT,
};
use codimate_animation::{animation, sequence, Animation, Playable};
use codimate_core::*;
use codimate_layout::Viewport;

const VIEW_W: f32 = 900.0;
const VIEW_H: f32 = 600.0;
const NODE_R: f32 = 30.0;

const NODE_NAMES: [&str; NODE_COUNT] = ["A", "B", "C", "D", "E"];

#[derive(Clone, Copy)]
pub struct DijkstraView;

pub fn dijkstra_view() -> DijkstraView {
    DijkstraView
}

/// Fixed screen positions for the five nodes. Kept here in the view because
/// layout is a presentation concern, not part of the algorithm.
fn node_positions() -> [Vec2; NODE_COUNT] {
    [
        Vec2::new(150.0, 300.0), // A
        Vec2::new(400.0, 150.0), // B
        Vec2::new(400.0, 450.0), // C
        Vec2::new(650.0, 300.0), // D
        Vec2::new(840.0, 200.0), // E
    ]
}

pub(crate) fn build_dijkstra(
    name: &'static str,
    _state: DijkstraState,
    trace: DijkstraTrace,
    motion: DijkstraMotion,
    timing: DijkstraTiming,
) -> (Box<dyn Playable>, Viewport) {
    let mut anims: Vec<Animation> = Vec::new();
    for step in &trace.steps {
        anims.push(animation(
            format!("dijkstra-step-{:02}", step.index),
            step_duration(step, timing),
            step_scene(step, &trace, motion),
        ));
    }

    (
        Box::new(sequence(name, anims)),
        Viewport::new(VIEW_W, VIEW_H),
    )
}

fn step_duration(step: &DijkstraStep, timing: DijkstraTiming) -> f32 {
    match step.action {
        DijkstraAction::Init => timing.init,
        DijkstraAction::Settle { .. } => timing.settle,
        DijkstraAction::Done => timing.done,
    }
}

// ----- small text/style helpers (mirrors the other examples) -----

fn text_width(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * font_size * 0.30
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

fn add_background(mut sc: Scene, step: &DijkstraStep) -> Scene {
    sc = sc.node(
        path_node()
            .path(rect_path(0.0, 0.0, VIEW_W, VIEW_H))
            .style(style(BG, 0.0, BG)),
    );
    sc = sc.node(label(54.0, 52.0, "Dijkstra's Shortest Path", 30.0, INK));
    sc = sc.node(label(54.0, 88.0, subtitle(step), 16.0, MUTED));
    sc.node(label(
        54.0,
        VIEW_H - 28.0,
        distances_line(step),
        16.0,
        MUTED,
    ))
}

// ----- edges -----

#[derive(Clone, Copy)]
enum EdgeState {
    Dim,
    Tree,
    Relax,
}

/// An edge belongs to the shortest-path tree when one endpoint is the current
/// predecessor of the other.
fn is_tree_edge(a: usize, b: usize, step: &DijkstraStep) -> bool {
    step.pred[a] == Some(b) || step.pred[b] == Some(a)
}

/// An edge is being relaxed when one endpoint is the node settled this step and
/// the other is still unsettled.
fn is_relax_edge(a: usize, b: usize, step: &DijkstraStep) -> bool {
    match step.current {
        Some(cur) if a == cur => !step.visited[b],
        Some(cur) if b == cur => !step.visited[a],
        _ => false,
    }
}

fn edge_state(a: usize, b: usize, step: &DijkstraStep) -> EdgeState {
    if is_relax_edge(a, b, step) {
        EdgeState::Relax
    } else if is_tree_edge(a, b, step) {
        EdgeState::Tree
    } else {
        EdgeState::Dim
    }
}

/// A straight edge drawn as a thin filled quad (so it can carry a solid fill),
/// the same trick the neural-net example uses for its connections.
fn edge_path(from: Vec2, to: Vec2, th: f32) -> Path {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let px = -dy / len * th / 2.0;
    let py = dx / len * th / 2.0;
    Path {
        segments: vec![
            Segment::Line(
                Vec2::new(from.x + px, from.y + py),
                Vec2::new(from.x - px, from.y - py),
            ),
            Segment::Line(
                Vec2::new(from.x - px, from.y - py),
                Vec2::new(to.x - px, to.y - py),
            ),
            Segment::Line(
                Vec2::new(to.x - px, to.y - py),
                Vec2::new(to.x + px, to.y + py),
            ),
            Segment::Line(
                Vec2::new(to.x + px, to.y + py),
                Vec2::new(from.x + px, from.y + py),
            ),
        ],
        closed: true,
    }
}

fn add_edge(
    sc: Scene,
    a: usize,
    b: usize,
    weight: u32,
    pos: &[Vec2; NODE_COUNT],
    step: &DijkstraStep,
    motion: DijkstraMotion,
) -> Scene {
    let from = pos[a];
    let to = pos[b];
    let mid = Vec2::new((from.x + to.x) / 2.0, (from.y + to.y) / 2.0);

    let (line, weight_color): (Scene, Color) = match edge_state(a, b, step) {
        EdgeState::Dim => (
            sc.node(path_node().path(edge_path(from, to, 2.0)).fill(EDGE_DIM)),
            MUTED,
        ),
        EdgeState::Tree => (
            sc.node(path_node().path(edge_path(from, to, 4.0)).fill(EDGE_TREE)),
            INK,
        ),
        EdgeState::Relax => (
            sc.node(
                path_node()
                    .path(edge_path(from, to, 5.0))
                    .fill(motion.ease(EDGE_DIM, EDGE_RELAX)),
            ),
            EDGE_RELAX,
        ),
    };

    // Nudge the weight off the line so it stays readable.
    line.node(centered_label(
        mid.x,
        mid.y - 8.0,
        weight.to_string(),
        16.0,
        weight_color,
    ))
}

// ----- nodes -----

fn node_fill(n: usize, step: &DijkstraStep, motion: DijkstraMotion) -> Animated<Color> {
    if step.current == Some(n) {
        // Just popped from the frontier — flash to the "current" colour.
        motion.ease(NODE_FRONTIER, NODE_CURRENT)
    } else if step.visited[n] {
        NODE_SETTLED.into_animated()
    } else if step.dist[n].is_some() {
        if step.improved[n] {
            motion.ease(NODE_FAR, NODE_FRONTIER)
        } else {
            NODE_FRONTIER.into_animated()
        }
    } else {
        NODE_FAR.into_animated()
    }
}

fn dist_text(n: usize, step: &DijkstraStep) -> String {
    match step.dist[n] {
        Some(d) => format!("{}", d),
        None => "\u{221e}".to_string(), // ∞
    }
}

fn add_node(sc: Scene, n: usize, pos: Vec2, step: &DijkstraStep, motion: DijkstraMotion) -> Scene {
    let is_current = step.current == Some(n);
    let radius = if is_current {
        motion.ease(NODE_R, NODE_R + 4.0)
    } else {
        NODE_R.into_animated()
    };
    let stroke_width = if is_current || step.visited[n] {
        3.0
    } else {
        1.6
    };

    let mut sc = sc.node(
        circle()
            .x(pos.x)
            .y(pos.y)
            .radius(radius)
            .fill(node_fill(n, step, motion)),
    );
    sc = sc.node(
        path_node()
            .path(circle_path(pos.x, pos.y, NODE_R + 2.0))
            .style(style(Color::TRANSPARENT, stroke_width, STROKE)),
    );

    // Node name inside, distance below.
    sc = sc.node(centered_label(pos.x, pos.y + 6.0, NODE_NAMES[n], 22.0, INK));
    let dist_color: Color = if step.improved[n] { DIST_IMPROVED } else { INK };
    sc.node(centered_label(
        pos.x,
        pos.y + NODE_R + 24.0,
        format!("d={}", dist_text(n, step)),
        16.0,
        dist_color,
    ))
}

fn step_scene(step: &DijkstraStep, trace: &DijkstraTrace, motion: DijkstraMotion) -> Scene {
    let pos = node_positions();
    let mut sc = add_background(scene(), step);

    for &(a, b, w) in &trace.edges {
        sc = add_edge(sc, a, b, w, &pos, step, motion);
    }
    for n in 0..NODE_COUNT {
        sc = add_node(sc, n, pos[n], step, motion);
    }
    sc
}

fn subtitle(step: &DijkstraStep) -> String {
    match step.action {
        DijkstraAction::Init => {
            format!(
                "Start at {}: distance 0, every other node \u{221e}",
                NODE_NAMES[0]
            )
        }
        DijkstraAction::Settle { node } => {
            let d = step.dist[node].unwrap_or(0);
            format!(
                "Settle {} (d={}) — relax its unsettled neighbours",
                NODE_NAMES[node], d
            )
        }
        DijkstraAction::Done => "Every node settled — shortest paths from A found".to_string(),
    }
}

fn distances_line(step: &DijkstraStep) -> String {
    let mut parts = Vec::with_capacity(NODE_COUNT);
    for n in 0..NODE_COUNT {
        parts.push(format!("{}={}", NODE_NAMES[n], dist_text(n, step)));
    }
    format!("distances:  {}", parts.join("   "))
}
