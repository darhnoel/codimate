use crate::layout::{transformer_layout, BHW, BW, CDEC, CENC, DEC_LAYER_IDS, ENC_LAYER_IDS};
use crate::motion::*;
use crate::style::*;
use crate::{TransformerMotion, TransformerPhase, TransformerTiming, TransformerTrace};
use codimate_animation::{animation, sequence, Animation, Playable};
use codimate_arrange::Route;
use codimate_core::*;
use codimate_layout::{box_in, centered_text, Slot, Viewport};

fn draw_box(cx: f32, y: f32, h: f32, c: Color) -> PathNode {
    let slot = Slot::new(cx - BHW, y, BW, h);
    box_in(&slot)
        .radius(6.0)
        .fill(c)
        .stroke(2.5, BOX_STROKE)
        .into_node()
}

fn active_box(cx: f32, y: f32, h: f32, start: f32, span: f32) -> PathNode {
    let slot = Slot::new(cx - BHW - 4.0, y - 4.0, BW + 8.0, h + 8.0);
    box_in(&slot)
        .radius(8.0)
        .style(active_outline_style(
            start,
            span,
            BLOCK_HIGHLIGHT,
            5.0,
            0.90,
        ))
        .into_node()
}

fn mklbl(cx: f32, y: f32, h: f32, s: &str, bg: Color) -> Text {
    let slot = Slot::new(cx - BHW, y, BW, h);
    centered_text(&slot, s, 18.0, text_fill(bg))
}

fn luminance(c: Color) -> f32 {
    0.299 * c.r + 0.587 * c.g + 0.114 * c.b
}

fn text_fill(bg: Color) -> Color {
    if luminance(bg) > 0.5 {
        Color {
            r: 0.08,
            g: 0.08,
            b: 0.08,
            a: 1.0,
        }
    } else {
        INK
    }
}

