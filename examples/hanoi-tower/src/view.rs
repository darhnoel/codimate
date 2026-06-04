use crate::{style::*, HanoiMotion, HanoiMove, HanoiTiming, HanoiTrace, PEG_COUNT};
use codimate_animation::{animation, sequence, Animation, Playable};
use codimate_core::*;
use codimate_layout::Viewport;

const VIEW_W: f32 = 1000.0;
const VIEW_H: f32 = 620.0;
const BASE_Y: f32 = 452.0;
const PEG_H: f32 = 230.0;
const PEG_W: f32 = 10.0;
const DISK_H: f32 = 32.0;
const DISK_RADIUS: f32 = 8.0;
const MIN_DISK_W: f32 = 82.0;
const DISK_W_STEP: f32 = 36.0;
// Altitude of the horizontal carry: a fixed line above every post top, so a
// disk in flight clears all three pegs. Post tops sit at BASE_Y - PEG_H.
const TRAVEL_Y: f32 = BASE_Y - PEG_H - DISK_H / 2.0 - 14.0;

#[derive(Clone, Copy)]
pub struct HanoiView;

#[derive(Clone, Copy)]
struct PegLayout {
    x: f32,
    label: &'static str,
}

pub fn hanoi_view() -> HanoiView {
    HanoiView
}

pub(crate) fn build_hanoi(
    name: &'static str,
    trace: HanoiTrace,
    motion: HanoiMotion,
    timing: HanoiTiming,
) -> (Box<dyn Playable>, Viewport) {
    let mut anims: Vec<Animation> = Vec::new();

    anims.push(animation(
        "intro",
        timing.intro,
        state_scene("Move the whole tower from A to C", &trace.start, None, None),
    ));
    for movement in &trace.moves {
        anims.push(animation(
            format!("move-{:02}", movement.step),
            timing.move_disk,
            move_scene(&trace, movement, motion),
        ));
    }
    anims.push(animation(
        "done",
        timing.final_hold,
        state_scene(
            "Solved: all disks moved to C",
            &trace.final_state,
            None,
            None,
        ),
    ));

    (
        Box::new(sequence(name, anims)),
        Viewport::new(VIEW_W, VIEW_H),
    )
}

fn pegs() -> [PegLayout; PEG_COUNT] {
    [
        PegLayout {
            x: 210.0,
            label: "A",
        },
        PegLayout {
            x: 500.0,
            label: "B",
        },
        PegLayout {
            x: 790.0,
            label: "C",
        },
    ]
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

fn box_path(
    center: impl IntoAnimated<Vec2>,
    width: f32,
    height: f32,
    radius: f32,
) -> Animated<Path> {
    let center = center.into_animated();
    Animated::new(move |t| {
        let center = center.resolve(t);
        rounded_rect_path(
            center.x - width / 2.0,
            center.y - height / 2.0,
            width,
            height,
            radius,
        )
    })
}

fn style(fill: Color, stroke_width: f32, stroke_color: Color) -> Style {
    Style::new().fill(fill).stroke(stroke_width, stroke_color)
}

fn disk_width(disk: usize) -> f32 {
    MIN_DISK_W + (disk as f32 - 1.0) * DISK_W_STEP
}

fn disk_center(peg_index: usize, stack_index: usize) -> Vec2 {
    Vec2::new(
        pegs()[peg_index].x,
        BASE_Y - DISK_H / 2.0 - stack_index as f32 * (DISK_H + 4.0),
    )
}

fn moving_target_center(state: &[Vec<usize>; 3], to_peg: usize) -> Vec2 {
    disk_center(to_peg, state[to_peg].len())
}

fn add_background(mut sc: Scene, subtitle: impl Into<String>) -> Scene {
    sc = sc.node(
        path_node()
            .path(rect_path(0.0, 0.0, VIEW_W, VIEW_H))
            .style(style(BG, 0.0, BG)),
    );
    sc = sc.node(label(54.0, 52.0, "Tower of Hanoi", 30.0, INK));
    sc.node(label(54.0, 88.0, subtitle, 16.0, MUTED))
}

fn add_pegs(mut sc: Scene, active: Option<(usize, usize)>) -> Scene {
    for (index, peg) in pegs().iter().enumerate() {
        let color = if active.is_some_and(|(from, _)| from == index) {
            ACCENT
        } else if active.is_some_and(|(_, to)| to == index) {
            TARGET
        } else {
            PANEL_BORDER
        };
        sc = sc.node(
            path_node()
                .path(rounded_rect_path(peg.x - 132.0, BASE_Y, 264.0, 12.0, 6.0))
                .style(style(PANEL, 1.2, color)),
        );
        sc = sc.node(
            path_node()
                .path(rounded_rect_path(
                    peg.x - PEG_W / 2.0,
                    BASE_Y - PEG_H,
                    PEG_W,
                    PEG_H,
                    5.0,
                ))
                .style(style(PANEL, 1.2, color)),
        );
        sc = sc.node(centered_label(peg.x, BASE_Y + 48.0, peg.label, 18.0, MUTED));
    }
    sc
}

fn add_disk(sc: Scene, disk: usize, center: impl IntoAnimated<Vec2>, active: bool) -> Scene {
    let fill = DISK_COLORS[disk];
    let stroke = if active { MOVING } else { DISK_STROKE };
    sc.node(
        path_node()
            .path(box_path(center, disk_width(disk), DISK_H, DISK_RADIUS))
            .style(style(fill, if active { 2.6 } else { 1.6 }, stroke)),
    )
}

fn add_disks(mut sc: Scene, state: &[Vec<usize>; 3], hidden_disk: Option<usize>) -> Scene {
    for (peg_index, stack) in state.iter().enumerate().take(PEG_COUNT) {
        for (stack_index, disk) in stack.iter().copied().enumerate() {
            if Some(disk) == hidden_disk {
                continue;
            }
            sc = add_disk(sc, disk, disk_center(peg_index, stack_index), false);
        }
    }
    sc
}

fn add_move_counter(sc: Scene, movement: &HanoiMove, total: usize) -> Scene {
    sc.node(centered_label(
        VIEW_W / 2.0,
        132.0,
        format!("Move {} of {}", movement.step, total),
        16.0,
        MUTED,
    ))
}

fn state_scene(
    title: impl Into<String>,
    state: &[Vec<usize>; 3],
    active: Option<(usize, usize)>,
    hidden_disk: Option<usize>,
) -> Scene {
    let mut sc = add_background(scene(), title);
    sc = add_pegs(sc, active);
    add_disks(sc, state, hidden_disk)
}

fn move_scene(trace: &HanoiTrace, movement: &HanoiMove, motion: HanoiMotion) -> Scene {
    let from_index = movement.from.index();
    let to_index = movement.to.index();
    let from_stack_index = movement.state_before[from_index]
        .iter()
        .position(|disk| *disk == movement.disk)
        .expect("moving disk is in source stack");
    let from_center = disk_center(from_index, from_stack_index);
    let to_center = moving_target_center(&movement.state_before, to_index);
    let moving_pos = motion.lift_carry_drop(from_center, to_center, TRAVEL_Y);

    let mut sc = state_scene(
        movement.title(),
        &movement.state_before,
        Some((from_index, to_index)),
        Some(movement.disk),
    );
    sc = add_move_counter(sc, movement, trace.moves.len());
    add_disk(sc, movement.disk, moving_pos, true)
}
