use crate::{
    state::{Ball, SwapState},
    style::*,
    BallKey, Swap, SwapMotion, SwapMove, SwapTiming,
};
use codimate_animation::{animation, sequence, Playable};
use codimate_core::*;
use codimate_layout::Viewport;

const VIEW_W: f32 = 960.0;
const VIEW_H: f32 = 540.0;
const SLOT_W: f32 = 150.0;
const SLOT_H: f32 = 150.0;
const SLOT_Y: f32 = 210.0;
const BALL_R: f32 = 42.0;
const SCENE_LABEL_Y: f32 = 134.0;

#[derive(Clone, Copy)]
pub struct SwapView;

#[derive(Clone, Copy)]
struct Slot {
    id: &'static str,
    label: &'static str,
    x: f32,
}

pub fn swap_view() -> SwapView {
    SwapView
}

pub(crate) fn build_swap(
    name: &'static str,
    swap: Swap,
    moves: Vec<SwapMove>,
    motion: SwapMotion,
    timing: SwapTiming,
) -> (Box<dyn Playable>, Viewport) {
    let mut state = SwapState::start(swap);
    let mut animations = Vec::new();

    animations.push(animation(
        "start",
        1.0,
        state_scene("A and B start in their slots", state),
    ));

    for movement in moves {
        let ball = swap.ball(movement.ball);
        animations.push(animation(
            format!("{}-to-{}", movement.from, movement.to),
            timing.move_duration,
            move_scene(
                movement.during_title,
                state,
                ball,
                movement.from,
                movement.to,
                motion,
            ),
        ));
        state = state.apply(movement);
        if let Some(title) = movement.pause_title {
            animations.push(animation(
                format!("pause-after-{}", movement.to),
                timing.pause,
                state_scene(title, state),
            ));
        }
    }

    let play = sequence(
        name,
        animations
            .into_iter()
            .chain([animation("done", 1.2, state_scene("Swapped", state))]),
    );

    (Box::new(play), Viewport::new(VIEW_W, VIEW_H))
}

fn slots() -> [Slot; 3] {
    [
        Slot {
            id: "a",
            label: "A slot",
            x: 170.0,
        },
        Slot {
            id: "b",
            label: "B slot",
            x: 405.0,
        },
        Slot {
            id: "temp",
            label: "Temp",
            x: 640.0,
        },
    ]
}

fn slot(id: &'static str) -> Slot {
    slots()
        .into_iter()
        .find(|slot| slot.id == id)
        .expect("slot id exists")
}

fn slot_center(slot: Slot) -> Vec2 {
    Vec2::new(slot.x + SLOT_W / 2.0, SLOT_Y + SLOT_H / 2.0)
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

fn slot_style(stroke: Color, width: f32) -> Style {
    style(SLOT_FILL, width, stroke)
}

fn ball_color(ball: Ball) -> Color {
    match ball.key {
        BallKey::Left => BALL_A,
        BallKey::Right => BALL_B,
    }
}

fn background(mut sc: Scene) -> Scene {
    sc = sc.add(
        primitive_path(rect_path(0.0, 0.0, VIEW_W, VIEW_H))
            .style(style(BG, 0.0, BG)),
    );
    sc = sc.add(label(54.0, 58.0, "Swap", 32.0, INK));
    sc.add(label(
        54.0,
        92.0,
        "Use temp so one value is never overwritten",
        16.0,
        MUTED,
    ))
}

fn add_slot(mut sc: Scene, slot: Slot, active: Option<Color>) -> Scene {
    let stroke = active.unwrap_or(SLOT_STROKE);
    let width = if active.is_some() { 3.2 } else { 1.5 };
    sc = sc.add(
        primitive_path(rounded_rect_path(slot.x, SLOT_Y, SLOT_W, SLOT_H, 18.0))
            .style(slot_style(stroke, width)),
    );
    sc.add(centered_label(
        slot.x + SLOT_W / 2.0,
        SLOT_Y + SLOT_H + 36.0,
        slot.label,
        15.0,
        MUTED,
    ))
}

fn add_all_slots(
    mut sc: Scene,
    active_source: Option<&'static str>,
    active_target: Option<&'static str>,
) -> Scene {
    for slot in slots() {
        let active = if Some(slot.id) == active_source {
            Some(ACTIVE)
        } else if Some(slot.id) == active_target {
            Some(if slot.id == "temp" {
                TEMP_ACTIVE
            } else {
                ACTIVE
            })
        } else {
            None
        };
        sc = add_slot(sc, slot, active);
    }
    sc
}

fn add_ball_at(
    mut sc: Scene,
    ball: Ball,
    position: impl IntoAnimated<Vec2>,
    radius: impl IntoAnimated<f32>,
) -> Scene {
    let position = position.into_animated();
    let radius = radius.into_animated();
    let circle_pos = position.clone();
    let text_x_pos = position.clone();
    let text_y_pos = position.clone();
    let label = ball.label;
    let font_size = 30.0;

    sc = sc.add(
        circle()
            .x(Animated::new(move |t| circle_pos.resolve(t).x))
            .y(Animated::new(move |t| position.resolve(t).y))
            .radius(radius)
            .fill(ball_color(ball)),
    );
    sc.add(
        text()
            .x(Animated::new(move |t| {
                text_x_pos.resolve(t).x - text_width(label, font_size) / 2.0
            }))
            .y(Animated::new(move |t| text_y_pos.resolve(t).y + 11.0))
            .text(label)
            .font_size(font_size)
            .fill(INK),
    )
}

fn add_state_balls(mut sc: Scene, state: SwapState, moving: Option<Ball>) -> Scene {
    for (slot, ball) in [
        (slot("a"), state.a),
        (slot("b"), state.b),
        (slot("temp"), state.temp),
    ] {
        if let Some(ball) = ball {
            if Some(ball.key) == moving.map(|ball| ball.key) {
                continue;
            }
            sc = add_ball_at(sc, ball, slot_center(slot), BALL_R);
        }
    }
    sc
}

fn state_scene(title: &'static str, state: SwapState) -> Scene {
    let mut sc = background(scene());
    sc = sc.add(centered_label(
        VIEW_W / 2.0,
        SCENE_LABEL_Y,
        title,
        18.0,
        MUTED,
    ));
    sc = add_all_slots(sc, None, None);
    add_state_balls(sc, state, None)
}

fn move_scene(
    title: &'static str,
    before: SwapState,
    ball: Ball,
    from: &'static str,
    to: &'static str,
    motion: SwapMotion,
) -> Scene {
    let mut sc = background(scene());
    let from_center = slot_center(slot(from));
    let to_center = slot_center(slot(to));

    sc = sc.add(centered_label(
        VIEW_W / 2.0,
        SCENE_LABEL_Y,
        title,
        18.0,
        MUTED,
    ));
    sc = add_all_slots(sc, Some(from), Some(to));
    sc = add_state_balls(sc, before, Some(ball));
    add_ball_at(
        sc,
        ball,
        motion.moving_position(from_center, to_center),
        motion.lift_radius(BALL_R),
    )
}
