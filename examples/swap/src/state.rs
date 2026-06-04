#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BallKey {
    Left,
    Right,
}

#[derive(Clone, Copy)]
pub struct Swap {
    left: &'static str,
    right: &'static str,
}

impl Swap {
    pub fn new(left: &'static str, right: &'static str) -> Self {
        Self { left, right }
    }

    pub(crate) fn ball(self, key: BallKey) -> Ball {
        match key {
            BallKey::Left => Ball {
                key,
                label: self.left,
            },
            BallKey::Right => Ball {
                key,
                label: self.right,
            },
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Ball {
    pub(crate) key: BallKey,
    pub(crate) label: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct SwapState {
    pub(crate) a: Option<Ball>,
    pub(crate) b: Option<Ball>,
    pub(crate) temp: Option<Ball>,
}

impl SwapState {
    pub(crate) fn start(swap: Swap) -> Self {
        Self {
            a: Some(swap.ball(BallKey::Left)),
            b: Some(swap.ball(BallKey::Right)),
            temp: None,
        }
    }

    pub(crate) fn apply(mut self, movement: crate::SwapMove) -> Self {
        let ball = self.take(movement.from);
        self.put(movement.to, ball);
        self
    }

    fn take(&mut self, slot: &'static str) -> Ball {
        match slot {
            "a" => self.a.take(),
            "b" => self.b.take(),
            "temp" => self.temp.take(),
            _ => None,
        }
        .expect("source slot contains the moving ball")
    }

    fn put(&mut self, slot: &'static str, ball: Ball) {
        match slot {
            "a" => self.a = Some(ball),
            "b" => self.b = Some(ball),
            "temp" => self.temp = Some(ball),
            _ => unreachable!("target slot exists"),
        }
    }
}
