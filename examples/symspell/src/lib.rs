mod algorithm;
mod builder;
mod motion;
mod state;
mod style;
mod timing;
mod view;

use codimate_animation::Playable;
use codimate_layout::Viewport;

pub use algorithm::{symspell_algorithm, SymStep, SymTrace};
pub use builder::{explain, ExplainBuilder};
pub use motion::{symspell_motion, SymSpellMotion};
pub use state::{SymSpell, DICT_ENTRIES, K, QUERY};
pub use timing::SymSpellTiming;
pub use view::{symspell_view, SymSpellView};

pub fn create() -> (Box<dyn Playable>, Viewport) {
    explain("SymSpell")
        .state(SymSpell::new())
        .view(symspell_view)
        .algorithm(symspell_algorithm)
        .motion(symspell_motion)
        .timing(SymSpellTiming::default())
        .build()
}
