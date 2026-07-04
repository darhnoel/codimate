//! system-diagram — a microservices architecture drawn as boxes + arrows,
//! with a request that animates down through the tiers.
//!
//! Hand-placed on a fixed 4-tier layout (client -> gateway -> services -> DBs).
//! ponytail: no graph/auto-layout crate yet — a fixed known topology needs
//! neither. Lift into `codimate-arrange` only when hand-placing edges hurts.

use codimate::*;

const VIEW_W: f32 = 1280.0;
const VIEW_H: f32 = 720.0;

/// Position along the edge: 0 before `t1`, 0->1 across [t1, t2], 1 after.
fn travel(t1: f32, t2: f32) -> Animated<f32> {
    Animated::new(move |t| {
        if t <= t1 {
            0.0
        } else if t >= t2 {
            1.0
        } else {
            (t - t1) / (t2 - t1)
        }
    })
}

/// Visibility alpha: 0 outside [t1, t2], with a few-frame fade in/out so the
/// dot only exists while in flight — no parked dots before launch or after arrival.
fn visibility(t1: f32, t2: f32) -> Animated<f32> {
    const FADE: f32 = 0.03; // fraction of the timeline
    Animated::new(move |t| {
        if t <= t1 || t >= t2 {
            0.0
        } else if t < t1 + FADE {
            (t - t1) / FADE
        } else if t > t2 - FADE {
            (t2 - t) / FADE
        } else {
            1.0
        }
    })
}

/// A slot centered at (cx, cy) — `Slot::new` is top-left, this is the diagram idiom.
fn node(cx: f32, cy: f32, w: f32, h: f32) -> Slot {
    Slot::new(cx - w / 2.0, cy - h / 2.0, w, h)
}

/// A downward arrow from `a`'s bottom to `b`'s top, plus a request dot that
/// travels it during [t1, t2].
fn hop(a: &Slot, b: &Slot, t1: f32, t2: f32) -> (Connection, Pulse) {
    let edge = connection(a.anchor(AnchorKind::Bottom), b.anchor(AnchorKind::Top))
        .stroke(3.0, manim::GRAY)
        .arrow(12.0);
    let alpha = visibility(t1, t2);
    let fill = Animated::new(move |t| {
        let mut c = manim::YELLOW;
        c.a *= alpha.resolve(t);
        c
    });
    let dot = pulse_on(edge.clone(), travel(t1, t2)).radius(7.0).fill(fill);
    (edge, dot)
}

fn system_diagram_scene() -> Scene {
    // Tiers, top -> bottom. Stack pushed down to reserve a header band for the
    // title and to balance top/bottom margins.
    let client = node(640.0, 120.0, 220.0, 64.0);
    let gateway = node(640.0, 265.0, 260.0, 72.0);

    let auth = node(290.0, 430.0, 240.0, 84.0);
    let orders = node(640.0, 430.0, 240.0, 84.0);
    let payments = node(990.0, 430.0, 240.0, 84.0);

    let auth_db = node(290.0, 615.0, 170.0, 72.0);
    let orders_db = node(640.0, 615.0, 170.0, 72.0);
    let payments_db = node(990.0, 615.0, 170.0, 72.0);

    let mut s = scene().add(
        primitive_path(rect_path(0.0, 0.0, VIEW_W, VIEW_H)).style(Style::new().fill(Color::BLACK)),
    );

    // Edges first (drawn under the boxes). Staggered dots make one request fan
    // out: client -> gateway, gateway -> each service, service -> its DB.
    for (edge, dot) in [
        hop(&client, &gateway, 0.05, 0.25),
        hop(&gateway, &auth, 0.28, 0.55),
        hop(&gateway, &orders, 0.28, 0.55),
        hop(&gateway, &payments, 0.28, 0.55),
        hop(&auth, &auth_db, 0.58, 0.85),
        hop(&orders, &orders_db, 0.58, 0.85),
        hop(&payments, &payments_db, 0.58, 0.85),
    ] {
        s = s.add(edge).add(dot);
    }

    // Boxes + labels, colored by role.
    let boxes = [
        (&client, "Client", manim::BLUE),
        (&gateway, "API Gateway", manim::GOLD),
        (&auth, "Auth Service", manim::TEAL),
        (&orders, "Order Service", manim::TEAL),
        (&payments, "Payment Service", manim::TEAL),
        (&auth_db, "Auth DB", manim::PURPLE),
        (&orders_db, "Orders DB", manim::PURPLE),
        (&payments_db, "Payments DB", manim::PURPLE),
    ];
    for (slot, label, color) in boxes {
        s = s
            .add(box_in(slot).radius(12.0).fill(color).stroke(2.0, manim::WHITE))
            .add(centered_text(slot, label, 22.0, manim::WHITE));
    }

    s.add(
        text()
            .x(VIEW_W / 2.0)
            .y(56.0)
            .text("Microservices request flow")
            .font_size(28.0)
            .align(TextAlign::Center),
    )
}

pub fn create() -> (Box<dyn Playable>, Viewport) {
    (
        Box::new(animation("system-diagram", 3.3, system_diagram_scene())),
        Viewport::new(VIEW_W, VIEW_H),
    )
}
