use crate::{
    view::build_connection_pulse, ConnectionPulse, ConnectionPulseMotion, ConnectionPulseTiming,
    ConnectionPulseTrace, ConnectionPulseView,
};
use codimate_animation::Playable;
use codimate_export::{export_mp4, ExportConfig};
use codimate_layout::Viewport;
use std::path::Path;

pub struct ExplainBuilder {
    name: &'static str,
    state: Option<ConnectionPulse>,
    trace: Option<ConnectionPulseTrace>,
    motion: Option<ConnectionPulseMotion>,
    timing: ConnectionPulseTiming,
}

pub fn explain(name: &'static str) -> ExplainBuilder {
    ExplainBuilder {
        name,
        state: None,
        trace: None,
        motion: None,
        timing: ConnectionPulseTiming::default(),
    }
}

impl ExplainBuilder {
    pub fn state(mut self, state: ConnectionPulse) -> Self {
        self.state = Some(state);
        self
    }

    pub fn view(self, view: fn() -> ConnectionPulseView) -> Self {
        let _ = view();
        self
    }

    pub fn algorithm(mut self, algorithm: fn(ConnectionPulse) -> ConnectionPulseTrace) -> Self {
        let state = self
            .state
            .expect("connection-pulse state must be provided before algorithm");
        self.trace = Some(algorithm(state));
        self.state = Some(state);
        self
    }

    pub fn motion(mut self, motion: fn() -> ConnectionPulseMotion) -> Self {
        self.motion = Some(motion());
        self
    }

    pub fn timing(mut self, timing: ConnectionPulseTiming) -> Self {
        self.timing = timing;
        self
    }

    pub fn build(self) -> (Box<dyn Playable>, Viewport) {
        build_connection_pulse(
            self.name,
            self.state.expect("connection-pulse state must be provided"),
            self.trace
                .expect("connection-pulse algorithm must be provided"),
            self.motion
                .expect("connection-pulse motion must be provided"),
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
