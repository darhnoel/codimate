use crate::{view::build_demo, Demo, DemoMotion, DemoTiming, DemoTrace};
use codimate_animation::Playable;
use codimate_core::ExplanationBuilder;
use codimate_export::{export_mp4, ExportConfig};
use codimate_layout::Viewport;
use std::path::Path;

type Inner = ExplanationBuilder<Demo, DemoTrace, DemoMotion, DemoTiming>;

pub struct ExplainBuilder(Inner);

pub fn explain(name: &'static str) -> ExplainBuilder {
    ExplainBuilder(Inner::new(name))
}

impl ExplainBuilder {
    pub fn state(self, state: Demo) -> Self {
        Self(self.0.state(state))
    }

    pub fn algorithm(mut self, algorithm: fn(Demo) -> DemoTrace) -> Self {
        let state = self
            .0
            .state
            .take()
            .expect("demo: state needed before algorithm");
        let trace = algorithm(state);
        self.0.state = Some(state);
        Self(self.0.algorithm(trace))
    }

    pub fn motion(self, motion: fn() -> DemoMotion) -> Self {
        Self(self.0.motion(motion()))
    }

    pub fn timing(self, timing: DemoTiming) -> Self {
        Self(self.0.timing(timing))
    }

    pub fn view<V>(self, view: fn() -> V) -> Self {
        Self(self.0.view(view))
    }

    pub fn build(self) -> (Box<dyn Playable>, Viewport) {
        let name = self.0.name;
        let (state, trace, motion, timing) =
            self.0.take().expect("demo: state, algorithm, motion required");
        build_demo(name, state, trace, motion, timing)
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
