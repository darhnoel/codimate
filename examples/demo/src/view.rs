use crate::{style::*, Demo, DemoMotion, DemoStep, DemoTiming, DemoTrace};
use codimate_animation::{animation, sequence, Animation, Playable};
use codimate_core::*;
use codimate_layout::Viewport;

const VIEW_W: f32 = 800.0;
const VIEW_H: f32 = 600.0;

#[derive(Clone, Copy)]
pub struct DemoView;

pub fn demo_view() -> DemoView {
    DemoView
}

fn step_duration(step: DemoStep, timing: DemoTiming) -> f32 {
    match step {
        DemoStep::GrowCircle => timing.grow_circle,
        DemoStep::MoveRect => timing.move_rect,
        DemoStep::MorphPath => timing.morph_path,
    }
}

fn step_scene(step: DemoStep, state: Demo, motion: DemoMotion) -> Scene {
    match step {
        DemoStep::GrowCircle => scene().add(
            circle()
                .x(motion.travel(state.circle_start_x, state.circle_end_x))
                .y(state.circle_y)
                .radius(motion.travel(state.circle_start_radius, state.circle_end_radius))
                .fill(RED),
        ),
        DemoStep::MoveRect => scene().add(
            rect()
                .x(state.rect_x)
                .y(motion.travel(state.rect_start_y, state.rect_end_y))
                .width(state.rect_w)
                .height(state.rect_h)
                .fill(BLUE),
        ),
        DemoStep::MorphPath => scene().add(
            primitive_path(motion.travel(
                    circle_path(400.0, 300.0, 80.0),
                    rect_path(300.0, 200.0, 200.0, 200.0),
                ))
                .fill(RED),
        ),
    }
}

pub(crate) fn build_demo(
    name: &'static str,
    state: Demo,
    trace: DemoTrace,
    motion: DemoMotion,
    timing: DemoTiming,
) -> (Box<dyn Playable>, Viewport) {
    let anims: Vec<Animation> = trace
        .steps
        .into_iter()
        .enumerate()
        .map(|(index, step)| {
            animation(
                format!("demo-step-{index:02}"),
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
