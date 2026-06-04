use crate::{
    view::build_merge_sort, MergeSort, MergeSortMotion, MergeSortTiming, MergeSortView, MergeTrace,
};
use codimate_animation::Playable;
use codimate_export::{export_mp4, ExportConfig};
use codimate_layout::Viewport;
use std::path::Path;

pub struct ExplainBuilder {
    name: &'static str,
    state: Option<MergeSort>,
    algorithm: Option<fn(MergeSort) -> MergeTrace>,
    motion: Option<MergeSortMotion>,
    timing: MergeSortTiming,
}

pub fn explain(name: &'static str) -> ExplainBuilder {
    ExplainBuilder {
        name,
        state: None,
        algorithm: None,
        motion: None,
        timing: MergeSortTiming::default(),
    }
}

impl ExplainBuilder {
    pub fn state(mut self, state: MergeSort) -> Self {
        self.state = Some(state);
        self
    }

    pub fn view(self, view: fn() -> MergeSortView) -> Self {
        let _ = view();
        self
    }

    pub fn algorithm(mut self, algorithm: fn(MergeSort) -> MergeTrace) -> Self {
        self.algorithm = Some(algorithm);
        self
    }

    pub fn motion(mut self, motion: fn() -> MergeSortMotion) -> Self {
        self.motion = Some(motion());
        self
    }

    pub fn timing(mut self, timing: MergeSortTiming) -> Self {
        self.timing = timing;
        self
    }

    pub fn build(self) -> (Box<dyn Playable>, Viewport) {
        let state = self.state.expect("merge sort state must be provided");
        let algorithm = self
            .algorithm
            .expect("merge sort algorithm must be provided");
        build_merge_sort(
            self.name,
            state,
            algorithm(state),
            self.motion.expect("merge sort motion must be provided"),
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
