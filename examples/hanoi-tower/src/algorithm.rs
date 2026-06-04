use crate::{HanoiTower, Peg};

#[derive(Clone)]
pub struct HanoiMove {
    pub(crate) step: usize,
    pub(crate) disk: usize,
    pub(crate) from: Peg,
    pub(crate) to: Peg,
    pub(crate) state_before: [Vec<usize>; 3],
}

impl HanoiMove {
    pub(crate) fn title(&self) -> String {
        format!(
            "Move disk {} from {} to {}",
            self.disk,
            self.from.label(),
            self.to.label()
        )
    }
}

pub struct HanoiTrace {
    pub(crate) start: [Vec<usize>; 3],
    pub(crate) moves: Vec<HanoiMove>,
    pub(crate) final_state: [Vec<usize>; 3],
}

pub fn hanoi_algorithm(state: HanoiTower) -> HanoiTrace {
    let disk_count = state.disk_count();
    let mut pegs = [Vec::new(), Vec::new(), Vec::new()];
    for disk in (1..=disk_count).rev() {
        pegs[Peg::A.index()].push(disk);
    }

    let start = pegs.clone();
    let mut moves = Vec::new();
    solve(disk_count, Peg::A, Peg::C, Peg::B, &mut pegs, &mut moves);

    HanoiTrace {
        start,
        moves,
        final_state: pegs,
    }
}

fn solve(
    count: usize,
    from: Peg,
    to: Peg,
    spare: Peg,
    pegs: &mut [Vec<usize>; 3],
    moves: &mut Vec<HanoiMove>,
) {
    if count == 0 {
        return;
    }

    solve(count - 1, from, spare, to, pegs, moves);
    move_top_disk(from, to, pegs, moves);
    solve(count - 1, spare, to, from, pegs, moves);
}

fn move_top_disk(from: Peg, to: Peg, pegs: &mut [Vec<usize>; 3], moves: &mut Vec<HanoiMove>) {
    let state_before = pegs.clone();
    let disk = pegs[from.index()]
        .pop()
        .expect("recursive Hanoi move has a disk to move");
    pegs[to.index()].push(disk);
    moves.push(HanoiMove {
        step: moves.len() + 1,
        disk,
        from,
        to,
        state_before,
    });
}
