use crate::{
    style::*, Edge, NeuralAction, NeuralNet, NeuralNetMotion, NeuralNetTiming, NeuralStep,
    NeuralTrace, HIDDEN_COUNT, INPUT_COUNT, OUTPUT_COUNT,
};
use codimate_animation::{animation, sequence, Animation, Playable};
use codimate_core::*;
use codimate_layout::Viewport;

const VIEW_W: f32 = 900.0;
const VIEW_H: f32 = 600.0;
const INPUT_X: f32 = 140.0;
const HIDDEN_X: f32 = 450.0;
const OUTPUT_X: f32 = 760.0;
const CENTER_Y: f32 = 310.0;
const INPUT_GAP: f32 = 136.0;
const HIDDEN_GAP: f32 = 96.0;
const OUTPUT_GAP: f32 = 136.0;
const NEURON_R: f32 = 24.0;

#[derive(Clone, Copy)]
pub struct NeuralNetView;

#[derive(Clone, Copy)]
struct NetLayout {
    inputs: [Vec2; INPUT_COUNT],
    hiddens: [Vec2; HIDDEN_COUNT],
    outputs: [Vec2; OUTPUT_COUNT],
}

#[derive(Clone, Copy)]
enum EdgeState {
    Dim,
    Fired,
    Active,
}

pub fn neural_net_view() -> NeuralNetView {
    NeuralNetView
}

pub(crate) fn build_neural_net(
    name: &'static str,
    _state: NeuralNet,
    trace: NeuralTrace,
    motion: NeuralNetMotion,
    timing: NeuralNetTiming,
) -> (Box<dyn Playable>, Viewport) {
    let mut anims: Vec<Animation> = Vec::new();
    for step in &trace.steps {
        anims.push(animation(
            format!("neural-step-{:02}", step.index),
            step_duration(step, timing),
            step_scene(step, &trace, motion),
        ));
    }

    (
        Box::new(sequence(name, anims)),
        Viewport::new(VIEW_W, VIEW_H),
    )
}

fn step_duration(step: &NeuralStep, timing: NeuralNetTiming) -> f32 {
    match step.action {
        NeuralAction::ShowInputs => timing.show_inputs,
        NeuralAction::FireToHidden { .. } | NeuralAction::FireToOutput { .. } => timing.fire_group,
        NeuralAction::Hold => timing.final_hold,
    }
}

fn layout() -> NetLayout {
    NetLayout {
        inputs: stack_points(INPUT_X, CENTER_Y, INPUT_GAP),
        hiddens: stack_points(HIDDEN_X, CENTER_Y, HIDDEN_GAP),
        outputs: stack_points(OUTPUT_X, CENTER_Y, OUTPUT_GAP),
    }
}

fn stack_points<const N: usize>(x: f32, center_y: f32, gap: f32) -> [Vec2; N] {
    std::array::from_fn(|i| {
        let y = center_y + (i as f32 - (N - 1) as f32 / 2.0) * gap;
        Vec2::new(x, y)
    })
}

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

fn style(fill: Color, stroke_width: f32, stroke_color: Color) -> Style {
    Style::new().fill(fill).stroke(stroke_width, stroke_color)
}

fn add_background(mut sc: Scene, subtitle: impl Into<String>) -> Scene {
    sc = sc.add(
        primitive_path(rect_path(0.0, 0.0, VIEW_W, VIEW_H))
            .style(style(BG, 0.0, BG)),
    );
    sc = sc.add(label(54.0, 52.0, "Neural Net", 30.0, INK));
    sc.add(label(54.0, 88.0, subtitle, 16.0, MUTED))
}

fn edge_order(edge: Edge, trace: &NeuralTrace) -> usize {
    match edge {
        Edge::InputHidden { hidden, .. } => hidden + 1,
        Edge::HiddenOutput { output, .. } => trace.hidden_group_count + output + 1,
    }
}

fn edge_points(edge: Edge, net: &NetLayout) -> (Vec2, Vec2) {
    match edge {
        Edge::InputHidden { input, hidden } => (net.inputs[input], net.hiddens[hidden]),
        Edge::HiddenOutput { hidden, output } => (net.hiddens[hidden], net.outputs[output]),
    }
}

fn edge_state(edge: Edge, step: &NeuralStep, trace: &NeuralTrace) -> EdgeState {
    match step.action {
        NeuralAction::FireToHidden { hidden } if matches!(edge, Edge::InputHidden { hidden: edge_hidden, .. } if edge_hidden == hidden) => {
            EdgeState::Active
        }
        NeuralAction::FireToOutput { output } if matches!(edge, Edge::HiddenOutput { output: edge_output, .. } if edge_output == output) => {
            EdgeState::Active
        }
        _ if edge_order(edge, trace) < step.index => EdgeState::Fired,
        NeuralAction::Hold => EdgeState::Fired,
        _ => EdgeState::Dim,
    }
}

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

fn active_edge_path(from: Vec2, to: Vec2, motion: NeuralNetMotion) -> Animated<Path> {
    let animated_to = motion.fire(from, to);
    Animated::new(move |t| edge_path(from, animated_to.resolve(t), 5.0))
}

