use codimate_animation::{animation, sequence};
use codimate_core::*;
use codimate_export::{export_mp4, ExportConfig};
use codimate_layout::Viewport;

// ── layout constants ────────────────────────────────────────────────────
const CL: f32 = 280.0; // encoder column center
const CR: f32 = 720.0; // decoder column center
const BW: f32 = 220.0;
const BHW: f32 = BW / 2.0;
const EMB: f32 = 40.0;
const ATT: f32 = 52.0;
const AN_: f32 = 28.0;
const FFN: f32 = 52.0;
const LIN: f32 = 34.0;
const SM: f32 = 34.0;
const SHOW_PULSES: bool = false;

// ── colors ──────────────────────────────────────────────────────────────
const fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color { r, g, b, a: 1.0 }
}
const BG: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
const WIRE: Color = Color {
    r: 0.12,
    g: 0.14,
    b: 0.18,
    a: 0.82,
};
const BOX_STROKE: Color = Color {
    r: 0.08,
    g: 0.09,
    b: 0.12,
    a: 1.0,
};
const RESIDUAL_WIRE: Color = Color {
    r: 0.30,
    g: 0.32,
    b: 0.38,
    a: 1.0,
};
const POS_WIRE: Color = Color {
    r: 0.42,
    g: 0.44,
    b: 0.50,
    a: 1.0,
};
const CONTAINER_FILL: Color = Color {
    r: 0.93,
    g: 0.95,
    b: 0.98,
    a: 1.0,
};
const FLOW_HIGHLIGHT: Color = Color {
    r: 0.00,
    g: 0.35,
    b: 1.00,
    a: 1.0,
};
const BLOCK_HIGHLIGHT: Color = Color {
    r: 1.00,
    g: 0.78,
    b: 0.00,
    a: 1.0,
};
const BRIDGE_HIGHLIGHT: Color = Color {
    r: 0.00,
    g: 0.42,
    b: 0.95,
    a: 1.0,
};
const CONTAINER_STROKE: Color = Color {
    r: 0.20,
    g: 0.23,
    b: 0.30,
    a: 1.0,
};
const PULSE_C: Color = Color {
    r: 0.00,
    g: 0.45,
    b: 1.00,
    a: 1.0,
};
const PULSE_D: Color = Color {
    r: 0.72,
    g: 0.00,
    b: 0.95,
    a: 1.0,
};
const PULSE_X: Color = Color {
    r: 1.00,
    g: 0.05,
    b: 0.05,
    a: 1.0,
};

// ── helper functions ────────────────────────────────────────────────────
fn mkbx(cx: f32, y: f32, h: f32, c: Color) -> Rect {
    rect().x(cx - BHW).y(y).width(BW).height(h).fill(c)
}

fn draw_box(cx: f32, y: f32, h: f32, c: Color) -> PathNode {
    path_node()
        .path(rect_path(cx - BHW, y, BW, h))
        .fill(c)
        .stroke(2.5, BOX_STROKE)
}

fn active_box(cx: f32, y: f32, h: f32, start: f32, span: f32) -> PathNode {
    path_node()
        .path(rect_path(cx - BHW - 4.0, y - 4.0, BW + 8.0, h + 8.0))
        .fill(Color::TRANSPARENT)
        .stroke(
            active_width(start, span, 5.0),
            active_color(BLOCK_HIGHLIGHT, start, span, 0.90),
        )
}

fn label_width(s: &str, font_size: f32) -> f32 {
    s.chars().count() as f32 * font_size * 0.30
}

fn mklbl(cx: f32, y: f32, h: f32, s: &'static str) -> Text {
    let font_size = 14.0;
    let text_width = label_width(s, font_size);
    text()
        .x(cx - text_width)
        .y(y + h / 2.0 + font_size * 0.35)
        .text(s)
        .font_size(font_size)
        .fill(Color::BLACK)
}

fn centered_text(cx: f32, y: f32, s: &'static str, font_size: f32) -> Text {
    text()
        .x(cx - label_width(s, font_size))
        .y(y)
        .text(s)
        .font_size(font_size)
        .fill(Color::BLACK)
}

