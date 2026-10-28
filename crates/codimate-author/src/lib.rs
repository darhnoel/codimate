//! The Authoring Core (ADR 0030): everything an Authoring Surface does that is
//! not the author's own code, written once for every Skin.
//!
//! One door, [`call`]: an operation's name and its data as JSON in, `{"ok": …}`
//! or `{"error": "…"}` out (ADR 0032). The Core keeps no state between calls;
//! what a call needs (the Group it draws on, the shape it changes) comes with
//! it, so any language can drive it and every call can be tested on its own.

mod call;
mod refusal;

pub use call::*;
pub use refusal::*;

mod axes;

mod film;

mod generated;

mod marker;

mod place;

mod preset;

mod pyrandom;

mod shape;

mod surface;

pub use shape::Shape;