fn add_edge(
    sc: Scene,
    edge: Edge,
    step: &NeuralStep,
    trace: &NeuralTrace,
    net: &NetLayout,
    motion: NeuralNetMotion,
) -> Scene {
    let (from, to) = edge_points(edge, net);
    match edge_state(edge, step, trace) {
        EdgeState::Dim => sc.add(primitive_path(edge_path(from, to, 2.0)).fill(EDGE_DIM)),
        EdgeState::Fired => sc.add(primitive_path(edge_path(from, to, 3.0)).fill(EDGE_FIRE)),
        EdgeState::Active => sc.add(
            primitive_path(active_edge_path(from, to, motion))
                .fill(EDGE_FIRE),
        ),
    }
}

fn active_input(step: &NeuralStep, input: usize) -> bool {
    matches!(step.action, NeuralAction::FireToHidden { .. }) && input < INPUT_COUNT
}

fn active_hidden(step: &NeuralStep) -> Option<usize> {
    match step.action {
        NeuralAction::FireToHidden { hidden } => Some(hidden),
        _ => None,
    }
}

fn all_hidden_active(step: &NeuralStep) -> bool {
    matches!(step.action, NeuralAction::FireToOutput { .. })
}

fn active_output(step: &NeuralStep) -> Option<usize> {
    match step.action {
        NeuralAction::FireToOutput { output } => Some(output),
        _ => None,
    }
}

fn hidden_ready(hidden: usize, step: &NeuralStep) -> bool {
    step.index > hidden + 1
}

fn output_ready(output: usize, step: &NeuralStep, trace: &NeuralTrace) -> bool {
    step.index > trace.hidden_group_count + output + 1
}

fn add_neuron(
    sc: Scene,
    pos: Vec2,
    fill: impl IntoAnimated<Color>,
    active: bool,
    label_text: impl Into<String>,
) -> Scene {
    let stroke_width = if active { 3.0 } else { 1.6 };
    let label_text = label_text.into();
    let mut sc = sc.add(
        circle()
            .x(pos.x)
            .y(pos.y)
            .radius(if active { NEURON_R + 2.0 } else { NEURON_R })
            .fill(fill),
    );
    sc = sc.add(
        primitive_path(circle_path(
                pos.x,
                pos.y,
                if active { NEURON_R + 2.0 } else { NEURON_R },
            ))
            .style(style(Color::TRANSPARENT, stroke_width, STROKE)),
    );
    sc.add(centered_label(pos.x, pos.y + 6.0, label_text, 13.0, INK))
}

fn step_scene(step: &NeuralStep, trace: &NeuralTrace, motion: NeuralNetMotion) -> Scene {
    let net = layout();
    let mut sc = add_background(scene(), subtitle(step));

    for input in 0..INPUT_COUNT {
        for hidden in 0..HIDDEN_COUNT {
            sc = add_edge(
                sc,
                Edge::InputHidden { input, hidden },
                step,
                trace,
                &net,
                motion,
            );
        }
    }
    for hidden in 0..HIDDEN_COUNT {
        for output in 0..OUTPUT_COUNT {
            sc = add_edge(
                sc,
                Edge::HiddenOutput { hidden, output },
                step,
                trace,
                &net,
                motion,
            );
        }
    }

    for (i, pos) in net.inputs.iter().copied().enumerate() {
        let active = active_input(step, i);
        let fill = match step.action {
            NeuralAction::ShowInputs => motion.fire(NEURON_DIM, NEURON_ON),
            _ if active => motion.fire(NEURON_ON, NEURON_ACTIVE),
            _ => NEURON_ON.into_animated(),
        };
        sc = add_neuron(sc, pos, fill, active, format!("x{}", i + 1));
    }

    for (i, pos) in net.hiddens.iter().copied().enumerate() {
        let active = active_hidden(step) == Some(i) || all_hidden_active(step);
        let fill = if active {
            motion.fire(NEURON_DIM, NEURON_ACTIVE)
        } else if matches!(step.action, NeuralAction::Hold) || hidden_ready(i, step) {
            NEURON_ON.into_animated()
        } else {
            NEURON_DIM.into_animated()
        };
        sc = add_neuron(sc, pos, fill, active, format!("h{}", i + 1));
    }

    for (i, pos) in net.outputs.iter().copied().enumerate() {
        let active = active_output(step) == Some(i);
        let fill = if active {
            motion.fire(NEURON_DIM, NEURON_ACTIVE)
        } else if matches!(step.action, NeuralAction::Hold) || output_ready(i, step, trace) {
            NEURON_ON.into_animated()
        } else {
            NEURON_DIM.into_animated()
        };
        sc = add_neuron(sc, pos, fill, active, format!("y{}", i + 1));
    }

    sc = sc.add(centered_label(INPUT_X, 522.0, "inputs", 14.0, MUTED));
    sc = sc.add(centered_label(HIDDEN_X, 522.0, "hidden layer", 14.0, MUTED));
    sc.add(centered_label(OUTPUT_X, 522.0, "outputs", 14.0, MUTED))
}

fn subtitle(step: &NeuralStep) -> String {
    match step.action {
        NeuralAction::ShowInputs => "Inputs enter the network".to_string(),
        NeuralAction::FireToHidden { hidden } => {
            format!("Signals x1, x2, x3 fire to h{}", hidden + 1)
        }
        NeuralAction::FireToOutput { output } => {
            format!("Signals h1, h2, h3, h4 fire to y{}", output + 1)
        }
        NeuralAction::Hold => "All connections have fired".to_string(),
    }
}
