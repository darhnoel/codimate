use codimate_arrange::{columns, ColumnDef, EdgeKind, LayerDef, Layout};
use codimate_core::manim;

pub const CENC: f32 = 280.0;
pub const CDEC: f32 = 720.0;
pub const BW: f32 = 220.0;
pub const BHW: f32 = BW / 2.0;

pub const ENC_LAYER_IDS: [&str; 5] = ["input_embedding", "mha", "add_norm_1", "ffn", "add_norm_2"];

pub const DEC_LAYER_IDS: [&str; 9] = [
    "output_embedding",
    "masked_mha",
    "add_norm_4",
    "mha_dec",
    "add_norm_5",
    "ffn_dec",
    "add_norm_6",
    "linear",
    "softmax",
];

pub fn transformer_layout() -> Layout {
    columns()
        .box_width(BW)
        .default_gap(30.0)
        .clearance(24.0)
        .arrow_gap(7.0)
        .column(
            ColumnDef::new("encoder", CENC)
                .anchor_y(328.0)
                .container_padding(42.0)
                .layer(LayerDef::new("add_norm_2", "Add & Norm", 28.0).color(manim::YELLOW))
                .layer(LayerDef::new("ffn", "Feed Forward", 52.0).color(manim::BLUE))
                .layer(LayerDef::new("add_norm_1", "Add & Norm", 28.0).color(manim::YELLOW))
                .layer(
                    LayerDef::new("mha", "Multi-Head Attention", 52.0)
                        .color(manim::ORANGE)
                        .qkv_arrows(true)
                        .gap_below(46.0),
                )
                .layer(
                    LayerDef::new("input_embedding", "Input Embedding", 40.0).color(manim::RED_E),
                ),
        )
        .column(
            ColumnDef::new("decoder", CDEC)
                .anchor_y(60.0)
                .container_padding(42.0)
                .layer(LayerDef::new("softmax", "Softmax", 34.0).color(manim::GREEN_E))
                .layer(LayerDef::new("linear", "Linear", 34.0).color(manim::PURPLE))
                .layer(LayerDef::new("add_norm_6", "Add & Norm", 28.0).color(manim::YELLOW))
                .layer(LayerDef::new("ffn_dec", "Feed Forward", 52.0).color(manim::BLUE))
                .layer(LayerDef::new("add_norm_5", "Add & Norm", 28.0).color(manim::YELLOW))
                .layer(
                    LayerDef::new("mha_dec", "Multi-Head Attention", 52.0)
                        .color(manim::ORANGE)
                        .qkv_arrows(true),
                )
                .layer(LayerDef::new("add_norm_4", "Add & Norm", 28.0).color(manim::YELLOW))
                .layer(
                    LayerDef::new("masked_mha", "Masked Multi-Head Attn", 52.0)
                        .color(manim::ORANGE)
                        .qkv_arrows(true)
                        .gap_below(46.0),
                )
                .layer(
                    LayerDef::new("output_embedding", "Output Embedding", 40.0).color(manim::RED_E),
                ),
        )
        .edge(
            "encoder",
            "input_embedding",
            "encoder",
            "mha",
            EdgeKind::Vertical,
        )
        .edge(
            "encoder",
            "mha",
            "encoder",
            "add_norm_1",
            EdgeKind::Vertical,
        )
        .edge(
            "encoder",
            "add_norm_1",
            "encoder",
            "ffn",
            EdgeKind::Vertical,
        )
        .edge(
            "encoder",
            "ffn",
            "encoder",
            "add_norm_2",
            EdgeKind::Vertical,
        )
        .edge(
            "encoder",
            "input_embedding",
            "encoder",
            "add_norm_1",
            EdgeKind::ResidualLeft,
        )
        .edge(
            "encoder",
            "add_norm_1",
            "encoder",
            "add_norm_2",
            EdgeKind::ResidualRight,
        )
        .edge(
            "decoder",
            "output_embedding",
            "decoder",
            "masked_mha",
            EdgeKind::Vertical,
        )
        .edge(
            "decoder",
            "masked_mha",
            "decoder",
            "add_norm_4",
            EdgeKind::Vertical,
        )
        .edge(
            "decoder",
            "add_norm_4",
            "decoder",
            "mha_dec",
            EdgeKind::Vertical,
        )
        .edge(
            "decoder",
            "mha_dec",
            "decoder",
            "add_norm_5",
            EdgeKind::Vertical,
        )
        .edge(
            "decoder",
            "add_norm_5",
            "decoder",
            "ffn_dec",
            EdgeKind::Vertical,
        )
        .edge(
            "decoder",
            "ffn_dec",
            "decoder",
            "add_norm_6",
            EdgeKind::Vertical,
        )
        .edge(
            "decoder",
            "add_norm_6",
            "decoder",
            "linear",
            EdgeKind::Vertical,
        )
        .edge(
            "decoder",
            "linear",
            "decoder",
            "softmax",
            EdgeKind::Vertical,
        )
        .edge(
            "decoder",
            "output_embedding",
            "decoder",
            "add_norm_4",
            EdgeKind::ResidualLeft,
        )
        .edge(
            "decoder",
            "add_norm_4",
            "decoder",
            "add_norm_5",
            EdgeKind::ResidualRight,
        )
        .edge(
            "decoder",
            "add_norm_5",
            "decoder",
            "add_norm_6",
            EdgeKind::ResidualLeft,
        )
        .edge(
            "encoder",
            "add_norm_2",
            "decoder",
            "mha_dec",
            EdgeKind::Cross,
        )
        .build()
        .expect("transformer layout definition is valid")
}
