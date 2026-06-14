use crate::{
    style::*, ConnectionPulse, ConnectionPulseMotion, ConnectionPulseStep, ConnectionPulseTiming,
    ConnectionPulseTrace,
};
use codimate::Viewport;
use codimate::*;
use codimate::{animation, sequence, Animation, Playable};

const VIEW_W: f32 = 600.0;
const VIEW_H: f32 = 360.0;

#[derive(Clone, Copy)]
pub struct ConnectionPulseView;

pub fn connection_pulse_view() -> ConnectionPulseView {
    ConnectionPulseView
}

fn step_duration(step: ConnectionPulseStep, timing: ConnectionPulseTiming) -> f32 {
    match step {
        ConnectionPulseStep::Pulse => timing.pulse,
    }
}

fn pulse_scene(state: ConnectionPulse, motion: ConnectionPulseMotion) -> Scene {
    let box_a = rect()
        .x(state.left_x)
        .y(state.box_y)
        .width(state.box_w)
        .height(state.box_h)
        .fill(BOX_A);

    let box_b = rect()
        .x(state.right_x)
        .y(state.box_y)
        .width(state.box_w)
        .height(state.box_h)
        .fill(BOX_B);

    let conn = connection(
        box_a.anchor(AnchorKind::Right),
        box_b.anchor(AnchorKind::Left),
    )
    .stroke(3.0, WIRE)
    .arrow(12.0);

    let pulse = pulse_on(conn.clone(), motion.pulse_progress())
        .radius(6.0)
        .fill(PULSE);

    scene().add(box_a).add(box_b).add(conn).add(pulse)
}

fn step_scene(
    step: ConnectionPulseStep,
    state: ConnectionPulse,
    motion: ConnectionPulseMotion,
) -> Scene {
    match step {
        ConnectionPulseStep::Pulse => pulse_scene(state, motion),
    }
}

pub(crate) fn build_connection_pulse(
    name: &'static str,
    state: ConnectionPulse,
    trace: ConnectionPulseTrace,
    motion: ConnectionPulseMotion,
    timing: ConnectionPulseTiming,
) -> (Box<dyn Playable>, Viewport) {
    let anims: Vec<Animation> = trace
        .steps
        .into_iter()
        .enumerate()
        .map(|(index, step)| {
            animation(
                format!("connection-pulse-step-{index:02}"),
                step_duration(step, timing),
                step_scene(step, state, motion),
            )
        })
        .collect();

    (
        Box::new(sequence(name, anims)),
        Viewport::new(VIEW_W, VIEW_H),
    )
}