fn vertical_conn(cx: f32, y1: f32, _h1: f32, y2: f32, h2: f32) -> Connection {
    let gap = 4.0;
    connection(Vec2::new(cx, y1 - gap), Vec2::new(cx, y2 + h2 + gap))
        .stroke(2.4, WIRE)
        .arrow(5.2)
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

fn qkv_arrows(mut s: Scene, cx: f32, ay: f32, ah: f32) -> Scene {
    let stub = 34.0;
    let origin_y = ay + ah + stub;
    let block_rect = rect().x(cx - BHW).y(ay).width(BW).height(ah);
    for j in 0..3 {
        let port = block_rect.anchor_port(AnchorKind::Bottom, j, 3);
        let p = port.resolve(0.0);
        let start_x = cx + (p.x - cx) * 0.45;
        s = s.node(
            connection(Vec2::new(start_x, origin_y), port)
                .stroke(2.3, WIRE)
                .arrow(5.6),
        );
    }
    s
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

fn container_rect(cx: f32, top: f32, bottom: f32) -> PathNode {
    let slot = Slot::new(cx - BHW - 42.0, top, BW + 84.0, bottom - top);
    box_in(&slot)
        .radius(10.0)
        .fill(CONTAINER_FILL)
        .stroke(2.5, CONTAINER_STROKE)
        .into_node()
}

fn residual_left(cx: f32, fy: f32, fh: f32, ty: f32, th: f32) -> Connection {
    residual_left_styled(
        cx,
        fy,
        fh,
        ty,
        th,
        ResidualStyle::new(2.1, RESIDUAL_WIRE, 5.0),
    )
}

fn residual_left_styled(
    cx: f32,
    fy: f32,
    fh: f32,
    ty: f32,
    th: f32,
    style: ResidualStyle,
) -> Connection {
    let off = 24.0;
    let arrow_gap = 7.0;
    let input_y = fy + fh + 4.0;
    let end_y = ty + th / 2.0;
    let source = Vec2::new(cx, input_y);
    let elbow = Vec2::new(cx - BHW - off, input_y);
    let target = Vec2::new(cx - BHW - arrow_gap, end_y);
    connection(source, target)
        .via([elbow, Vec2::new(cx - BHW - off, end_y)])
        .stroke(style.width, style.color)
        .arrow(style.arrow)
}

fn residual_right_styled(
    cx: f32,
    fy: f32,
    fh: f32,
    ty: f32,
    th: f32,
    style: ResidualStyle,
) -> Connection {
    let off = 24.0;
    let arrow_gap = 7.0;
    let input_y = fy + fh + 4.0;
    let end_y = ty + th / 2.0;
    let source = Vec2::new(cx, input_y);
    let elbow = Vec2::new(cx + BHW + off, input_y);
    let target = Vec2::new(cx + BHW + arrow_gap, end_y);
    connection(source, target)
        .via([elbow, Vec2::new(cx + BHW + off, end_y)])
        .stroke(style.width, style.color)
        .arrow(style.arrow)
}

fn residual_right(cx: f32, fy: f32, fh: f32, ty: f32, th: f32) -> Connection {
    residual_right_styled(
        cx,
        fy,
        fh,
        ty,
        th,
        ResidualStyle::new(2.1, RESIDUAL_WIRE, 5.0),
    )
}

fn residual_left_pulse(
    cx: f32,
    fy: f32,
    fh: f32,
    ty: f32,
    th: f32,
    p: impl IntoAnimated<f32>,
    c: Color,
) -> Pulse {
    pulse_on(residual_left(cx, fy, fh, ty, th), p)
        .radius(5.5)
        .fill(c)
}

fn residual_right_pulse(
    cx: f32,
    fy: f32,
    fh: f32,
    ty: f32,
    th: f32,
    p: impl IntoAnimated<f32>,
    c: Color,
) -> Pulse {
    pulse_on(residual_right(cx, fy, fh, ty, th), p)
        .radius(5.5)
        .fill(c)
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
        &Slot::new(wave_cx - 80.0, cy - 30.0, 160.0, 20.0),
        "Positional Encoding",
        16.0,
        INK,
    ));
    base = base.node(stroked_circle(cx, cy, 10.0, INK, 2.5, WIRE));
    base.node(
        text()
            .x(cx - 4.0)
            .y(cy + 5.0)
            .text("+")
            .font_size(20.0)
            .fill(INK),
    )
}

fn stroked_circle(cx: f32, cy: f32, r: f32, fill: Color, stroke: f32, sc: Color) -> PathNode {
    path_node()
        .path(circle_path(cx, cy, r))
        .fill(fill)
        .stroke(stroke, sc)
}

fn cross_bridge(
    stroke_width: impl IntoAnimated<f32>,
    stroke_color: impl IntoAnimated<Color>,
    arrow_size: impl IntoAnimated<f32>,
) -> Connection {
    let layout = transformer_layout();
    let route = layout
        .route("encoder", "add_norm_2", "decoder", "mha_dec")
        .expect("transformer cross route exists");
    routed_conn(route, stroke_width, stroke_color, arrow_size)
}

fn routed_conn(
    route: &Route,
    stroke_width: impl IntoAnimated<f32>,
    stroke_color: impl IntoAnimated<Color>,
    arrow_size: impl IntoAnimated<f32>,
) -> Connection {
    connection(route.start(), route.end())
        .via(route.waypoints().iter().copied())
        .stroke(stroke_width, stroke_color)
        .arrow(arrow_size)
}

