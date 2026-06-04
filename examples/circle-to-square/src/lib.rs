use codimate_animation::{animation, sequence, Animation, Playable};
use codimate_core::{
    circle_path, ease_in_out, manim, path_node, rect_path, scene, tween, Animated, Color, Path,
    Scene, Style,
};
use codimate_layout::Viewport;

#[derive(Clone, Copy)]
pub struct ShapeDemo;

#[derive(Clone, Copy)]
pub enum ShapeStep {
    MorphCircleToSquare,
    MorphSquareToCircle,
}

impl ShapeStep {
    fn name(self) -> &'static str {
        match self {
            ShapeStep::MorphCircleToSquare => "circle-to-square",
            ShapeStep::MorphSquareToCircle => "square-to-circle",
        }
    }

    fn duration(self, timing: ShapeTiming) -> f32 {
        match self {
            ShapeStep::MorphCircleToSquare => timing.circle_to_square,
            ShapeStep::MorphSquareToCircle => timing.square_to_circle,
        }
    }
}

pub fn circle_to_square_algorithm(_state: ShapeDemo) -> Vec<ShapeStep> {
    vec![
        ShapeStep::MorphCircleToSquare,
        ShapeStep::MorphSquareToCircle,
    ]
}

#[derive(Clone, Copy)]
pub struct ShapeMotion;

pub fn circle_to_square_motion() -> ShapeMotion {
    ShapeMotion
}

impl ShapeMotion {
    fn circle(self) -> Path {
        circle_path(240.0, 300.0, 80.0)
    }

    fn square(self) -> Path {
        rect_path(480.0, 220.0, 160.0, 160.0)
    }

    fn circle_to_square(self) -> Animated<Path> {
        tween(self.circle(), self.square()).ease(ease_in_out)
    }

    fn square_to_circle(self) -> Animated<Path> {
        tween(self.square(), self.circle()).ease(ease_in_out)
    }
}

#[derive(Clone, Copy)]
pub struct ShapeTiming {
    pub circle_to_square: f32,
    pub square_to_circle: f32,
}

impl Default for ShapeTiming {
    fn default() -> Self {
        Self {
            circle_to_square: 1.4,
            square_to_circle: 1.4,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ShapeView;

pub fn circle_to_square_view() -> ShapeView {
    ShapeView
}

fn step_scene(step: ShapeStep, motion: ShapeMotion) -> Scene {
    match step {
        ShapeStep::MorphCircleToSquare => {
            let outline = Style::new()
                .fill(Color::TRANSPARENT)
                .stroke(6.0, manim::BLUE);

            scene().node(path_node().path(motion.circle_to_square()).style(outline))
        }
        ShapeStep::MorphSquareToCircle => {
            let outline = Style::new()
                .fill(Color::TRANSPARENT)
                .stroke(6.0, manim::GREEN);

            scene().node(path_node().path(motion.square_to_circle()).style(outline))
        }
    }
}

pub struct ExplainBuilder {
    name: &'static str,
    state: Option<ShapeDemo>,
    algorithm: Option<fn(ShapeDemo) -> Vec<ShapeStep>>,
    motion: Option<ShapeMotion>,
    timing: ShapeTiming,
}

pub fn explain(name: &'static str) -> ExplainBuilder {
    ExplainBuilder {
        name,
        state: None,
        algorithm: None,
        motion: None,
        timing: ShapeTiming::default(),
    }
}

impl ExplainBuilder {
    pub fn state(mut self, state: ShapeDemo) -> Self {
        self.state = Some(state);
        self
    }

    pub fn view(self, view: fn() -> ShapeView) -> Self {
        let _ = view();
        self
    }

    pub fn algorithm(mut self, algorithm: fn(ShapeDemo) -> Vec<ShapeStep>) -> Self {
        self.algorithm = Some(algorithm);
        self
    }

    pub fn motion(mut self, motion: fn() -> ShapeMotion) -> Self {
        self.motion = Some(motion());
        self
    }

    pub fn timing(mut self, timing: ShapeTiming) -> Self {
        self.timing = timing;
        self
    }

    pub fn build(self) -> (Box<dyn Playable>, Viewport) {
        let state = self.state.expect("shape demo state must be provided");
        let algorithm = self
            .algorithm
            .expect("shape demo algorithm must be provided");
        build_circle_to_square(
            self.name,
            algorithm(state),
            self.motion.expect("shape demo motion must be provided"),
            self.timing,
        )
    }
}

fn build_circle_to_square(
    name: &'static str,
    trace: Vec<ShapeStep>,
    motion: ShapeMotion,
    timing: ShapeTiming,
) -> (Box<dyn Playable>, Viewport) {
    let animations = trace
        .into_iter()
        .map(|step| animation(step.name(), step.duration(timing), step_scene(step, motion)))
        .collect::<Vec<Animation>>();

    (
        Box::new(sequence(name, animations)),
        Viewport::new(800.0, 600.0),
    )
}

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("Circle to Square")
        .state(ShapeDemo)
        .view(circle_to_square_view)
        .algorithm(circle_to_square_algorithm)
        .motion(circle_to_square_motion)
        .timing(ShapeTiming::default())
        .build()
}
