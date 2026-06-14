use crate::{view::build_knapsack, Knapsack, KnapsackMotion, KnapsackTiming, KnapsackTrace};
use codimate::ExplanationBuilder;
use codimate::Playable;
use codimate::Viewport;
use codimate::{export_mp4, ExportConfig};
use std::path::Path;

type Inner =
    ExplanationBuilder<Knapsack, fn(Knapsack) -> KnapsackTrace, KnapsackMotion, KnapsackTiming>;

pub struct ExplainBuilder(Inner);

pub fn explain(name: &'static str) -> ExplainBuilder {
    ExplainBuilder(Inner::new(name))
}

impl ExplainBuilder {
    pub fn state(self, state: Knapsack) -> Self {
        Self(self.0.state(state))
    }

    pub fn algorithm(self, algorithm: fn(Knapsack) -> KnapsackTrace) -> Self {
        Self(self.0.algorithm(algorithm))
    }

    pub fn motion(self, motion: fn() -> KnapsackMotion) -> Self {
        Self(self.0.motion(motion()))
    }

    pub fn timing(self, timing: KnapsackTiming) -> Self {
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
            .expect("knapsack: state, algorithm, motion required");
        build_knapsack(name, state, algorithm(state), motion, timing)
    }

    pub fn render(self, output: impl AsRef<Path>) {
        self.render_with(output, |viewport| ExportConfig::new(30.0, viewport).crf(12));
    }

    pub fn render_with(
        self,
        output: impl AsRef<Path>,
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