fn vertical_conn(cx: f32, y1: f32, _h1: f32, y2: f32, h2: f32) -> Connection {
    let gap = 4.0;
    connection(Vec2::new(cx, y1 - gap), Vec2::new(cx, y2 + h2 + gap))
        .stroke(1.8, WIRE)
        .arrow(3.0)
}

fn active_vertical_conn(
    cx: f32,
    y1: f32,
    _h1: f32,
    y2: f32,
    h2: f32,
    start: f32,
    span: f32,
) -> Connection {
    let gap = 4.0;
    connection(Vec2::new(cx, y1 - gap), Vec2::new(cx, y2 + h2 + gap))
        .stroke(
            active_width(start, span, 3.8),
            active_color(FLOW_HIGHLIGHT, start, span, 1.0),
        )
        .arrow(active_width(start, span, 3.6))
}

/// Three short Q/K/V stub arrows fanning into an attention box's bottom edge,
/// placed with evenly-divided anchor *ports* so they don't overlap.
fn qkv_arrows(mut s: Scene, cx: f32, ay: f32, ah: f32) -> Scene {
    let bx = mkbx(cx, ay, ah, Color::WHITE);
    let stub = 34.0;
    // Fan the three stubs out from a shared point below the box, so they read
    // as Q/K/V inputs (the outer two angle in toward the box).
    let origin_y = ay + ah + stub;
    for j in 0..3 {
        let port = bx.anchor_port(AnchorKind::Bottom, j, 3);
        let p = port.resolve(0.0); // boxes are static, so the port is constant
        let start_x = cx + (p.x - cx) * 0.45; // pull the outer stubs toward centre
        s = s.node(
            connection(Vec2::new(start_x, origin_y), port)
                .stroke(1.8, WIRE)
                .arrow(4.0),
        );
    }
    s
}

fn vpulse(
    cx: f32,
    y1: f32,
    h1: f32,
    y2: f32,
    h2: f32,
    p: impl IntoAnimated<f32>,
    c: Color,
) -> Pulse {
    let conn = vertical_conn(cx, y1, h1, y2, h2);
    pulse_on(conn, p).radius(5.5).fill(c)
}

fn stagger(offset: f32, span: f32) -> Animated<f32> {
    Animated::new(move |t| ((t - offset) / span).clamp(0.0, 1.0))
}

fn active_amount(t: f32, start: f32, span: f32) -> f32 {
    let u = ((t - start) / span).clamp(0.0, 1.0);
    if t < start || t > start + span {
        0.0
    } else {
        (std::f32::consts::PI * u).sin()
    }
}

fn active_width(start: f32, span: f32, max: f32) -> Animated<f32> {
    Animated::new(move |t| active_amount(t, start, span) * max)
}

fn active_color(color: Color, start: f32, span: f32, max_alpha: f32) -> Animated<Color> {
    Animated::new(move |t| Color {
        a: active_amount(t, start, span) * max_alpha,
        ..color
    })
}

fn stroked_circle(cx: f32, cy: f32, r: f32, fill: Color, stroke: f32, sc: Color) -> PathNode {
    path_node()
        .path(circle_path(cx, cy, r))
        .fill(fill)
        .stroke(stroke, sc)
}

fn line_path(from: Vec2, to: Vec2) -> Path {
    Path {
        segments: vec![Segment::Line(from, to)],
        closed: false,
    }
}

