//! The Waist (ADR 0032): every Skin's one crossing into the Authoring Core,
//! and into the Engine for what only it does (render, draw a frame).
//!
//! One crate however many languages there are, a module per transport:
//! `python` (PyO3, built by maturin) and `web` (WebAssembly, for JavaScript).
//! Each translates and nothing else; what an authoring call means lives in
//! `codimate-author`, what a frame looks like in the Engine.

#[cfg(feature = "python")]
mod python;
#[cfg(feature = "wasm")]
mod web;
