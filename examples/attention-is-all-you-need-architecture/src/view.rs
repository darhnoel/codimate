use crate::{
    style::*, TransformerArchitectureAction, TransformerArchitectureMotion,
    TransformerArchitectureStep, TransformerArchitectureTiming, TransformerArchitectureTrace,
};
use codimate_animation::{animation, sequence, Animation, Playable};
use codimate_core::*;
use codimate_layout::{box_at, box_in, centered_text, column, row, Slot, Viewport};

const VIEW_W: f32 = 1280.0;
const VIEW_H: f32 = 720.0;
const ENC_X: f32 = 410.0;
const DEC_X: f32 = 870.0;
const STACK_W: f32 = 250.0;
const LAYER_H: f32 = 42.0;
const NORM_H: f32 = 32.0;

#[derive(Clone, Copy)]
pub struct TransformerArchitectureView;

pub fn transformer_architecture_view() -> TransformerArchitectureView {
    TransformerArchitectureView
}

#[derive(Clone, Copy)]
enum Tone {
    Active,
    Context,
}

pub(crate) fn build_transformer_architecture(
    name: &'static str,
    trace: TransformerArchitectureTrace,
    motion: TransformerArchitectureMotion,
    timing: TransformerArchitectureTiming,
) -> (Box<dyn Playable>, Viewport) {
    let mut anims: Vec<Animation> = Vec::new();
    for step in &trace.steps {
        anims.push(animation(
            format!("transformer-architecture-step-{:02}", step.index + 1),
            step_duration(step, timing),
            step_scene(step, motion),
        ));
    }

    (
        Box::new(sequence(name, anims)),
        Viewport::new(VIEW_W, VIEW_H),
    )
}

fn step_duration(step: &TransformerArchitectureStep, timing: TransformerArchitectureTiming) -> f32 {
    match step.action {
        TransformerArchitectureAction::TranslationProblem
        | TransformerArchitectureAction::EncoderReadsDecoderWrites
        | TransformerArchitectureAction::OriginalLayout => timing.short,
        TransformerArchitectureAction::FullArchitecture => timing.final_reveal,
        _ => timing.normal,
    }
}

fn step_scene(step: &TransformerArchitectureStep, motion: TransformerArchitectureMotion) -> Scene {
    match step.action {
        TransformerArchitectureAction::TranslationProblem => translation_scene(step),
        TransformerArchitectureAction::EncoderReadsDecoderWrites => mental_model_scene(),
        TransformerArchitectureAction::OriginalLayout => original_layout_scene(),
        TransformerArchitectureAction::InputEmbedding => input_embedding_scene(step, motion),
        TransformerArchitectureAction::PositionalEncoding => positional_encoding_scene(),
        TransformerArchitectureAction::EncoderBlock => encoder_block_scene(),
        TransformerArchitectureAction::SelfAttentionIntuition => self_attention_scene(),
        TransformerArchitectureAction::MultiHeadAttentionIntuition => multi_head_scene(),
        TransformerArchitectureAction::AddNorm => add_norm_scene(motion),
        TransformerArchitectureAction::FeedForward => feed_forward_scene(),
        TransformerArchitectureAction::EncoderRepeats => encoder_repeats_scene(),
        TransformerArchitectureAction::EncoderMemory => encoder_memory_scene(),
        TransformerArchitectureAction::DecoderInput => decoder_input_scene(),
        TransformerArchitectureAction::MaskedSelfAttention => masked_self_attention_scene(),
        TransformerArchitectureAction::CrossAttention => cross_attention_scene(),
        TransformerArchitectureAction::DecoderBlock => decoder_block_scene(),
        TransformerArchitectureAction::LinearSoftmax => linear_softmax_scene(),
        TransformerArchitectureAction::FullArchitecture => full_architecture_scene(motion),
    }
}