fn wave_path(cx: f32, cy: f32) -> Path {
    Path {
        segments: vec![
            Segment::Cubic(
                Vec2::new(cx - 16.0, cy),
                Vec2::new(cx - 12.0, cy - 7.0),
                Vec2::new(cx - 7.0, cy - 7.0),
                Vec2::new(cx - 3.0, cy),
            ),
            Segment::Cubic(
                Vec2::new(cx - 3.0, cy),
                Vec2::new(cx + 1.0, cy + 7.0),
                Vec2::new(cx + 7.0, cy + 7.0),
                Vec2::new(cx + 11.0, cy),
            ),
            Segment::Cubic(
                Vec2::new(cx + 11.0, cy),
                Vec2::new(cx + 15.0, cy - 7.0),
                Vec2::new(cx + 20.0, cy - 7.0),
                Vec2::new(cx + 24.0, cy),
            ),
        ],
        closed: false,
    }
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

fn container_rect(cx: f32, top: f32, bottom: f32) -> PathNode {
    path_node()
        .path(rounded_rect_path(
            cx - BHW - 42.0,
            top,
            BW + 84.0,
            bottom - top,
            10.0,
        ))
        .fill(CONTAINER_FILL)
        .stroke(2.5, CONTAINER_STROKE)
}

/// Local residual skip wrapping a sublayer into the following Add & Norm.
/// Left-side variant.
fn residual_left(cx: f32, fy: f32, fh: f32, ty: f32, th: f32) -> Connection {
    residual_left_styled(cx, fy, fh, ty, th, 1.3, RESIDUAL_WIRE, 3.2)
}

fn residual_left_styled(
    cx: f32,
    fy: f32,
    fh: f32,
    ty: f32,
    th: f32,
    width: impl IntoAnimated<f32>,
    color: impl IntoAnimated<Color>,
    arrow: impl IntoAnimated<f32>,
) -> Connection {
    let off = 6.0;
    let from_y = fy + fh / 2.0;
    let end_y = ty + th / 2.0;
    connection(
        Vec2::new(cx - BHW + 8.0, from_y),
        mkbx(cx, ty, th, Color::WHITE).anchor(AnchorKind::Left),
    )
    .via([
        Vec2::new(cx - BHW - off, from_y),
        Vec2::new(cx - BHW - off, end_y),
    ])
    .stroke(width, color)
    .arrow(arrow)
}

/// Right-side variant.
fn residual_right(cx: f32, fy: f32, fh: f32, ty: f32, th: f32) -> Connection {
    residual_right_styled(cx, fy, fh, ty, th, 1.3, RESIDUAL_WIRE, 3.2)
}

fn residual_right_styled(
    cx: f32,
    fy: f32,
    fh: f32,
    ty: f32,
    th: f32,
    width: impl IntoAnimated<f32>,
    color: impl IntoAnimated<Color>,
    arrow: impl IntoAnimated<f32>,
) -> Connection {
    let off = 6.0;
    let from_y = fy + fh / 2.0;
    let end_y = ty + th / 2.0;
    connection(
        Vec2::new(cx + BHW - 8.0, from_y),
        mkbx(cx, ty, th, Color::WHITE).anchor(AnchorKind::Right),
    )
    .via([
        Vec2::new(cx + BHW + off, from_y),
        Vec2::new(cx + BHW + off, end_y),
    ])
    .stroke(width, color)
    .arrow(arrow)
}

fn pos_encoding_group(mut base: Scene, cx: f32, cy: f32, label_dx: f32) -> Scene {
    let wave_cx = cx + label_dx;
    let line_start_x = if label_dx < 0.0 {
        wave_cx + 26.0
    } else {
        wave_cx - 18.0
    };
    let line_end_x = if label_dx < 0.0 { cx - 12.0 } else { cx + 12.0 };
    let wave_y = cy + 2.0;
    base = base.node(
        path_node()
            .path(wave_path(wave_cx, wave_y))
            .fill(Color::TRANSPARENT)
            .stroke(1.8, POS_WIRE),
    );
    base = base.node(
        path_node()
            .path(line_path(
                Vec2::new(line_start_x, wave_y),
                Vec2::new(line_end_x, cy + 8.0),
            ))
            .fill(Color::TRANSPARENT)
            .stroke(1.2, POS_WIRE),
    );
    base = base.node(centered_text(
        wave_cx,
        cy - 22.0,
        "Positional Encoding",
        13.0,
    ));
    base = base.node(stroked_circle(cx, cy, 10.0, Color::WHITE, 2.5, WIRE));
    base.node(
        text()
            .x(cx - 4.0)
            .y(cy + 5.0)
            .text("+")
            .font_size(16.0)
            .fill(Color::BLACK),
    )
}

fn cross_bridge(
    stroke_width: impl IntoAnimated<f32>,
    stroke_color: impl IntoAnimated<Color>,
    arrow_size: impl IntoAnimated<f32>,
) -> Connection {
    let start = Vec2::new(CL + BHW, 390.0);
    let end = Vec2::new(CR - BHW, 444.0);
    connection(start, end)
        .via([
            Vec2::new(CL + BHW + 84.0, 390.0),
            Vec2::new(CL + BHW + 84.0, 444.0),
            Vec2::new(CR - BHW - 18.0, 444.0),
        ])
        .stroke(stroke_width, stroke_color)
        .arrow(arrow_size)
}

fn add_encoder_highlights(mut s: Scene, ey: [f32; 5], enc_h: [f32; 5]) -> Scene {
    let block_times = [0.02, 0.18, 0.34, 0.52, 0.70];
    for (i, start) in block_times.into_iter().enumerate() {
        s = s.node(active_box(CL, ey[i], enc_h[i], start, 0.24));
    }
    for i in 0..4 {
        s = s.node(active_vertical_conn(
            CL,
            ey[i],
            enc_h[i],
            ey[i + 1],
            enc_h[i + 1],
            0.10 + i as f32 * 0.17,
            0.25,
        ));
    }
    s = s.node(residual_left_styled(
        CL,
        ey[1],
        enc_h[1],
        ey[2],
        enc_h[2],
        active_width(0.28, 0.26, 3.2),
        active_color(BRIDGE_HIGHLIGHT, 0.28, 0.26, 0.75),
        active_width(0.28, 0.26, 4.8),
    ));
    s.node(residual_right_styled(
        CL,
        ey[3],
        enc_h[3],
        ey[4],
        enc_h[4],
        active_width(0.66, 0.26, 3.2),
        active_color(BRIDGE_HIGHLIGHT, 0.66, 0.26, 0.75),
        active_width(0.66, 0.26, 4.8),
    ))
}

fn add_decoder_highlights(mut s: Scene, dy: [f32; 9], dec_h: [f32; 9]) -> Scene {
    for i in 0..9 {
        s = s.node(active_box(
            CR,
            dy[i],
            dec_h[i],
            0.02 + i as f32 * 0.095,
            0.18,
        ));
    }
    for i in 0..8 {
        s = s.node(active_vertical_conn(
            CR,
            dy[i],
            dec_h[i],
            dy[i + 1],
            dec_h[i + 1],
            0.06 + i as f32 * 0.10,
            0.17,
        ));
    }
    s = s.node(residual_left_styled(
        CR,
        dy[1],
        dec_h[1],
        dy[2],
        dec_h[2],
        active_width(0.20, 0.20, 3.0),
        active_color(BRIDGE_HIGHLIGHT, 0.20, 0.20, 0.70),
        active_width(0.20, 0.20, 4.5),
    ));
    s = s.node(residual_right_styled(
        CR,
        dy[3],
        dec_h[3],
        dy[4],
        dec_h[4],
        active_width(0.44, 0.20, 3.0),
        active_color(BRIDGE_HIGHLIGHT, 0.44, 0.20, 0.70),
        active_width(0.44, 0.20, 4.5),
    ));
    s.node(residual_left_styled(
        CR,
        dy[5],
        dec_h[5],
        dy[6],
        dec_h[6],
        active_width(0.66, 0.20, 3.0),
        active_color(BRIDGE_HIGHLIGHT, 0.66, 0.20, 0.70),
        active_width(0.66, 0.20, 4.5),
    ))
}

// ── main ────────────────────────────────────────────────────────────────
fn main() {
    // ── Layout: bottom-to-top = first box at high y, last box at low y ──
    // Encoder y-positions (bottom → top):
    //   embed @ 610  |  mha @ 526  |  an1 @ 484  |  ffn @ 418  |  an2 @ 376
    // Decoder y-positions (bottom → top):
    //   embed @ 610  |  mmha @ 526  |  an1 @ 484  |  mhax @ 418
    //   |  an2 @ 376  |  ffn @ 310  |  an3 @ 268  |  lin @ 218  |  sm @ 170
    let ey: [f32; 5] = [624.0, 526.0, 484.0, 418.0, 376.0];
    let dy: [f32; 9] = [
        624.0, 526.0, 484.0, 418.0, 376.0, 310.0, 268.0, 218.0, 170.0,
    ];
    let enc_h: [f32; 5] = [EMB, ATT, AN_, FFN, AN_];
    let dec_h: [f32; 9] = [EMB, ATT, AN_, ATT, AN_, FFN, AN_, LIN, SM];

    // ── Build base scene ──
    let mut base = scene();
    // Background
    base = base.node(
        path_node()
            .path(rect_path(0.0, 0.0, 1000.0, 800.0))
            .fill(BG),
    );

    // Draw repeated-stack containers behind their boxes. They must be first
    // among foreground nodes, otherwise their opaque fill hides labels.
    let enc_cont_top = 356.0;
    let enc_cont_bot = 610.0;
    let dec_cont_top = 248.0;
    let dec_cont_bot = 610.0;
    base = base.node(container_rect(CL, enc_cont_top, enc_cont_bot));
    base = base.node(container_rect(CR, dec_cont_top, dec_cont_bot));

    // Encoder boxes + labels
    for i in 0..5 {
        let color = match i {
            0 => rgb(1.0, 0.85, 0.85),
            1 => rgb(1.0, 0.82, 0.58),
            2 => rgb(1.0, 1.0, 0.72),
            3 => rgb(0.68, 0.94, 1.0),
            4 => rgb(1.0, 1.0, 0.72),
            _ => unreachable!(),
        };
        let label = match i {
            0 => "Input Embedding",
            1 => "Multi-Head Attention",
            2 => "Add & Norm",
            3 => "Feed Forward",
            4 => "Add & Norm",
            _ => unreachable!(),
        };
        base = base
            .node(draw_box(CL, ey[i], enc_h[i], color))
            .node(mklbl(CL, ey[i], enc_h[i], label));
    }

    // Decoder boxes + labels
    for i in 0..9 {
        let color = match i {
            0 => rgb(1.0, 0.85, 0.85),
            1 | 3 => rgb(1.0, 0.82, 0.58),
            2 | 4 | 6 => rgb(1.0, 1.0, 0.72),
            5 => rgb(0.68, 0.94, 1.0),
            7 => rgb(0.80, 0.80, 1.0),
            8 => rgb(0.72, 1.0, 0.72),
            _ => unreachable!(),
        };
        let label = match i {
            0 => "Output Embedding",
            1 => "Masked Multi-Head Attn",
            2 => "Add & Norm",
            3 => "Multi-Head Attention",
            4 => "Add & Norm",
            5 => "Feed Forward",
            6 => "Add & Norm",
            7 => "Linear",
            8 => "Softmax",
            _ => unreachable!(),
        };
        base = base
            .node(draw_box(CR, dy[i], dec_h[i], color))
            .node(mklbl(CR, dy[i], dec_h[i], label));
    }

    // Q/K/V triple-arrow inputs into the attention blocks (uses anchor ports).
    base = qkv_arrows(base, CL, ey[1], enc_h[1]); // encoder Multi-Head Attention
    base = qkv_arrows(base, CR, dy[1], dec_h[1]); // decoder Masked Multi-Head Attn
    base = qkv_arrows(base, CR, dy[3], dec_h[3]); // decoder Multi-Head Attention

    // N× labels
    base = base.node(
        text()
            .x(CL - BHW - 76.0)
            .y((enc_cont_top + enc_cont_bot) / 2.0 + 3.0)
            .text("N×")
            .font_size(20.0)
            .fill(Color::BLACK),
    );
    base = base.node(
        text()
            .x(CR + BHW + 78.0)
            .y((dec_cont_top + dec_cont_bot) / 2.0 + 3.0)
            .text("N×")
            .font_size(20.0)
            .fill(Color::BLACK),
    );

    // ── Inputs / Outputs labels ──
    base = base.node(
        text()
            .x(CL - 28.0)
            .y(700.0)
            .text("Inputs")
            .font_size(16.0)
            .fill(Color::BLACK),
    );
    base = base.node(
        text()
            .x(CR - 32.0)
            .y(700.0)
            .text("Outputs")
            .font_size(16.0)
            .fill(Color::BLACK),
    );
    base = base.node(
        text()
            .x(CR - 56.0)
            .y(724.0)
            .text("(shifted right)")
            .font_size(12.0)
            .fill(Color::BLACK),
    );

    // Output Probabilities label (above Softmax)
    base = base.node(
        text()
            .x(CR - 78.0)
            .y(146.0)
            .text("Output Probabilities")
            .font_size(15.0)
            .fill(Color::BLACK),
    );

    // ── Vertical connections (bottom → top) ──
    // Encoder: 4 connections
    for i in 0..4 {
        base = base.node(vertical_conn(CL, ey[i], enc_h[i], ey[i + 1], enc_h[i + 1]));
    }
    // Decoder: 8 connections
    for i in 0..8 {
        base = base.node(vertical_conn(CR, dy[i], dec_h[i], dy[i + 1], dec_h[i + 1]));
    }

    // ── Positional encoding groups ──
    base = pos_encoding_group(base, CL, 594.0, -138.0);
    base = pos_encoding_group(base, CR, 594.0, 138.0);

    // ── Residual skips (local, around each sublayer) ──
    // Encoder MHA → A&N 1 (left)
    base = base.node(residual_left(CL, ey[1], enc_h[1], ey[2], enc_h[2]));
    // Encoder FFN → A&N 2 (right, alternating)
    base = base.node(residual_right(CL, ey[3], enc_h[3], ey[4], enc_h[4]));
    // Decoder Masked MHA → A&N 1 (left)
    base = base.node(residual_left(CR, dy[1], dec_h[1], dy[2], dec_h[2]));
    // Decoder Cross MHA → A&N 2 (right)
    base = base.node(residual_right(CR, dy[3], dec_h[3], dy[4], dec_h[4]));
    // Decoder FFN → A&N 3 (left)
    base = base.node(residual_left(CR, dy[5], dec_h[5], dy[6], dec_h[6]));

    // ── Cross-attention connection ──
    // From encoder A&N 2 right side → decoder Multi-Head Attention left side
    base = base.node(cross_bridge(2.0, WIRE, 6.5));

    // ── Animation phases ──

    // Phase 1 — Encoder (3s), bottom → top
    let mut enc_scene = add_encoder_highlights(base.clone(), ey, enc_h);
    if SHOW_PULSES {
        for i in 0..4 {
            enc_scene = enc_scene.node(vpulse(
                CL,
                ey[i],
                enc_h[i],
                ey[i + 1],
                enc_h[i + 1],
                stagger(i as f32 * 0.18, 0.50),
                PULSE_C,
            ));
        }
    }

    // Phase 2 — Decoder (3s), bottom → top
    let mut dec_scene = add_decoder_highlights(base.clone(), dy, dec_h);
    if SHOW_PULSES {
        for i in 0..8 {
            dec_scene = dec_scene.node(vpulse(
                CR,
                dy[i],
                dec_h[i],
                dy[i + 1],
                dec_h[i + 1],
                stagger(i as f32 * 0.09, 0.28),
                PULSE_D,
            ));
        }
    }

    // Phase 3 — Cross-attention (1.5s)
    let mut cross_scene = base
        .node(active_box(CL, ey[4], enc_h[4], 0.04, 0.38))
        .node(active_box(CR, dy[3], dec_h[3], 0.42, 0.38))
        .node(cross_bridge(
            active_width(0.12, 0.76, 3.2),
            active_color(BRIDGE_HIGHLIGHT, 0.12, 0.76, 0.72),
            active_width(0.12, 0.76, 7.0),
        ));
    if SHOW_PULSES {
        cross_scene = cross_scene.node(
            pulse_on(cross_bridge(3.5, WIRE, 9.0), stagger(0.0, 1.0))
                .radius(6.0)
                .fill(PULSE_X),
        );
    }

    let play = sequence(
        "transformer",
        [
            animation("encoder", 3.0, enc_scene),
            animation("decoder", 3.0, dec_scene),
            animation("cross", 1.5, cross_scene),
        ],
    );

    let viewport = Viewport::new(1000.0, 800.0);
    let cfg = ExportConfig::new(30.0, viewport).crf(12);

    std::fs::create_dir_all("results").ok();
    println!("Exporting results/transformer.mp4 …");
    match export_mp4(&play, &cfg, "results/transformer.mp4") {
        Ok(()) => println!("Written results/transformer.mp4"),
        Err(e) => eprintln!("mp4 export skipped: {e}"),
    }
}
