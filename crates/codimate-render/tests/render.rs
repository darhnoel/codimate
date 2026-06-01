use codimate_core::{circle, connection, path_node, pulse_on, rect_path, scene, AnchorKind, Color};
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

#[test]
fn render_commands_path_node_with_stroke_produces_stroke_in_command() {
    let concrete = scene()
        .node(
            path_node()
                .path(rect_path(10.0, 10.0, 50.0, 30.0))
                .fill(Color::RED)
                .stroke(4.0, Color::WHITE),
        )
        .resolve(0.0);
    let frame = layout_scene(concrete, Viewport::new(100.0, 100.0));

    let cmds = render_commands(&frame);
    assert_eq!(cmds.len(), 1);
    match &cmds[0] {
        RenderCommand::Path {
            segments,
            closed,
            fill,
            stroke_width,
            stroke_color,
        } => {
            assert_eq!(segments.len(), 4);
            assert!(*closed);
            assert_eq!(*fill, Color::RED);
            assert_eq!(*stroke_width, 4.0);
            assert_eq!(*stroke_color, Color::WHITE);
        }
        other => panic!("expected Path command, got {other:?}"),
    }
}

#[test]
fn render_commands_connection_produces_path_command() {
    let a = circle().x(50.0).y(100.0).radius(20.0);
    let b = circle().x(200.0).y(100.0).radius(20.0);

    let concrete = scene()
        .node(a.clone())
        .node(b.clone())
        .node(
            connection(a.anchor(AnchorKind::Right), b.anchor(AnchorKind::Left))
                .stroke(2.0, Color::RED),
        )
        .resolve(0.0);
    let frame = layout_scene(concrete, Viewport::new(300.0, 200.0));

    let cmds = render_commands(&frame);
    assert_eq!(cmds.len(), 3);
    match &cmds[2] {
        RenderCommand::Path {
            segments,
            closed,
            fill,
            stroke_width,
            stroke_color,
        } => {
            assert_eq!(segments.len(), 1);
            assert!(!*closed);
            assert_eq!(*fill, Color::TRANSPARENT);
            assert_eq!(*stroke_width, 2.0);
            assert_eq!(*stroke_color, Color::RED);
        }
        other => panic!("expected Path for connection, got {other:?}"),
    }
}

#[test]
fn render_commands_pulse_produces_circle_command() {
    let a = circle().x(50.0).y(100.0).radius(20.0);
    let b = circle().x(200.0).y(100.0).radius(20.0);
    let conn = connection(a.anchor(AnchorKind::Right), b.anchor(AnchorKind::Left));

    let concrete = scene()
        .node(a.clone())
        .node(b.clone())
        .node(conn.clone())
        .node(pulse_on(conn, 0.5).radius(5.0).fill(Color::CYAN))
        .resolve(0.0);
    let frame = layout_scene(concrete, Viewport::new(300.0, 200.0));

    let cmds = render_commands(&frame);
    assert_eq!(cmds.len(), 4);
    match &cmds[3] {
        RenderCommand::Circle { x, y, radius, fill } => {
            assert!((*x - 125.0).abs() < 0.1, "pulse x={x} (expected ~125)");
            assert!((*y - 100.0).abs() < 0.1, "pulse y={y} (expected 100)");
            assert_eq!(*radius, 5.0);
            assert_eq!(*fill, Color::CYAN);
        }
        other => panic!("expected Circle for pulse, got {other:?}"),
    }
}
