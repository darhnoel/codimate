use codimate_animation::{animation, parallel};
use codimate_core::*;
use codimate_layout::Viewport;
use codimate_previewer::{PreviewConfig, Previewer};

mod style;

use style::*;

fn main() {
    let ball = animation(
        "ball",
        2.0,
        scene().node(
            circle()
                .x(tween(100.0, 700.0))
                .y(300.0)
                .radius(32.0)
                .fill(RED),
        ),
    );

    let box_grow = animation(
        "box-grow",
        2.0,
        scene().node(
            rect()
                .x(300.0)
                .y(100.0)
                .width(tween(100.0, 400.0))
                .height(200.0)
                .fill(BLUE),
        ),
    );

    let morph = animation(
        "morph",
        2.0,
        scene().node(
            path_node()
                .path(tween(
                    circle_path(600.0, 400.0, 40.0),
                    rect_path(560.0, 360.0, 80.0, 80.0),
                ))
                .fill(GREEN),
        ),
    );

    let demo = parallel("demo", [ball, box_grow, morph]);
    let viewport = Viewport::new(800.0, 600.0);
    let config = PreviewConfig::new(30.0, viewport);

    let previewer = Previewer::new(Box::new(demo), config);
    previewer.run().unwrap();
}