fn add_encoder_highlights(mut s: Scene, ey: &[f32], enc_h: &[f32]) -> Scene {
    let block_times = [0.02, 0.18, 0.34, 0.52, 0.70];
    for (i, start) in block_times.into_iter().enumerate() {
        s = s.node(active_box(CENC, ey[i], enc_h[i], start, 0.24));
    }
    for i in 0..4 {
        s = s.node(active_vertical_conn(
            CENC,
            ey[i],
            enc_h[i],
            ey[i + 1],
            enc_h[i + 1],
            0.10 + i as f32 * 0.17,
            0.25,
        ));
    }
    s = s.node(residual_left_styled(
        CENC,
        ey[0],
        enc_h[0],
        ey[2],
        enc_h[2],
        ResidualStyle::new(
            active_width(0.28, 0.26, 3.2),
            active_color(BRIDGE_HIGHLIGHT, 0.28, 0.26, 0.75),
            active_width(0.28, 0.26, 4.8),
        ),
    ));
    s.node(residual_right_styled(
        CENC,
        ey[2],
        enc_h[2],
        ey[4],
        enc_h[4],
        ResidualStyle::new(
            active_width(0.66, 0.26, 3.2),
            active_color(BRIDGE_HIGHLIGHT, 0.66, 0.26, 0.75),
            active_width(0.66, 0.26, 4.8),
        ),
    ))
}

fn add_decoder_highlights(mut s: Scene, dy: &[f32], dec_h: &[f32]) -> Scene {
    for i in 0..9 {
        s = s.node(active_box(
            CDEC,
            dy[i],
            dec_h[i],
            0.02 + i as f32 * 0.095,
            0.18,
        ));
    }
    for i in 0..8 {
        s = s.node(active_vertical_conn(
            CDEC,
            dy[i],
            dec_h[i],
            dy[i + 1],
            dec_h[i + 1],
            0.06 + i as f32 * 0.10,
            0.17,
        ));
    }
    s = s.node(residual_left_styled(
        CDEC,
        dy[0],
        dec_h[0],
        dy[2],
        dec_h[2],
        ResidualStyle::new(
            active_width(0.20, 0.20, 3.0),
            active_color(BRIDGE_HIGHLIGHT, 0.20, 0.20, 0.70),
            active_width(0.20, 0.20, 4.5),
        ),
    ));
    s = s.node(residual_right_styled(
        CDEC,
        dy[2],
        dec_h[2],
        dy[4],
        dec_h[4],
        ResidualStyle::new(
            active_width(0.44, 0.20, 3.0),
            active_color(BRIDGE_HIGHLIGHT, 0.44, 0.20, 0.70),
            active_width(0.44, 0.20, 4.5),
        ),
    ));
    s.node(residual_left_styled(
        CDEC,
        dy[4],
        dec_h[4],
        dy[6],
        dec_h[6],
        ResidualStyle::new(
            active_width(0.66, 0.20, 3.0),
            active_color(BRIDGE_HIGHLIGHT, 0.66, 0.20, 0.70),
            active_width(0.66, 0.20, 4.5),
        ),
    ))
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
    pulse_on(vertical_conn(cx, y1, h1, y2, h2), p)
        .radius(5.5)
        .fill(c)
}

