pub const PEG_COUNT: usize = 3;
pub const DISK_COUNT: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Peg {
    A,
    B,
    C,
}

impl Peg {
    pub(crate) fn index(self) -> usize {
        match self {
            Peg::A => 0,
            Peg::B => 1,
            Peg::C => 2,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Peg::A => "A",
            Peg::B => "B",
            Peg::C => "C",
        }
    }
}

#[derive(Clone, Copy)]
pub struct HanoiTower {
    disk_count: usize,
}

impl HanoiTower {
    pub fn new(disk_count: usize) -> Self {
        assert!(disk_count > 0, "Tower of Hanoi needs at least one disk");
        assert!(
            disk_count <= DISK_COUNT,
            "this example supports up to {DISK_COUNT} disks"
        );
        Self { disk_count }
    }

    pub(crate) fn disk_count(self) -> usize {
        self.disk_count
    }
}
