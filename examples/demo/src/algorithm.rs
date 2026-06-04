#[derive(Clone, Copy)]
pub enum DemoStep {
    GrowCircle,
    MoveRect,
    MorphPath,
}

pub struct DemoTrace {
    pub steps: Vec<DemoStep>,
}

pub fn demo_algorithm(_state: crate::Demo) -> DemoTrace {
    DemoTrace {
        steps: vec![
            DemoStep::GrowCircle,
            DemoStep::MoveRect,
            DemoStep::MorphPath,
        ],
    }
}
