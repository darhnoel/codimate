use crate::{
    view::build_faster_dijkstra, FasterDijkstra, FasterDijkstraMotion, FasterDijkstraTiming,
    FasterDijkstraTrace, FasterDijkstraView,
};
use codimate_animation::Playable;
use codimate_export::{export_mp4, ExportConfig};
use codimate_layout::Viewport;
use std::path::Path;

pub struct ExplainBuilder {
    name: &'static str,
    state: Option<FasterDijkstra>,
    algorithm: Option<fn(FasterDijkstra) -> FasterDijkstraTrace>,
    motion: Option<FasterDijkstraMotion>,
    timing: FasterDijkstraTiming,
}

pub fn explain(name: &'static str) -> ExplainBuilder {
    ExplainBuilder {
        name,
        state: None,
        algorithm: None,
        motion: None,
        timing: FasterDijkstraTiming::default(),
    }
}

impl ExplainBuilder {
    pub fn state(mut self, state: FasterDijkstra) -> Self {
        self.state = Some(state);
        self
    }

    pub fn view(self, view: fn() -> FasterDijkstraView) -> Self {
        let _ = view();
        self
    }

    pub fn algorithm(mut self, algorithm: fn(FasterDijkstra) -> FasterDijkstraTrace) -> Self {
        self.algorithm = Some(algorithm);
        self
    }

    pub fn motion(mut self, motion: fn() -> FasterDijkstraMotion) -> Self {
        self.motion = Some(motion());
        self
    }

    pub fn timing(mut self, timing: FasterDijkstraTiming) -> Self {
        self.timing = timing;
        self
    }

    pub fn build(self) -> (Box<dyn Playable>, Viewport) {
        let state = self.state.expect("faster dijkstra state must be provided");
        let algorithm = self
            .algorithm
            .expect("faster dijkstra algorithm must be provided");
        build_faster_dijkstra(
            self.name,
            state,
            algorithm(state),
            self.motion
                .expect("faster dijkstra motion must be provided"),
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
