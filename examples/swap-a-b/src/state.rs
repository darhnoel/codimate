#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemId {
    A,
    B,
}

impl ItemId {
    pub(crate) fn label(self) -> &'static str {
        match self {
            ItemId::A => "A",
            ItemId::B => "B",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotId {
    Left,
    Right,
}

impl SlotId {
    pub(crate) fn index(self) -> usize {
        match self {
            SlotId::Left => 0,
            SlotId::Right => 1,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            SlotId::Left => "slot 0",
            SlotId::Right => "slot 1",
        }
    }
}

#[derive(Clone, Copy)]
pub struct SwapAB {
    initial: [(ItemId, SlotId); 2],
}

impl SwapAB {
    pub fn new() -> Self {
        Self {
            initial: [(ItemId::A, SlotId::Left), (ItemId::B, SlotId::Right)],
        }
    }

    pub(crate) fn initial(self) -> [(ItemId, SlotId); 2] {
        self.initial
    }
}

impl Default for SwapAB {
    fn default() -> Self {
        Self::new()
    }
}
