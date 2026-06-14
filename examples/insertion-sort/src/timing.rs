use crate::InsertionAction;

#[derive(Clone, Copy)]
pub struct InsertionSortTiming {
    pub choose: f32,
    pub compare: f32,
    pub shift: f32,
    pub insert: f32,
    pub final_hold: f32,
}

impl Default for InsertionSortTiming {
    fn default() -> Self {
        let default_time = 0.1;
        Self {
            choose: default_time,
            compare: default_time,
            shift: default_time,
            insert: default_time,
            final_hold: default_time,
        }
    }
}

impl InsertionSortTiming {
    pub(crate) fn duration_for(self, action: InsertionAction) -> f32 {
        match action {
            InsertionAction::ChooseKey { .. } => self.choose,
            InsertionAction::Compare { .. } => self.compare,
            InsertionAction::Shift { .. } => self.shift,
            InsertionAction::Insert { .. } => self.insert,
        }
    }
}