fn build_base_scene() -> Scene {
    let layout = transformer_layout();
    let enc = &layout["encoder"];
    let dec = &layout["decoder"];
    let enc_ey = enc.ey();
    let enc_h = enc.h();
    let dec_ey = dec.ey();
    let dec_h = dec.h();
    let enc_labels = enc.labels();
    let dec_labels = dec.labels();
    let enc_colors = enc.colors();
    let dec_colors = dec.colors();

    let mut base = scene();
    base = base.node(
        path_node()
            .path(rect_path(0.0, 0.0, 1000.0, 800.0))
            .fill(BG),
    );

    let enc_cont_top = 308.0;
    let enc_cont_bot = 610.0;
    let dec_cont_top = 168.0;
    let dec_cont_bot = 610.0;
    base = base.node(container_rect(CENC, enc_cont_top, enc_cont_bot));
    base = base.node(container_rect(CDEC, dec_cont_top, dec_cont_bot));

    for i in 0..5 {
        let color = enc_colors[i];
        base = base
            .node(draw_box(CENC, enc_ey[i], enc_h[i], color))
            .node(mklbl(CENC, enc_ey[i], enc_h[i], enc_labels[i], color));
    }

    for i in 0..9 {
        let color = dec_colors[i];
        base = base
            .node(draw_box(CDEC, dec_ey[i], dec_h[i], color))
            .node(mklbl(CDEC, dec_ey[i], dec_h[i], dec_labels[i], color));
    }

    base = qkv_arrows(base, CENC, enc_ey[1], enc_h[1]);
    base = qkv_arrows(base, CDEC, dec_ey[1], dec_h[1]);
    base = qkv_arrows(base, CDEC, dec_ey[3], dec_h[3]);

    let enc_cy = (enc_cont_top + enc_cont_bot) / 2.0 + 3.0;
    let dec_cy = (dec_cont_top + dec_cont_bot) / 2.0 + 3.0;
    base = base.node(
        text()
            .x(CENC - BHW - 76.0)
            .y(enc_cy)
            .text("N×")
            .font_size(24.0)
            .fill(INK),
    );
    base = base.node(
        text()
            .x(CDEC + BHW + 78.0)
            .y(dec_cy)
            .text("N×")
            .font_size(24.0)
            .fill(INK),
    );

    base = base.node(
        text()
            .x(CENC - 32.0)
            .y(696.0)
            .text("Inputs")
            .font_size(20.0)
            .fill(INK),
    );
    base = base.node(
        text()
            .x(CDEC - 36.0)
            .y(696.0)
            .text("Outputs")
            .font_size(20.0)
            .fill(INK),
    );
    base = base.node(
        text()
            .x(CDEC - 66.0)
            .y(724.0)
            .text("(shifted right)")
            .font_size(15.0)
            .fill(INK),
    );
    base = base.node(
        text()
            .x(CDEC - 78.0)
            .y(36.0)
            .text("Output Probabilities")
            .font_size(18.0)
            .fill(INK),
    );

    for i in 0..4 {
        let route = layout
            .route("encoder", ENC_LAYER_IDS[i], "encoder", ENC_LAYER_IDS[i + 1])
            .expect("encoder vertical route exists");
        base = base.node(routed_conn(route, 2.4, WIRE, 5.2));
    }
    for i in 0..8 {
        let route = layout
            .route("decoder", DEC_LAYER_IDS[i], "decoder", DEC_LAYER_IDS[i + 1])
            .expect("decoder vertical route exists");
        base = base.node(routed_conn(route, 2.4, WIRE, 5.2));
    }

    base = pos_encoding_group(base, CENC, 594.0, -138.0);
    base = pos_encoding_group(base, CDEC, 594.0, 138.0);

    for (from_col, from_layer, to_col, to_layer) in [
        ("encoder", "input_embedding", "encoder", "add_norm_1"),
        ("encoder", "add_norm_1", "encoder", "add_norm_2"),
        ("decoder", "output_embedding", "decoder", "add_norm_4"),
        ("decoder", "add_norm_4", "decoder", "add_norm_5"),
        ("decoder", "add_norm_5", "decoder", "add_norm_6"),
    ] {
        let route = layout
            .route(from_col, from_layer, to_col, to_layer)
            .expect("residual route exists");
        base = base.node(routed_conn(route, 2.1, RESIDUAL_WIRE, 5.0));
    }

    let route = layout
        .route("encoder", "add_norm_2", "decoder", "mha_dec")
        .expect("cross route exists");
    base = base.node(routed_conn(route, 2.0, WIRE, 6.5));
    base
}

