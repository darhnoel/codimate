#[derive(Clone, Copy)]
pub struct NewtonLawsTiming {
    pub(crate) intro: f32,
    pub(crate) law: f32,
    pub(crate) summary: f32,
}

impl Default for NewtonLawsTiming {
    fn default() -> Self {
        Self {
            intro: 2.2,
            law: 3.8,
            summary: 3.4,
        }
    }
}
