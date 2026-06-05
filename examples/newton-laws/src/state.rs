#[derive(Clone, Copy)]
pub struct NewtonLaws {
    mass: f32,
    force: f32,
}

impl NewtonLaws {
    pub fn new() -> Self {
        Self {
            mass: 4.0,
            force: 12.0,
        }
    }

    pub(crate) fn mass(self) -> f32 {
        self.mass
    }

    pub(crate) fn force(self) -> f32 {
        self.force
    }

    pub(crate) fn acceleration(self) -> f32 {
        self.force / self.mass
    }
}

impl Default for NewtonLaws {
    fn default() -> Self {
        Self::new()
    }
}
