use codimate_arrange::{columns, ColumnDef, EdgeKind, LayerDef, Layout};
use codimate_core::{manim, Vec2};

fn transformer_layout() -> Layout {
    columns()
        .box_width(220.0)
        .default_gap(30.0)
        .clearance(24.0)
        .arrow_gap(7.0)
        .column(
            ColumnDef::new("encoder", 280.0)
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
            ColumnDef::new("decoder", 720.0)
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
        .expect("valid transformer layout")
}

#[test]
fn columns_arrangement_matches_transformer_generated_arrays() {
    let layout = transformer_layout();

    assert_eq!(layout["encoder"].center_x(), 280.0);
    assert_eq!(layout["encoder"].ey(), &[624.0, 526.0, 468.0, 386.0, 328.0]);
    assert_eq!(layout["encoder"].h(), &[40.0, 52.0, 28.0, 52.0, 28.0]);
    assert_eq!(layout["encoder"].container().top, 286.0);
    assert_eq!(layout["encoder"].container().bottom, 706.0);

    assert_eq!(layout["decoder"].center_x(), 720.0);
    assert_eq!(
        layout["decoder"].ey(),
        &[624.0, 526.0, 468.0, 386.0, 328.0, 246.0, 188.0, 124.0, 60.0]
    );
    assert_eq!(
        layout["decoder"].h(),
        &[40.0, 52.0, 28.0, 52.0, 28.0, 52.0, 28.0, 34.0, 34.0]
    );
    assert_eq!(layout["decoder"].container().top, 18.0);
    assert_eq!(layout["decoder"].container().bottom, 706.0);
}

#[test]
fn columns_arrangement_matches_transformer_generated_routes() {
    let layout = transformer_layout();

    assert_eq!(
        layout
            .route("encoder", "input_embedding", "encoder", "mha")
            .unwrap()
            .points,
        vec![Vec2::new(280.0, 620.0), Vec2::new(280.0, 582.0)]
    );
    assert_eq!(
        layout
            .route("encoder", "input_embedding", "encoder", "add_norm_1")
            .unwrap()
            .points,
        vec![
            Vec2::new(280.0, 668.0),
            Vec2::new(146.0, 668.0),
            Vec2::new(146.0, 482.0),
            Vec2::new(163.0, 482.0),
        ]
    );
    assert_eq!(
        layout
            .route("encoder", "add_norm_1", "encoder", "add_norm_2")
            .unwrap()
            .points,
        vec![
            Vec2::new(280.0, 500.0),
            Vec2::new(414.0, 500.0),
            Vec2::new(414.0, 342.0),
            Vec2::new(397.0, 342.0),
        ]
    );
    assert_eq!(
        layout
            .route("encoder", "add_norm_2", "decoder", "mha_dec")
            .unwrap()
            .points,
        vec![
            Vec2::new(390.0, 342.0),
            Vec2::new(474.0, 342.0),
            Vec2::new(474.0, 412.0),
            Vec2::new(592.0, 412.0),
            Vec2::new(610.0, 412.0),
        ]
    );
}
