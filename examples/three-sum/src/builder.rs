use crate::{
    view::build_three_sum, ThreeSum, ThreeSumMotion, ThreeSumTiming, ThreeSumTrace, ThreeSumView,
};
use codimate_animation::Playable;
use codimate_export::{export_mp4, ExportConfig};
use codimate_layout::Viewport;
use std::path::Path;

pub struct ExplainBuilder {
    name: &'static str,
    state: Option<ThreeSum>,
    algorithm: Option<fn(ThreeSum) -> ThreeSumTrace>,
    motion: Option<ThreeSumMotion>,
    timing: ThreeSumTiming,
}

pub fn explain(name: &'static str) -> ExplainBuilder {
    ExplainBuilder {
        name,
        state: None,
        algorithm: None,
        motion: None,
        timing: ThreeSumTiming::default(),
    }
}

impl ExplainBuilder {
    pub fn state(mut self, state: ThreeSum) -> Self {
        self.state = Some(state);
        self
    }

    pub fn view(self, view: fn() -> ThreeSumView) -> Self {
        let _ = view();
        self
    }

    pub fn algorithm(mut self, algorithm: fn(ThreeSum) -> ThreeSumTrace) -> Self {
        self.algorithm = Some(algorithm);
        self
    }

    pub fn motion(mut self, motion: fn() -> ThreeSumMotion) -> Self {
        self.motion = Some(motion());
        self
    }

    pub fn timing(mut self, timing: ThreeSumTiming) -> Self {
        self.timing = timing;
        self
    }

    pub fn build(self) -> (Box<dyn Playable>, Viewport) {
        let state = self.state.expect("3Sum state must be provided");
        let algorithm = self.algorithm.expect("3Sum algorithm must be provided");
        build_three_sum(
            self.name,
            algorithm(state),
            self.motion.expect("3Sum motion must be provided"),
            self.timing,
        )
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