fn build_encoder_scene(base: &Scene, show_pulses: bool) -> Scene {
    let layout = transformer_layout();
    let ey = layout["encoder"].ey();
    let h = layout["encoder"].h();
    let mut s = add_encoder_highlights(base.clone(), ey, h);
    if show_pulses {
        for i in 0..4 {
            s = s.node(vpulse(
                CENC,
                ey[i],
                h[i],
                ey[i + 1],
                h[i + 1],
                stagger(i as f32 * 0.18, 0.50),
                PULSE_C,
            ));
        }
        s = s.node(residual_left_pulse(
            CENC,
            ey[1],
            h[1],
            ey[2],
            h[2],
            stagger(0.28, 0.26),
            BRIDGE_HIGHLIGHT,
        ));
        s = s.node(residual_right_pulse(
            CENC,
            ey[3],
            h[3],
            ey[4],
            h[4],
            stagger(0.66, 0.26),
            BRIDGE_HIGHLIGHT,
        ));
    }
    s
}

fn build_decoder_scene(base: &Scene, show_pulses: bool) -> Scene {
    let layout = transformer_layout();
    let dy = layout["decoder"].ey();
    let h = layout["decoder"].h();
    let mut s = add_decoder_highlights(base.clone(), dy, h);
    if show_pulses {
        for i in 0..8 {
            s = s.node(vpulse(
                CDEC,
                dy[i],
                h[i],
                dy[i + 1],
                h[i + 1],
                stagger(i as f32 * 0.09, 0.28),
                PULSE_D,
            ));
        }
        s = s.node(residual_left_pulse(
            CDEC,
            dy[1],
            h[1],
            dy[2],
            h[2],
            stagger(0.20, 0.20),
            BRIDGE_HIGHLIGHT,
        ));
        s = s.node(residual_right_pulse(
            CDEC,
            dy[3],
            h[3],
            dy[4],
            h[4],
            stagger(0.44, 0.20),
            BRIDGE_HIGHLIGHT,
        ));
        s = s.node(residual_left_pulse(
            CDEC,
            dy[5],
            h[5],
            dy[6],
            h[6],
            stagger(0.66, 0.20),
            BRIDGE_HIGHLIGHT,
        ));
    }
    s
}

fn build_cross_scene(base: &Scene, show_pulses: bool) -> Scene {
    let layout = transformer_layout();
    let enc_ey = layout["encoder"].ey();
    let enc_h = layout["encoder"].h();
    let dec_ey = layout["decoder"].ey();
    let dec_h = layout["decoder"].h();
    let mut s = base
        .clone()
        .node(active_box(CENC, enc_ey[4], enc_h[4], 0.04, 0.38))
        .node(active_box(CDEC, dec_ey[3], dec_h[3], 0.42, 0.38))
        .node(cross_bridge(
            active_width(0.12, 0.76, 3.2),
            active_color(BRIDGE_HIGHLIGHT, 0.12, 0.76, 0.72),
            active_width(0.12, 0.76, 7.0),
        ));
    if show_pulses {
        s = s.node(
            pulse_on(cross_bridge(3.5, WIRE, 9.0), stagger(0.0, 1.0))
                .radius(6.0)
                .fill(PULSE_X),
        );
    }
    s
}

pub fn build_transformer(
    name: &'static str,
    trace: TransformerTrace,
    show_pulses: bool,
    _motion: TransformerMotion,
    timing: TransformerTiming,
) -> (Box<dyn Playable>, Viewport) {
    let base = build_base_scene();

    let mut anims: Vec<Animation> = Vec::new();
    for phase in &trace {
        let scene = match phase {
            TransformerPhase::Encoder => build_encoder_scene(&base, show_pulses),
            TransformerPhase::Decoder => build_decoder_scene(&base, show_pulses),
            TransformerPhase::Cross => build_cross_scene(&base, show_pulses),
        };
        let duration = match phase {
            TransformerPhase::Encoder => timing.encoder,
            TransformerPhase::Decoder => timing.decoder,
            TransformerPhase::Cross => timing.cross,
        };
        anims.push(animation(phase.name(), duration, scene));
    }

    (
        Box::new(sequence(name, anims)),
        Viewport::new(1000.0, 800.0),
    )
}
