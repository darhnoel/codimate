#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SceneTransformError {
    ChildCountMismatch {
        from: usize,
        to: usize,
    },
    NodeKindMismatch {
        index: usize,
        from: &'static str,
        to: &'static str,
    },
    WaypointCountMismatch {
        from: usize,
        to: usize,
    },
}
