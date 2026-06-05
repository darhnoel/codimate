use crate::{ItemId, SlotId, SwapAB};

#[derive(Clone, Copy)]
pub struct SwapABEvent {
    pub(crate) item: ItemId,
    pub(crate) from: SlotId,
    pub(crate) to: SlotId,
}

pub struct SwapABTrace {
    pub(crate) initial: [(ItemId, SlotId); 2],
    pub(crate) swap: [SwapABEvent; 2],
    pub(crate) final_mapping: [(ItemId, SlotId); 2],
}

pub fn swap_a_b_algorithm(state: SwapAB) -> SwapABTrace {
    SwapABTrace {
        initial: state.initial(),
        swap: [
            SwapABEvent {
                item: ItemId::A,
                from: SlotId::Left,
                to: SlotId::Right,
            },
            SwapABEvent {
                item: ItemId::B,
                from: SlotId::Right,
                to: SlotId::Left,
            },
        ],
        final_mapping: [(ItemId::A, SlotId::Right), (ItemId::B, SlotId::Left)],
    }
}
