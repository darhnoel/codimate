use crate::{
    style::*, NewtonLawAction, NewtonLawStep, NewtonLawTrace, NewtonLawsMotion, NewtonLawsTiming,
};
use codimate_animation::{animation, sequence, Animation, Playable};
use codimate_core::*;
use codimate_layout::{box_at, box_in, row, Viewport};
use codimate_math;

const VIEW_W: f32 = 1280.0;
const VIEW_H: f32 = 720.0;
const FLOOR_Y: f32 = 455.0;
const CART_W: f32 = 118.0;
const CART_H: f32 = 54.0;
const CART_R: f32 = 8.0;
const MAG_W: f32 = 150.0;
const MAG_H: f32 = 56.0;
const MAG_Y: f32 = FLOOR_Y - 28.0;

#[derive(Clone, Copy)]
pub struct NewtonLawsView;

pub fn newton_laws_view() -> NewtonLawsView {
    NewtonLawsView
}

pub(crate) fn build_newton_laws(
    name: &'static str,
    trace: NewtonLawTrace,
    motion: NewtonLawsMotion,
    timing: NewtonLawsTiming,
) -> (Box<dyn Playable>, Viewport) {
    let mut anims: Vec<Animation> = Vec::new();
    for step in &trace.steps {
        anims.push(animation(
            format!("newton-law-step-{:02}", step.index),
            step_duration(step, timing),
            step_scene(step, motion),
        ));
    }

    (
        Box::new(sequence(name, anims)),
        Viewport::new(VIEW_W, VIEW_H),
    )
}

fn step_duration(step: &NewtonLawStep, timing: NewtonLawsTiming) -> f32 {
    match step.action {
        NewtonLawAction::Intro => timing.intro,
        NewtonLawAction::Summary => timing.summary,
        _ => timing.law,
    }
}

fn text_width(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * font_size * 0.60
}

