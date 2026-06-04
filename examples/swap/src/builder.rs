use crate::{view::build_swap, Swap, SwapMotion, SwapMove, SwapTiming, SwapView};
use codimate_animation::Playable;
use codimate_export::{export_mp4, ExportConfig};
use codimate_layout::Viewport;
use std::path::Path;

pub struct ExplainBuilder {
    name: &'static str,
    state: Option<Swap>,
    moves: Option<Vec<SwapMove>>,
    motion: Option<SwapMotion>,
    timing: SwapTiming,
}

pub fn explain(name: &'static str) -> ExplainBuilder {
    ExplainBuilder {
        name,
        state: None,
        moves: None,
        motion: None,
        timing: SwapTiming::default(),
    }
}

impl ExplainBuilder {
    pub fn state(mut self, state: Swap) -> Self {
        self.state = Some(state);
        self
    }

    pub fn view(self, view: fn() -> SwapView) -> Self {
        let _ = view();
        self
    }

    pub fn algorithm(mut self, algorithm: fn() -> [SwapMove; 3]) -> Self {
        self.moves = Some(algorithm().to_vec());
        self
    }

    pub fn motion(mut self, motion: fn() -> SwapMotion) -> Self {
        self.motion = Some(motion());
        self
    }

    pub fn timing(mut self, timing: SwapTiming) -> Self {
        self.timing = timing;
        self
    }

    pub fn build(self) -> (Box<dyn Playable>, Viewport) {
        build_swap(
            self.name,
            self.state.expect("swap state must be provided"),
            self.moves.expect("swap algorithm must be provided"),
            self.motion.expect("swap motion must be provided"),
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
