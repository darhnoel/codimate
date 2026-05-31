use codimate_core::{circle, scene, Color};
use codimate_layout::{layout_scene, Viewport};
use codimate_render::{render_commands, render_frame, RenderCommand};

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

#[test]
fn render_frame_carries_metadata_viewport_and_commands() {
    let concrete = scene()
        .node(circle().x(10.0).y(20.0).radius(5.0).fill(Color::RED))
        .resolve(0.0);
    let layout = layout_scene(concrete, Viewport::new(800.0, 600.0));

    let frame = render_frame("demo", 0.5, &layout);

    assert_eq!(frame.name, "demo");
    assert_eq!(frame.elapsed_seconds, 0.5);
    assert_eq!(frame.viewport, Viewport::new(800.0, 600.0));
    assert_eq!(frame.commands.len(), 1);
}
