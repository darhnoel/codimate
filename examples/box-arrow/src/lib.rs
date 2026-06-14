use codimate::*;

const VIEW_W: f32 = 900.0;
const VIEW_H: f32 = 540.0;
const T1: f32 = 0.25;
const T2: f32 = 0.75;

fn flow_progress() -> Animated<f32> {
    Animated::new(|t| {
        if t <= T1 {
            0.0
        } else if t >= T2 {
            1.0
        } else {
            (t - T1) / (T2 - T1)
        }
    })
}

fn box_arrow_scene() -> Scene {
    let box_a = rect()
        .x(160.0)
        .y(210.0)
        .width(200.0)
        .height(120.0)
        .fill(manim::BLUE);

    let box_b = rect()
        .x(540.0)
        .y(210.0)
        .width(200.0)
        .height(120.0)
        .fill(manim::GREEN);

    let arrow = connection(
        box_a.anchor(AnchorKind::Right),
        box_b.anchor(AnchorKind::Left),
    )
    .stroke(5.0, manim::WHITE)
    .arrow(18.0);

    let flow = pulse_on(arrow.clone(), flow_progress())
        .radius(10.0)
        .fill(manim::YELLOW);

    scene()
        .add(
            primitive_path(rect_path(0.0, 0.0, VIEW_W, VIEW_H))
                .style(Style::new().fill(Color::BLACK)),
        )
        .add(box_a)
        .add(box_b)
        .add(arrow)
        .add(flow)
        .add(text().x(250.0).y(275.0).text("A").font_size(42.0))
        .add(text().x(630.0).y(275.0).text("B").font_size(42.0))
        .add(
            text()
                .x(450.0)
                .y(80.0)
                .text("Arrow flow animates only between t1=0.25 and t2=0.75")
                .font_size(24.0),
        )
}

pub fn create() -> (Box<dyn Playable>, Viewport) {
    (
        Box::new(animation("box-arrow", 2.0, box_arrow_scene())),
        Viewport::new(VIEW_W, VIEW_H),
    )
}
