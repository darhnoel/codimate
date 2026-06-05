use crate::NewtonLaws;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewtonLawAction {
    Intro,
    FirstLaw,
    SecondLaw,
    ThirdLaw,
    Summary,
}

#[derive(Clone, Copy)]
pub struct NewtonLawStep {
    pub(crate) index: usize,
    pub(crate) action: NewtonLawAction,
    pub(crate) mass: f32,
    pub(crate) force: f32,
    pub(crate) acceleration: f32,
}

pub struct NewtonLawTrace {
    pub(crate) steps: Vec<NewtonLawStep>,
}

pub fn newton_laws_algorithm(state: NewtonLaws) -> NewtonLawTrace {
    let actions = [
        NewtonLawAction::Intro,
        NewtonLawAction::FirstLaw,
        NewtonLawAction::SecondLaw,
        NewtonLawAction::ThirdLaw,
        NewtonLawAction::Summary,
    ];

    NewtonLawTrace {
        steps: actions
            .into_iter()
            .enumerate()
            .map(|(index, action)| NewtonLawStep {
                index,
                action,
                mass: state.mass(),
                force: state.force(),
                acceleration: state.acceleration(),
            })
            .collect(),
    }
}
