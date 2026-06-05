use crate::FasterDijkstra;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FasterDijkstraAction {
    Problem,
    SortingBarrier,
    QuestionShift,
    BoundedWindow,
    MultiSourceBatch,
    KStepRelaxation,
    FindPivots,
    RecursiveBmssp,
    BatchDataStructure,
    Result,
}

#[derive(Clone, Copy)]
pub struct FasterDijkstraStep {
    pub(crate) index: usize,
    pub(crate) action: FasterDijkstraAction,
}

pub struct FasterDijkstraTrace {
    pub(crate) steps: Vec<FasterDijkstraStep>,
}

pub fn faster_dijkstra_algorithm(_state: FasterDijkstra) -> FasterDijkstraTrace {
    let actions = [
        FasterDijkstraAction::Problem,
        FasterDijkstraAction::SortingBarrier,
        FasterDijkstraAction::QuestionShift,
        FasterDijkstraAction::BoundedWindow,
        FasterDijkstraAction::MultiSourceBatch,
        FasterDijkstraAction::KStepRelaxation,
        FasterDijkstraAction::FindPivots,
        FasterDijkstraAction::RecursiveBmssp,
        FasterDijkstraAction::BatchDataStructure,
        FasterDijkstraAction::Result,
    ];

    FasterDijkstraTrace {
        steps: actions
            .into_iter()
            .enumerate()
            .map(|(index, action)| FasterDijkstraStep { index, action })
            .collect(),
    }
}