fn centered_label(
    x: f32,
    y: f32,
    content: impl Into<String>,
    font_size: f32,
    fill: impl IntoAnimated<Color>,
) -> Text {
    text()
        .x(x)
        .y(y)
        .text(content.into())
        .font_size(font_size)
        .fill(fill)
        .align(TextAlign::Center)
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

fn add_background(mut sc: Scene, subtitle: &'static str) -> Scene {
    sc = sc.node(
        path_node()
            .path(rect_path(0.0, 0.0, VIEW_W, VIEW_H))
            .style(style(BG, 0.0, BG)),
    );
    sc = sc.node(centered_label(
        VIEW_W / 2.0,
        54.0,
        "Newton's Three Laws of Motion",
        30.0,
        INK,
    ));
    sc.node(centered_label(VIEW_W / 2.0, 88.0, subtitle, 15.0, MUTED))
}

fn add_floor(sc: Scene) -> Scene {
    sc.node(
        connection(Vec2::new(145.0, FLOOR_Y), Vec2::new(1135.0, FLOOR_Y))
            .stroke(1.4, TRACK)
            .arrow(0.0),
    )
}

fn add_cart(
    mut sc: Scene,
    center: impl IntoAnimated<Vec2>,
    fill: impl IntoAnimated<Color>,
    stroke: impl IntoAnimated<Color>,
    stroke_width: impl IntoAnimated<f32>,
) -> Scene {
    let center = center.into_animated();
    let wheel_a_x = center.clone();
    let wheel_a_y = center.clone();
    let wheel_b_x = center.clone();
    let wheel_b_y = center.clone();
    sc = sc.node(
        box_at(center, Vec2::new(CART_W, CART_H))
            .radius(CART_R)
            .fill(fill)
            .stroke(stroke_width, stroke),
    );
    sc = sc.node(
        circle()
            .x(Animated::new(move |t| wheel_a_x.resolve(t).x - 34.0))
            .y(Animated::new(move |t| wheel_a_y.resolve(t).y + 34.0))
            .radius(9.0)
            .fill(INK),
    );
    sc.node(
        circle()
            .x(Animated::new(move |t| wheel_b_x.resolve(t).x + 34.0))
            .y(Animated::new(move |t| wheel_b_y.resolve(t).y + 34.0))
            .radius(9.0)
            .fill(INK),
    )
}

fn add_arrow(
    mut sc: Scene,
    start: impl IntoAnimated<Vec2>,
    end: impl IntoAnimated<Vec2>,
    color: impl IntoAnimated<Color>,
    width: impl IntoAnimated<f32>,
    text_value: &'static str,
    text_pos: Vec2,
) -> Scene {
    sc = sc.node(connection(start, end).stroke(width, color).arrow(6.0));
    sc.node(centered_label(
        text_pos.x, text_pos.y, text_value, 16.0, INK,
    ))
}

fn bottom_takeaway(
    mut sc: Scene,
    law: &'static str,
    equation: &'static str,
    explanation: &'static str,
    accent: Color,
) -> Scene {
    sc = sc.node(centered_label(VIEW_W / 2.0, 608.0, law, 20.0, accent));
    sc = sc.node(centered_label(
        VIEW_W / 2.0,
        638.0,
        equation,
        18.0,
        fade_in(INK, 0.32),
    ));
    sc.node(centered_label(
        VIEW_W / 2.0,
        670.0,
        explanation,
        17.0,
        fade_in(MUTED, 0.50),
    ))
}

fn step_scene(step: &NewtonLawStep, motion: NewtonLawsMotion) -> Scene {
    match step.action {
        NewtonLawAction::Intro => intro_scene(),
        NewtonLawAction::FirstLaw => first_law_scene(motion),
        NewtonLawAction::SecondLaw => second_law_scene(step, motion),
        NewtonLawAction::ThirdLaw => third_law_scene(motion),
        NewtonLawAction::Summary => summary_scene(step),
    }
}

fn intro_scene() -> Scene {
    let mut sc = add_background(scene(), "A force changes motion; no force preserves it.");
    let cards = row()
        .origin(Vec2::new(210.0, 210.0))
        .cell_size(Vec2::new(220.0, 170.0))
        .gap(100.0)
        .count(3);

    for (i, (title, line_a, line_b, color)) in [
        ("1", "no net force", "no change in motion", OBJECT),
        ("2", "more force", "more acceleration", ACCEL),
        ("3", "forces come", "in equal pairs", REACTION),
    ]
    .iter()
    .enumerate()
    {
        let slot = &cards[i];
        let cx = slot.top_left().resolve(0.0).x + 110.0;
        sc = sc.node(
            box_in(slot)
                .radius(10.0)
                .style(style(with_alpha(CARD, 0.70), 1.5, *color)),
        );
        sc = sc.node(centered_label(cx, 265.0, *title, 34.0, *color));
        sc = sc.node(centered_label(cx, 316.0, *line_a, 16.0, INK));
        sc = sc.node(centered_label(cx, 342.0, *line_b, 16.0, INK));
    }
    bottom_takeaway(
        sc,
        "Mental model",
        "motion changes only when net force is present",
        "The video shows each law as a change in state.",
        LAW,
    )
}

fn first_law_scene(motion: NewtonLawsMotion) -> Scene {
    let mut sc = add_background(scene(), "Law 1: inertia");
    sc = add_floor(sc);

    for (x, alpha) in [(325.0, 0.22), (498.0, 0.35), (671.0, 0.50)] {
        sc = add_cart(
            sc,
            Vec2::new(x, FLOOR_Y - 44.0),
            with_alpha(OBJECT, alpha),
            with_alpha(INK, alpha),
            1.2,
        );
    }

    let cart_pos = motion.linear(
        Vec2::new(325.0, FLOOR_Y - 44.0),
        Vec2::new(844.0, FLOOR_Y - 44.0),
    );
    sc = add_cart(sc, cart_pos, OBJECT, INK, motion.pulse(1.4, 2.6));
    sc = add_arrow(
        sc,
        Vec2::new(848.0, FLOOR_Y - 88.0),
        Vec2::new(1018.0, FLOOR_Y - 88.0),
        OBJECT,
        2.2,
        "constant velocity",
        Vec2::new(930.0, FLOOR_Y - 112.0),
    );
    for node in centered_formula(r"F_{\text{net}} = 0", 640.0, 188.0, FORCE) {
        sc = sc.node(node);
    }
    bottom_takeaway(
        sc,
        "First law: inertia",
        "F_net = 0  ->  velocity stays constant",
        "An object keeps resting or moving straight unless a net force acts.",
        OBJECT,
    )
}

fn second_law_scene(step: &NewtonLawStep, motion: NewtonLawsMotion) -> Scene {
    let mut sc = add_background(scene(), "Law 2: force causes acceleration");
    sc = add_floor(sc);

    for (x, alpha) in [(300.0, 0.20), (430.0, 0.28), (610.0, 0.40)] {
        sc = add_cart(
            sc,
            Vec2::new(x, FLOOR_Y - 44.0),
            with_alpha(OBJECT, alpha),
            with_alpha(INK, alpha),
            1.0,
        );
    }

    let cart_pos = motion.accelerate(
        Vec2::new(300.0, FLOOR_Y - 44.0),
        Vec2::new(880.0, FLOOR_Y - 44.0),
    );

    // The push rides WITH the cart: a steady shove from just behind it, so the
    // force is clearly acting the whole time the cart keeps accelerating.
    let half = CART_W / 2.0;
    let len = 104.0;
    let gap = 8.0;
    let c_tail = cart_pos.clone();
    let c_head = cart_pos.clone();
    let c_lx = cart_pos.clone();
    let c_ly = cart_pos.clone();

    sc = add_cart(sc, cart_pos, OBJECT, INK, motion.pulse(1.4, 2.7));

    sc = sc.node(
        connection(
            Animated::new(move |t| {
                let c = c_tail.resolve(t);
                Vec2::new(c.x - half - gap - len, c.y)
            }),
            Animated::new(move |t| {
                let c = c_head.resolve(t);
                Vec2::new(c.x - half - gap, c.y)
            }),
        )
        .stroke(3.4, FORCE)
        .arrow(6.0),
    );
    let fw = text_width("force", 16.0);
    sc = sc.node(
        text()
            .x(Animated::new(move |t| {
                c_lx.resolve(t).x - half - gap - len / 2.0 - fw / 2.0
            }))
            .y(Animated::new(move |t| c_ly.resolve(t).y - 30.0))
            .text("force".to_string())
            .font_size(16.0)
            .fill(INK),
    );
    sc = add_arrow(
        sc,
        Vec2::new(885.0, FLOOR_Y - 106.0),
        Vec2::new(1070.0, FLOOR_Y - 106.0),
        ACCEL,
        2.5,
        "acceleration",
        Vec2::new(980.0, FLOOR_Y - 132.0),
    );
    let eq_latex = format!(
        r"\frac{{{:.0}\,\text{{N}}}}{{{:.0}\,\text{{kg}}}} = {:.0}\ \text{{m/s}}^2",
        step.force, step.mass, step.acceleration
    );
    for node in centered_formula(&eq_latex, 640.0, 184.0, INK) {
        sc = sc.node(node);
    }
    sc = bottom_takeaway(
        sc,
        "Second law",
        "",
        "For the same mass, a larger net force creates a larger acceleration.",
        ACCEL,
    );
    for node in centered_formula_fade(r"F = m a", VIEW_W / 2.0, 638.0, INK, 0.32) {
        sc = sc.node(node);
    }
    sc
}

/// A bar magnet centred on an animated point: two coloured pole halves, an
/// outline, and the pole letters. Everything rides with `center`.
fn add_magnet(
    mut sc: Scene,
    center: Animated<Vec2>,
    left_color: Color,
    right_color: Color,
    left_label: &'static str,
    right_label: &'static str,
) -> Scene {
    let hw = MAG_W / 2.0; // half width
    let hh = MAG_H / 2.0; // half height
    let q = MAG_W / 4.0; // centre of each pole half

    let c_left = center.clone();
    let c_right = center.clone();
    let c_outline = center.clone();
    let c_la_x = center.clone();
    let c_la_y = center.clone();
    let c_lb_x = center.clone();
    let c_lb_y = center.clone();

    sc = sc.node(
        path_node()
            .path(Animated::new(move |t| {
                let c = c_left.resolve(t);
                rect_path(c.x - hw, c.y - hh, hw, MAG_H)
            }))
            .fill(left_color),
    );
    sc = sc.node(
        path_node()
            .path(Animated::new(move |t| {
                let c = c_right.resolve(t);
                rect_path(c.x, c.y - hh, hw, MAG_H)
            }))
            .fill(right_color),
    );
    sc = sc.node(
        path_node()
            .path(Animated::new(move |t| {
                let c = c_outline.resolve(t);
                rect_path(c.x - hw, c.y - hh, MAG_W, MAG_H)
            }))
            .style(style(with_alpha(BG, 0.0), 1.8, INK)),
    );

    let lw = text_width(left_label, 24.0);
    let rw = text_width(right_label, 24.0);
    sc = sc.node(
        text()
            .x(Animated::new(move |t| c_la_x.resolve(t).x - q - lw / 2.0))
            .y(Animated::new(move |t| c_la_y.resolve(t).y + 8.0))
            .text(left_label.to_string())
            .font_size(24.0)
            .fill(INK),
    );
    sc.node(
        text()
            .x(Animated::new(move |t| c_lb_x.resolve(t).x + q - rw / 2.0))
            .y(Animated::new(move |t| c_lb_y.resolve(t).y + 8.0))
            .text(right_label.to_string())
            .font_size(24.0)
            .fill(INK),
    )
}

/// One field line in the gap: a vertical arc that bows outward and fades as the
/// magnets drift apart (the field weakens with distance).
fn field_line(x: f32, bow: f32, base_alpha: f32) -> PathNode {
    let half = 24.0;
    path_node()
        .path(Path {
            segments: vec![Segment::Quad(
                Vec2::new(x, MAG_Y - half),
                Vec2::new(x + bow, MAG_Y),
                Vec2::new(x, MAG_Y + half),
            )],
            closed: false,
        })
        .stroke(
            2.0,
            Animated::new(move |t| with_alpha(FORCE, base_alpha * (1.0 - t))),
        )
}

fn third_law_scene(motion: NewtonLawsMotion) -> Scene {
    let mut sc = add_background(scene(), "Law 3: action and reaction");
    sc = add_floor(sc);

    // Two magnets with like poles (N, red) facing the gap. They never touch,
    // yet repel and recoil apart by equal amounts about the centre (x = 640).
    let mag_a = motion.ease(Vec2::new(540.0, MAG_Y), Vec2::new(420.0, MAG_Y));
    let mag_b = motion.ease(Vec2::new(740.0, MAG_Y), Vec2::new(860.0, MAG_Y));

    // The field that carries the force across the empty gap.
    sc = sc.node(field_line(618.0, -10.0, 0.45));
    sc = sc.node(field_line(640.0, 0.0, 0.50));
    sc = sc.node(field_line(662.0, 10.0, 0.45));

    sc = add_magnet(sc, mag_a, OBJECT, REACTION, "S", "N");
    sc = add_magnet(sc, mag_b, REACTION, OBJECT, "N", "S");

    // Equal, opposite force arrows in the gap point outward: same length, same
    // height, opposite direction — even though nothing is touching.
    sc = add_arrow(
        sc,
        Vec2::new(632.0, FLOOR_Y - 120.0),
        Vec2::new(472.0, FLOOR_Y - 120.0),
        FORCE,
        2.6,
        "force on A",
        Vec2::new(552.0, FLOOR_Y - 148.0),
    );
    sc = add_arrow(
        sc,
        Vec2::new(648.0, FLOOR_Y - 120.0),
        Vec2::new(808.0, FLOOR_Y - 120.0),
        FORCE,
        2.6,
        "force on B",
        Vec2::new(728.0, FLOOR_Y - 148.0),
    );
    sc = sc.node(centered_label(
        640.0,
        185.0,
        "no contact, yet equal and opposite",
        24.0,
        INK,
    ));
    sc = bottom_takeaway(
        sc,
        "Third law",
        "",
        "The magnets never touch, yet each feels the same force at the same time.",
        REACTION,
    );
    for node in centered_formula_fade(
        r"F_{A\text{ on }B} = -F_{B\text{ on }A}",
        VIEW_W / 2.0,
        638.0,
        INK,
        0.32,
    ) {
        sc = sc.node(node);
    }
    sc
}

fn summary_scene(step: &NewtonLawStep) -> Scene {
    let mut sc = add_background(scene(), "The three laws in one picture");
    let cards = row()
        .origin(Vec2::new(100.0, 200.0))
        .cell_size(Vec2::new(300.0, 220.0))
        .gap(90.0)
        .count(3);

    for (i, (title, equation, body, color)) in [
        (
            "1. Inertia",
            r"F_{\text{net}} = 0",
            "motion stays unchanged",
            OBJECT,
        ),
        (
            "2. Acceleration",
            r"F = m a",
            "net force changes velocity",
            ACCEL,
        ),
        (
            "3. Interaction",
            r"F_{AB} = -F_{BA}",
            "forces appear in opposite pairs",
            REACTION,
        ),
    ]
    .iter()
    .enumerate()
    {
        let slot = &cards[i];
        let cx = slot.top_left().resolve(0.0).x + 150.0;
        sc = sc.node(
            box_in(slot)
                .radius(10.0)
                .style(style(with_alpha(PANEL, 0.90), 1.5, *color)),
        );
        sc = sc.node(centered_label(cx, 255.0, *title, 22.0, *color));
        for node in centered_formula(equation, cx, 315.0, INK) {
            sc = sc.node(node);
        }
        sc = sc.node(centered_label(cx, 370.0, *body, 15.0, MUTED));
    }

    sc = sc.node(centered_label(
        VIEW_W / 2.0,
        495.0,
        format!(
            "Example numbers: force {:.0}, mass {:.0}, acceleration {:.0}",
            step.force, step.mass, step.acceleration
        ),
        17.0,
        fade_in(MUTED, 0.25),
    ));
    bottom_takeaway(
        sc,
        "Takeaway",
        "Forces explain changes in motion",
        "First ask: what is the net force, and who is pushing whom?",
        LAW,
    )
}

// ── formula helpers ──────────────────────────────────────────────────────────

/// Try to compile a formula; on failure log a hint and return empty.
fn formula_glyphs(latex: &str, fill: Color) -> (Vec<PathNode>, f32, f32) {
    match codimate_math::formula(latex, fill) {
        Ok(f) => (f.glyphs, f.width, f.height),
        Err(e) => {
            eprintln!("[newton-laws] formula skipped (install `cargo install typst-cli`): {e:?}");
            (Vec::new(), 0.0, 0.0)
        }
    }
}

fn centered_formula(latex: &str, x: f32, y: f32, fill: Color) -> Vec<PathNode> {
    let (glyphs, width, height) = formula_glyphs(latex, fill);
    let ox = x - width / 2.0;
    let oy = y - height / 2.0;
    glyphs
        .into_iter()
        .map(|node| {
            let path = node.resolve(0.0).path;
            PathNode::new().path(path.translate(ox, oy)).fill(fill)
        })
        .collect()
}

fn centered_formula_fade(latex: &str, x: f32, y: f32, fill: Color, start: f32) -> Vec<PathNode> {
    let (glyphs, width, height) = formula_glyphs(latex, fill);
    let ox = x - width / 2.0;
    let oy = y - height / 2.0;
    glyphs
        .into_iter()
        .map(|node| {
            let path = node.resolve(0.0).path;
            PathNode::new()
                .path(path.translate(ox, oy))
                .fill(fade_in(fill, start))
        })
        .collect()
}
