use crate::{view::build_demo, Demo, DemoMotion, DemoTiming, DemoTrace, DemoView};
use codimate_animation::Playable;
use codimate_export::{export_mp4, ExportConfig};
use codimate_layout::Viewport;
use std::path::Path;

pub struct ExplainBuilder {
    name: &'static str,
    state: Option<Demo>,
    trace: Option<DemoTrace>,
    motion: Option<DemoMotion>,
    timing: DemoTiming,
}

pub fn explain(name: &'static str) -> ExplainBuilder {
    ExplainBuilder {
        name,
        state: None,
        trace: None,
        motion: None,
        timing: DemoTiming::default(),
    }
}

impl ExplainBuilder {
    pub fn state(mut self, state: Demo) -> Self {
        self.state = Some(state);
        self
    }

    pub fn view(self, view: fn() -> DemoView) -> Self {
        let _ = view();
        self
    }

    pub fn algorithm(mut self, algorithm: fn(Demo) -> DemoTrace) -> Self {
        let state = self
            .state
            .expect("demo state must be provided before algorithm");
        self.trace = Some(algorithm(state));
        self.state = Some(state);
        self
    }

    pub fn motion(mut self, motion: fn() -> DemoMotion) -> Self {
        self.motion = Some(motion());
        self
    }

    pub fn timing(mut self, timing: DemoTiming) -> Self {
        self.timing = timing;
        self
    }

    pub fn build(self) -> (Box<dyn Playable>, Viewport) {
        build_demo(
            self.name,
            self.state.expect("demo state must be provided"),
            self.trace.expect("demo algorithm must be provided"),
            self.motion.expect("demo motion must be provided"),
            self.timing,
        )
    }

    pub fn render(self, output: impl AsRef<Path>) {
        let output = output.as_ref();
        if let Some(parent) = output.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).ok();
            }
        }

        let (play, viewport) = self.build();
        let cfg = ExportConfig::new(30.0, viewport).crf(12);
        println!("Exporting {} ...", output.display());
        match export_mp4(&play, &cfg, output) {
            Ok(()) => println!("Written {}", output.display()),
            Err(e) => eprintln!("mp4 export skipped: {e}"),
        }
    }
}
