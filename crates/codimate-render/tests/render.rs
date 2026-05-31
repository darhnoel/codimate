use codimate_core::{circle, scene, Color};
use codimate_layout::{layout_scene, Viewport};
use codimate_render::{render_commands, RenderCommand};

#[test]
fn render_commands_project_concrete_nodes() {
    let concrete = scene()
        .node(circle().x(10.0).y(20.0).radius(5.0).fill(Color::RED))
        .resolve(0.0);
    let frame = layout_scene(concrete, Viewport::new(800.0, 600.0));

    assert_eq!(
        render_commands(&frame),
        vec![RenderCommand::Circle {
            x: 10.0,
            y: 20.0,
            radius: 5.0,
            fill: Color::RED,
        }]
    );
}
