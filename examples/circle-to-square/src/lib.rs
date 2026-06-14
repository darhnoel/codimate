use codimate::{
    animation, circle_path, ease_in_out, export_mp4, manim, primitive_path, rect_path, scene,
    sequence, tween, Animated, Animation, Color, ExplanationBuilder, ExportConfig, Path, Playable,
    Scene, Style, Transformable, Viewport,
};

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

            scene().add(primitive_path(motion.circle_to_square()).style(outline))
        }
        ShapeStep::MorphSquareToCircle => {
            let outline = Style::new()
                .fill(Color::TRANSPARENT)
                .stroke(6.0, manim::GREEN);

            scene().add(primitive_path(motion.square_to_circle()).style(outline))
        }
    }
}

type Inner =
    ExplanationBuilder<ShapeDemo, fn(ShapeDemo) -> Vec<ShapeStep>, ShapeMotion, ShapeTiming>;

pub struct ExplainBuilder(Inner);

pub fn explain(name: &'static str) -> ExplainBuilder {
    ExplainBuilder(Inner::new(name))
}

impl ExplainBuilder {
    pub fn state(self, state: ShapeDemo) -> Self {
        Self(self.0.state(state))
    }

    pub fn algorithm(self, algorithm: fn(ShapeDemo) -> Vec<ShapeStep>) -> Self {
        Self(self.0.algorithm(algorithm))
    }

    pub fn motion(self, motion: fn() -> ShapeMotion) -> Self {
        Self(self.0.motion(motion()))
    }

    pub fn timing(self, timing: ShapeTiming) -> Self {
        Self(self.0.timing(timing))
    }

    pub fn view<V>(self, view: fn() -> V) -> Self {
        Self(self.0.view(view))
    }

    pub fn build(self) -> (Box<dyn Playable>, Viewport) {
        let name = self.0.name;
        let (state, algorithm, motion, timing) = self
            .0
            .take()
            .expect("circle-to-square: state, algorithm, motion required");
        build_circle_to_square(name, algorithm(state), motion, timing)
    }

    pub fn render(self, output: impl AsRef<std::path::Path>) {
        self.render_with(output, |viewport| ExportConfig::new(30.0, viewport).crf(12));
    }

    pub fn render_with(
        self,
        output: impl AsRef<std::path::Path>,
        export_config: impl FnOnce(Viewport) -> ExportConfig,
    ) {
        let output = output.as_ref();
        if let Some(parent) = output.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).ok();
            }
        }

        let (play, viewport) = self.build();
        let cfg = export_config(viewport);
        println!("Exporting {} ...", output.display());
        match export_mp4(&play, &cfg, output) {
            Ok(()) => println!("Written {}", output.display()),
            Err(e) => eprintln!("mp4 export skipped: {e}"),
        }
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