fn base_scene(narration: &'static str) -> Scene {
    scene()
        .node(
            path_node()
                .path(rect_path(0.0, 0.0, VIEW_W, VIEW_H))
                .style(Style::new().fill(BG).stroke(0.0, BG)),
        )
        .node(centered_label(VIEW_W / 2.0, 650.0, narration, 23.0, INK))
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

fn slot_center(cx: f32, cy: f32, w: f32, h: f32) -> Slot {
    Slot::new(cx - w / 2.0, cy - h / 2.0, w, h)
}

fn style(fill: Color, tone: Tone) -> Style {
    match tone {
        Tone::Active => Style::new()
            .fill(alpha(fill, 0.86))
            .stroke(2.4, alpha(INK, 0.95)),
        Tone::Context => Style::new()
            .fill(alpha(fill, 0.25))
            .stroke(1.5, alpha(INK, 0.30)),
    }
}

fn label_color(tone: Tone) -> Color {
    match tone {
        Tone::Active => INK,
        Tone::Context => alpha(INK, 0.44),
    }
}

fn box_label(
    mut sc: Scene,
    slot: &Slot,
    label: impl Into<String>,
    color: Color,
    tone: Tone,
) -> Scene {
    sc = sc.node(box_in(slot).radius(8.0).style(style(color, tone)));
    sc.node(centered_text(slot, label, 17.0, label_color(tone)))
}

fn thin_label_box(
    mut sc: Scene,
    cx: f32,
    cy: f32,
    w: f32,
    label: impl Into<String>,
    color: Color,
    tone: Tone,
) -> Scene {
    let slot = slot_center(cx, cy, w, 38.0);
    sc = sc.node(box_in(&slot).radius(7.0).style(style(color, tone)));
    sc.node(centered_text(&slot, label, 16.0, label_color(tone)))
}

fn arrow(sc: Scene, from: Vec2, to: Vec2, tone: Tone) -> Scene {
    let (width, color, head) = match tone {
        Tone::Active => (2.8, WIRE, 7.0),
        Tone::Context => (1.7, CONTEXT_WIRE, 5.0),
    };
    sc.node(connection(from, to).stroke(width, color).arrow(head))
}

fn elbow_arrow(sc: Scene, from: Vec2, mid: Vec2, to: Vec2, tone: Tone) -> Scene {
    let (width, color, head) = match tone {
        Tone::Active => (2.6, WIRE, 6.5),
        Tone::Context => (1.6, CONTEXT_WIRE, 5.0),
    };
    sc.node(
        connection(from, to)
            .via([mid, Vec2::new(mid.x, to.y)])
            .stroke(width, color)
            .arrow(head),
    )
}

fn vertical_arrow(sc: Scene, cx: f32, from_y: f32, to_y: f32, tone: Tone) -> Scene {
    arrow(sc, Vec2::new(cx, from_y), Vec2::new(cx, to_y), tone)
}

fn plus(sc: Scene, x: f32, y: f32, tone: Tone) -> Scene {
    let color = label_color(tone);
    sc.node(centered_label(x, y + 7.0, "+", 28.0, color))
}

fn translation_scene(step: &TransformerArchitectureStep) -> Scene {
    let mut sc =
        base_scene("The Transformer was originally designed to turn one sequence into another.");
    let input = step.input.join(" ");
    sc = sc.node(centered_label(360.0, 330.0, input, 40.0, INK));
    sc = arrow(
        sc,
        Vec2::new(510.0, 320.0),
        Vec2::new(760.0, 320.0),
        Tone::Active,
    );
    sc.node(centered_label(930.0, 330.0, step.output, 38.0, INK))
}

fn mental_model_scene() -> Scene {
    let mut sc = base_scene("The encoder reads the input. The decoder writes the output.");
    let slots = row()
        .origin(Vec2::new(145.0, 280.0))
        .cell_size(Vec2::new(190.0, 92.0))
        .gap(70.0)
        .count(4);
    for (i, (label, color)) in [
        ("Input", EMBEDDING),
        ("Encoder", ENCODER),
        ("Decoder", DECODER),
        ("Output", OUTPUT),
    ]
    .into_iter()
    .enumerate()
    {
        sc = box_label(sc, &slots[i], label, color, Tone::Active);
    }
    for i in 0..3 {
        let start = slots[i].right().resolve(0.0);
        let end = slots[i + 1].left().resolve(0.0);
        sc = arrow(sc, start, end, Tone::Active);
    }
    sc
}

fn original_layout_scene() -> Scene {
    let mut sc =
        base_scene("The encoder creates memory. The decoder uses that memory to generate output.");
    let enc = slot_center(ENC_X, 330.0, 260.0, 210.0);
    let dec = slot_center(DEC_X, 330.0, 260.0, 210.0);
    sc = box_label(sc, &enc, "Encoder", ENCODER, Tone::Active);
    sc = box_label(sc, &dec, "Decoder", DECODER, Tone::Active);
    arrow(
        sc,
        Vec2::new(540.0, 330.0),
        Vec2::new(740.0, 330.0),
        Tone::Active,
    )
}

fn input_embedding_scene(
    step: &TransformerArchitectureStep,
    motion: TransformerArchitectureMotion,
) -> Scene {
    let mut sc = base_scene("Words are converted into vectors.");
    let enc = slot_center(ENC_X, 350.0, 330.0, 370.0);
    sc = box_label(sc, &enc, "Encoder", ENCODER, Tone::Context);
    sc = sc.node(centered_label(ENC_X, 535.0, "Inputs", 20.0, MUTED));

    let token_slots = row()
        .origin(Vec2::new(255.0, 570.0))
        .cell_size(Vec2::new(85.0, 44.0))
        .gap(20.0)
        .count(step.input.len());
    let vector_slots = row()
        .origin(Vec2::new(255.0, 425.0))
        .cell_size(Vec2::new(85.0, 52.0))
        .gap(20.0)
        .count(step.input.len());

    for i in 0..step.input.len() {
        sc = box_label(sc, &token_slots[i], step.input[i], PANEL, Tone::Active);
        let from = token_slots[i].center().resolve(0.0);
        let to = vector_slots[i].center().resolve(0.0);
        sc = sc.node(
            box_at(motion.ease(from, to), Vec2::new(64.0, 42.0))
                .radius(6.0)
                .fill(alpha(EMBEDDING, 0.84))
                .stroke(1.8, INK),
        );
        sc = vertical_arrow(sc, from.x, from.y - 26.0, to.y + 34.0, Tone::Context);
    }
    thin_label_box(
        sc,
        ENC_X,
        365.0,
        245.0,
        "Input Embedding",
        EMBEDDING,
        Tone::Active,
    )
}

fn positional_encoding_scene() -> Scene {
    let mut sc = base_scene("The model also needs word order.");
    let embed = slot_center(ENC_X, 430.0, 250.0, 54.0);
    let pos = slot_center(ENC_X, 315.0, 250.0, 54.0);
    sc = box_label(sc, &embed, "Input Embedding", EMBEDDING, Tone::Active);
    sc = plus(sc, ENC_X, 372.0, Tone::Active);
    sc = box_label(sc, &pos, "Positional Encoding", POSITION, Tone::Active);
    sc = vertical_arrow(sc, ENC_X, 403.0, 345.0, Tone::Active);
    vertical_arrow(sc, ENC_X, 287.0, 345.0, Tone::Active)
}

fn encoder_block_scene() -> Scene {
    let mut sc = base_scene("One encoder block lets tokens communicate, then refines each token.");
    sc = encoder_block(sc, ENC_X, 215.0, 312.0, Tone::Active, true, false);
    sc.node(centered_label(ENC_X, 178.0, "Encoder Block", 24.0, INK))
}

fn self_attention_scene() -> Scene {
    let mut sc = base_scene("Self-attention lets each token look at other tokens.");
    let slots = row()
        .origin(Vec2::new(390.0, 310.0))
        .cell_size(Vec2::new(120.0, 52.0))
        .gap(90.0)
        .count(3);
    for (i, token) in ["I", "like", "cats"].into_iter().enumerate() {
        sc = box_label(sc, &slots[i], token, ATTENTION, Tone::Active);
    }

    let centers: Vec<Vec2> = (0..3).map(|i| slots[i].center().resolve(0.0)).collect();
    sc = sc.node(
        connection(centers[0], centers[1])
            .via([
                Vec2::new(centers[0].x, 240.0),
                Vec2::new(centers[1].x, 240.0),
            ])
            .stroke(2.6, WIRE)
            .arrow(6.5),
    );
    sc = sc.node(
        connection(centers[1], centers[2])
            .via([
                Vec2::new(centers[1].x, 245.0),
                Vec2::new(centers[2].x, 245.0),
            ])
            .stroke(2.6, WIRE)
            .arrow(6.5),
    );
    sc.node(
        connection(centers[2], centers[0])
            .via([
                Vec2::new(centers[2].x, 430.0),
                Vec2::new(centers[0].x, 430.0),
            ])
            .stroke(2.6, WIRE)
            .arrow(6.5),
    )
}

fn multi_head_scene() -> Scene {
    let mut sc = base_scene("Multiple heads look at different relationships at the same time.");
    let input = slot_center(270.0, 310.0, 170.0, 58.0);
    let heads = column()
        .origin(Vec2::new(560.0, 205.0))
        .cell_size(Vec2::new(170.0, 50.0))
        .gap(36.0)
        .count(3);
    let combined = slot_center(1010.0, 310.0, 220.0, 58.0);

    sc = box_label(sc, &input, "Input", EMBEDDING, Tone::Active);
    for (i, label) in ["Head 1", "Head 2", "Head 3"].into_iter().enumerate() {
        sc = box_label(sc, &heads[i], label, ATTENTION, Tone::Active);
        sc = arrow(
            sc,
            input.right().resolve(0.0),
            heads[i].left().resolve(0.0),
            Tone::Active,
        );
        sc = arrow(
            sc,
            heads[i].right().resolve(0.0),
            combined.left().resolve(0.0),
            Tone::Context,
        );
    }
    box_label(sc, &combined, "Combined context", MEMORY, Tone::Active)
}

fn add_norm_scene(motion: TransformerArchitectureMotion) -> Scene {
    let mut sc = base_scene("The model keeps old information and adds the new update.");
    let x = slot_center(300.0, 255.0, 135.0, 50.0);
    let attn = slot_center(300.0, 405.0, 170.0, 50.0);
    let plus_slot = slot_center(645.0, 330.0, 54.0, 54.0);
    let norm = slot_center(880.0, 330.0, 190.0, 58.0);

    sc = box_label(sc, &x, "x", PANEL, Tone::Active);
    sc = box_label(sc, &attn, "attention(x)", ATTENTION, Tone::Active);
    sc = sc.node(
        circle()
            .x(645.0)
            .y(330.0)
            .radius(27.0)
            .fill(alpha(PANEL, 0.85)),
    );
    sc = sc.node(centered_text(&plus_slot, "+", 26.0, INK));
    sc = box_label(sc, &norm, "normalize", NORM, Tone::Active);

    sc = arrow(
        sc,
        attn.right().resolve(0.0),
        plus_slot.left().resolve(0.0),
        Tone::Active,
    );
    sc = arrow(
        sc,
        plus_slot.right().resolve(0.0),
        norm.left().resolve(0.0),
        Tone::Active,
    );
    sc = sc.node(
        connection(x.right().resolve(0.0), plus_slot.left().resolve(0.0))
            .via([Vec2::new(470.0, 255.0), Vec2::new(470.0, 330.0)])
            .stroke(motion.pulse(2.0, 4.0), RESIDUAL)
            .arrow(7.0),
    );
    sc
}

fn feed_forward_scene() -> Scene {
    let mut sc = base_scene("After sharing context, each token is processed on its own.");
    sc = sc.node(centered_label(
        380.0,
        170.0,
        "Attention = tokens talk to each other",
        22.0,
        ATTENTION,
    ));
    sc = sc.node(centered_label(
        870.0,
        170.0,
        "Feed Forward = each token is refined individually",
        22.0,
        FEED_FORWARD,
    ));

    for (i, (before, after)) in [("I*", "I'"), ("like*", "like'"), ("cats*", "cats'")]
        .into_iter()
        .enumerate()
    {
        let x = 430.0 + i as f32 * 210.0;
        let top = slot_center(x, 315.0, 110.0, 52.0);
        let bottom = slot_center(x, 455.0, 110.0, 52.0);
        sc = box_label(sc, &top, before, ATTENTION, Tone::Active);
        sc = vertical_arrow(sc, x, 345.0, 425.0, Tone::Active);
        sc = box_label(sc, &bottom, after, FEED_FORWARD, Tone::Active);
    }
    sc
}

fn encoder_repeats_scene() -> Scene {
    let mut sc = base_scene("The same block is repeated many times.");
    sc = encoder_stack_context(sc, Tone::Active, false);
    sc.node(centered_label(
        ENC_X,
        185.0,
        "Encoder Block × N",
        26.0,
        HIGHLIGHT,
    ))
}

fn encoder_memory_scene() -> Scene {
    let mut sc = base_scene("The encoder output becomes memory for the decoder.");
    sc = sc.node(centered_label(ENC_X, 575.0, "Input sentence", 21.0, INK));
    sc = vertical_arrow(sc, ENC_X, 545.0, 505.0, Tone::Active);
    sc = encoder_stack_context(sc, Tone::Active, false);
    sc = vertical_arrow(sc, ENC_X, 238.0, 182.0, Tone::Active);
    thin_label_box(
        sc,
        ENC_X,
        150.0,
        250.0,
        "Encoder Memory",
        MEMORY,
        Tone::Active,
    )
}

fn decoder_input_scene() -> Scene {
    let mut sc = base_scene("The decoder receives the output generated so far.");
    let outputs = slot_center(DEC_X, 500.0, 260.0, 48.0);
    let embed = slot_center(DEC_X, 390.0, 250.0, 54.0);
    let pos = slot_center(DEC_X, 275.0, 250.0, 54.0);
    sc = box_label(sc, &outputs, "Outputs shifted right", PANEL, Tone::Active);
    sc = vertical_arrow(sc, DEC_X, 475.0, 420.0, Tone::Active);
    sc = box_label(sc, &embed, "Output Embedding", EMBEDDING, Tone::Active);
    sc = plus(sc, DEC_X, 332.0, Tone::Active);
    sc = box_label(sc, &pos, "Positional Encoding", POSITION, Tone::Active);
    sc = vertical_arrow(sc, DEC_X, 362.0, 305.0, Tone::Active);
    vertical_arrow(sc, DEC_X, 247.0, 305.0, Tone::Active)
}

fn masked_self_attention_scene() -> Scene {
    let mut sc = base_scene("The decoder cannot look at future tokens.");
    let x0 = 465.0;
    let y0 = 180.0;
    let cell = 72.0;
    for i in 0..4 {
        sc = sc.node(centered_label(
            x0 + (i + 1) as f32 * cell,
            y0,
            format!("t{}", i + 1),
            18.0,
            MUTED,
        ));
        sc = sc.node(centered_label(
            x0,
            y0 + (i + 1) as f32 * cell,
            format!("t{}", i + 1),
            18.0,
            MUTED,
        ));
    }
    for r in 0..4 {
        for c in 0..4 {
            let cx = x0 + (c + 1) as f32 * cell;
            let cy = y0 + (r + 1) as f32 * cell;
            let allowed = c <= r;
            let slot = slot_center(cx, cy, 52.0, 44.0);
            sc = box_label(
                sc,
                &slot,
                if allowed { "✓" } else { "x" },
                if allowed { ATTENTION } else { PANEL },
                if allowed { Tone::Active } else { Tone::Context },
            );
        }
    }
    sc
}

fn cross_attention_scene() -> Scene {
    let mut sc = base_scene("The decoder looks back at the encoder memory.");
    let mem = slot_center(310.0, 330.0, 230.0, 70.0);
    let attn = slot_center(890.0, 330.0, 280.0, 70.0);
    sc = box_label(sc, &mem, "Encoder Memory", MEMORY, Tone::Active);
    sc = box_label(
        sc,
        &attn,
        "Decoder Multi-Head Attention",
        ATTENTION,
        Tone::Active,
    );
    arrow(
        sc,
        mem.right().resolve(0.0),
        attn.left().resolve(0.0),
        Tone::Active,
    )
}

fn decoder_block_scene() -> Scene {
    let mut sc = base_scene(
        "The decoder looks at previous output, then encoder memory, then refines the result.",
    );
    sc = decoder_block(sc, DEC_X, 145.0, 450.0, Tone::Active, true);
    sc.node(centered_label(DEC_X, 112.0, "Decoder Block", 24.0, INK))
}

fn linear_softmax_scene() -> Scene {
    let mut sc = base_scene("The final layers choose the next token.");
    let slots = column()
        .origin(Vec2::new(500.0, 110.0))
        .cell_size(Vec2::new(280.0, 54.0))
        .gap(32.0)
        .count(4);
    for (i, (label, color)) in [
        ("Decoder output", DECODER),
        ("Linear", OUTPUT),
        ("Softmax", OUTPUT),
        ("Output Probabilities", MEMORY),
    ]
    .into_iter()
    .enumerate()
    {
        sc = box_label(sc, &slots[i], label, color, Tone::Active);
        if i > 0 {
            sc = arrow(
                sc,
                slots[i - 1].bottom().resolve(0.0),
                slots[i].top().resolve(0.0),
                Tone::Active,
            );
        }
    }

    for (i, line) in ["猫が: 62%", "犬が: 12%", "私は: 8%"]
        .into_iter()
        .enumerate()
    {
        sc = sc.node(centered_label(
            910.0,
            310.0 + i as f32 * 38.0,
            line,
            22.0,
            if i == 0 { HIGHLIGHT } else { MUTED },
        ));
    }
    sc
}

fn full_architecture_scene(motion: TransformerArchitectureMotion) -> Scene {
    let mut sc = base_scene("Now the full diagram is just the pieces we introduced one by one.");
    sc = architecture_column(sc, ENC_X, true, motion, Tone::Active);
    sc = architecture_column(sc, DEC_X, false, motion, Tone::Active);
    sc = cross_memory_bridge(sc, Tone::Active);
    sc = sc.node(centered_label(ENC_X, 72.0, "Encoder", 25.0, ENCODER));
    sc = sc.node(centered_label(DEC_X, 72.0, "Decoder", 25.0, DECODER));
    sc
}

fn architecture_column(
    mut sc: Scene,
    cx: f32,
    encoder: bool,
    motion: TransformerArchitectureMotion,
    tone: Tone,
) -> Scene {
    if encoder {
        sc = sc.node(centered_label(cx, 680.0, "Inputs", 18.0, label_color(tone)));
        sc = vertical_arrow(sc, cx, 662.0, 640.0, tone);
        sc = thin_label_box(sc, cx, 612.0, STACK_W, "Input Embedding", EMBEDDING, tone);
        sc = plus(sc, cx, 570.0, tone);
        sc = sc.node(centered_label(
            cx - 138.0,
            579.0,
            "Positional Encoding",
            15.0,
            label_color(tone),
        ));
        sc = vertical_arrow(sc, cx, 550.0, 522.0, tone);
        sc = encoder_block(sc, cx, 260.0, 300.0, tone, true, true);
        sc = sc.node(centered_label(cx - 166.0, 420.0, "N×", 25.0, HIGHLIGHT));
        sc = vertical_arrow(sc, cx, 258.0, 122.0, tone);
    } else {
        sc = sc.node(centered_label(
            cx,
            680.0,
            "Outputs shifted right",
            17.0,
            label_color(tone),
        ));
        sc = vertical_arrow(sc, cx, 662.0, 640.0, tone);
        sc = thin_label_box(sc, cx, 612.0, STACK_W, "Output Embedding", EMBEDDING, tone);
        sc = plus(sc, cx, 570.0, tone);
        sc = sc.node(centered_label(
            cx + 142.0,
            579.0,
            "Positional Encoding",
            15.0,
            label_color(tone),
        ));
        sc = vertical_arrow(sc, cx, 550.0, 530.0, tone);
        sc = decoder_block(sc, cx, 160.0, 384.0, tone, true);
        sc = sc.node(centered_label(cx + 164.0, 360.0, "N×", 25.0, HIGHLIGHT));
        sc = vertical_arrow(sc, cx, 158.0, 130.0, tone);
        sc = thin_label_box(sc, cx, 105.0, STACK_W, "Linear", OUTPUT, tone);
        sc = vertical_arrow(sc, cx, 86.0, 74.0, tone);
        sc = thin_label_box(sc, cx, 50.0, STACK_W, "Softmax", OUTPUT, tone);
        sc = vertical_arrow(sc, cx, 31.0, 23.0, tone);
        sc = sc.node(centered_label(
            cx,
            18.0,
            "Output Probabilities",
            16.0,
            label_color(tone),
        ));
    }

    sc = sc.node(
        circle()
            .x(cx)
            .y(if encoder { 570.0 } else { 570.0 })
            .radius(motion.pulse(7.0, 10.0))
            .fill(alpha(POSITION, 0.16)),
    );
    sc
}

fn encoder_stack_context(sc: Scene, tone: Tone, compact: bool) -> Scene {
    let top = if compact { 260.0 } else { 230.0 };
    let h = if compact { 300.0 } else { 315.0 };
    encoder_block(sc, ENC_X, top, h, tone, true, true)
}

fn encoder_block(
    mut sc: Scene,
    cx: f32,
    top: f32,
    height: f32,
    tone: Tone,
    residuals: bool,
    repeated: bool,
) -> Scene {
    let cont = Slot::new(cx - STACK_W / 2.0 - 20.0, top, STACK_W + 40.0, height);
    sc = sc.node(
        box_in(&cont).radius(12.0).style(
            Style::new()
                .fill(alpha(PANEL, if repeated { 0.72 } else { 0.45 }))
                .stroke(
                    2.0,
                    alpha(
                        INK,
                        if matches!(tone, Tone::Active) {
                            0.65
                        } else {
                            0.22
                        },
                    ),
                ),
        ),
    );
    sc = sc.node(centered_label(
        cx,
        top + 22.0,
        "Encoder Block × N",
        17.0,
        HIGHLIGHT,
    ));

    let mha = slot_center(cx, top + height - 78.0, STACK_W, LAYER_H);
    let an1 = slot_center(cx, top + height - 126.0, STACK_W, NORM_H);
    let ffn = slot_center(cx, top + height - 198.0, STACK_W, LAYER_H);
    let an2 = slot_center(cx, top + height - 246.0, STACK_W, NORM_H);

    sc = box_label(sc, &mha, "Multi-Head Attention", ATTENTION, tone);
    sc = box_label(sc, &an1, "Add & Norm", NORM, tone);
    sc = box_label(sc, &ffn, "Feed Forward", FEED_FORWARD, tone);
    sc = box_label(sc, &an2, "Add & Norm", NORM, tone);
    sc = vertical_arrow(
        sc,
        cx,
        mha.top().resolve(0.0).y,
        an1.bottom().resolve(0.0).y,
        tone,
    );
    sc = vertical_arrow(
        sc,
        cx,
        an1.top().resolve(0.0).y,
        ffn.bottom().resolve(0.0).y,
        tone,
    );
    sc = vertical_arrow(
        sc,
        cx,
        ffn.top().resolve(0.0).y,
        an2.bottom().resolve(0.0).y,
        tone,
    );

    if residuals {
        sc = residual(sc, cx, &mha, &an1, true, tone);
        sc = residual(sc, cx, &ffn, &an2, false, tone);
    }
    sc
}

fn decoder_block(
    mut sc: Scene,
    cx: f32,
    top: f32,
    height: f32,
    tone: Tone,
    residuals: bool,
) -> Scene {
    let cont = Slot::new(cx - STACK_W / 2.0 - 20.0, top, STACK_W + 40.0, height);
    sc = sc.node(
        box_in(&cont)
            .radius(12.0)
            .style(Style::new().fill(alpha(PANEL, 0.60)).stroke(
                2.0,
                alpha(
                    INK,
                    if matches!(tone, Tone::Active) {
                        0.65
                    } else {
                        0.22
                    },
                ),
            )),
    );
    sc = sc.node(centered_label(
        cx,
        top + 22.0,
        "Decoder Block × N",
        17.0,
        HIGHLIGHT,
    ));

    let masked = slot_center(cx, top + height - 58.0, STACK_W, LAYER_H);
    let an1 = slot_center(cx, top + height - 104.0, STACK_W, NORM_H);
    let cross = slot_center(cx, top + height - 168.0, STACK_W, LAYER_H);
    let an2 = slot_center(cx, top + height - 214.0, STACK_W, NORM_H);
    let ffn = slot_center(cx, top + height - 278.0, STACK_W, LAYER_H);
    let an3 = slot_center(cx, top + height - 324.0, STACK_W, NORM_H);

    sc = box_label(sc, &masked, "Masked Multi-Head Attention", ATTENTION, tone);
    sc = box_label(sc, &an1, "Add & Norm", NORM, tone);
    sc = box_label(sc, &cross, "Multi-Head Attention", ATTENTION, tone);
    sc = box_label(sc, &an2, "Add & Norm", NORM, tone);
    sc = box_label(sc, &ffn, "Feed Forward", FEED_FORWARD, tone);
    sc = box_label(sc, &an3, "Add & Norm", NORM, tone);

    for (from, to) in [
        (&masked, &an1),
        (&an1, &cross),
        (&cross, &an2),
        (&an2, &ffn),
        (&ffn, &an3),
    ] {
        sc = vertical_arrow(
            sc,
            cx,
            from.top().resolve(0.0).y,
            to.bottom().resolve(0.0).y,
            tone,
        );
    }

    if residuals {
        sc = residual(sc, cx, &masked, &an1, true, tone);
        sc = residual(sc, cx, &cross, &an2, false, tone);
        sc = residual(sc, cx, &ffn, &an3, true, tone);
    }
    sc
}

fn residual(sc: Scene, cx: f32, from: &Slot, to: &Slot, left: bool, tone: Tone) -> Scene {
    let side = if left { -1.0 } else { 1.0 };
    let offset = STACK_W / 2.0 + 34.0;
    let start = Vec2::new(cx, from.bottom().resolve(0.0).y + 3.0);
    let mid = Vec2::new(cx + side * offset, start.y);
    let end = Vec2::new(cx + side * STACK_W / 2.0, to.center().resolve(0.0).y);
    let color = match tone {
        Tone::Active => RESIDUAL,
        Tone::Context => CONTEXT_WIRE,
    };
    sc.node(
        connection(start, end)
            .via([mid, Vec2::new(mid.x, end.y)])
            .stroke(
                if matches!(tone, Tone::Active) {
                    2.1
                } else {
                    1.2
                },
                color,
            )
            .arrow(if matches!(tone, Tone::Active) {
                5.4
            } else {
                3.5
            }),
    )
}

fn cross_memory_bridge(sc: Scene, tone: Tone) -> Scene {
    elbow_arrow(
        sc,
        Vec2::new(ENC_X + STACK_W / 2.0 + 20.0, 280.0),
        Vec2::new(650.0, 280.0),
        Vec2::new(DEC_X - STACK_W / 2.0 - 4.0, 376.0),
        tone,
    )
}
