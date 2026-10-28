//! codimate-animation — Layer 3 (Composition).
//!
//! Duration lives here, not in `codimate-core`.

mod animation;
mod parallel;
mod playable;
mod sequence;
mod stagger;

pub use animation::*;
pub use parallel::*;
pub use playable::*;
pub use sequence::*;
pub use stagger::*;
