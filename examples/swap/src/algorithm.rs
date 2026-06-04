use crate::BallKey;

#[derive(Clone, Copy)]
pub struct SwapMove {
    pub(crate) ball: BallKey,
    pub(crate) from: &'static str,
    pub(crate) to: &'static str,
    pub(crate) during_title: &'static str,
    pub(crate) pause_title: Option<&'static str>,
}

pub fn swap_algorithm() -> [SwapMove; 3] {
    [
        SwapMove {
            ball: BallKey::Left,
            from: "a",
            to: "temp",
            during_title: "Save A in temp first",
            pause_title: Some("A is safe in temp"),
        },
        SwapMove {
            ball: BallKey::Right,
            from: "b",
            to: "a",
            during_title: "Move B into A's old slot",
            pause_title: Some("B is now in A's slot"),
        },
        SwapMove {
            ball: BallKey::Left,
            from: "temp",
            to: "b",
            during_title: "Move A into B's old slot",
            pause_title: None,
        },
    ]
}
