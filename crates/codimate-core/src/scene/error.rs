/// Why two Scenes cannot be blended: they do not have the same shape.
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
