use crate::{
    view::build_connection_pulse, ConnectionPulse, ConnectionPulseMotion, ConnectionPulseTiming,
    ConnectionPulseTrace,
};
use codimate_animation::Playable;
use codimate_core::ExplanationBuilder;
use codimate_export::{export_mp4, ExportConfig};
use codimate_layout::Viewport;
use std::path::Path;

type Inner = ExplanationBuilder<
    ConnectionPulse,
    ConnectionPulseTrace,
    ConnectionPulseMotion,
    ConnectionPulseTiming,
>;

pub struct ExplainBuilder(Inner);

pub fn explain(name: &'static str) -> ExplainBuilder {
    ExplainBuilder(Inner::new(name))
}

impl ExplainBuilder {
    pub fn state(self, state: ConnectionPulse) -> Self {
        Self(self.0.state(state))
    }

    pub fn algorithm(mut self, algorithm: fn(ConnectionPulse) -> ConnectionPulseTrace) -> Self {
        let state = self
            .0
            .state
            .take()
            .expect("connection-pulse: state needed before algorithm");
        let trace = algorithm(state);
        self.0.state = Some(state);
        Self(self.0.algorithm(trace))
    }

    pub fn motion(self, motion: fn() -> ConnectionPulseMotion) -> Self {
        Self(self.0.motion(motion()))
    }

    pub fn timing(self, timing: ConnectionPulseTiming) -> Self {
        Self(self.0.timing(timing))
    }

    pub fn view<V>(self, view: fn() -> V) -> Self {
        Self(self.0.view(view))
    }

    pub fn build(self) -> (Box<dyn Playable>, Viewport) {
        let name = self.0.name;
        let (state, trace, motion, timing) = self
            .0
            .take()
            .expect("connection-pulse: state, algorithm, motion required");
        build_connection_pulse(name, state, trace, motion, timing)
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
