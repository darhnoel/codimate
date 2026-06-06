//! codimate-effects — pure timeless visual change recipes.
//!
//! Effects are Manim-style authoring conveniences without Manim's mutable
//! lifecycle. An [`Effect`] owns a Scene-shaped visual recipe and has no
//! duration until [`Effect::animate`] is called.

use codimate_animation::{animation, Animation};
use codimate_core::{tween, ConcreteScene, Scene, SceneTransformError};

pub use codimate_core::SceneTransformError as TransformError;

#[derive(Clone)]
pub struct Effect {
    scene: Scene,
}

impl Effect {
    pub fn new(scene: Scene) -> Self {
        Effect { scene }
    }

    pub fn resolve(&self, t: f32) -> ConcreteScene {
        self.scene.resolve(t)
    }

    pub fn ease(self, curve: impl Fn(f32) -> f32 + 'static) -> Self {
        Effect {
            scene: self.scene.ease(curve),
        }
    }

    pub fn animate(self, name: impl Into<String>, duration: f32) -> Animation {
        animation(name, duration, self.scene)
    }

    pub fn into_scene(self) -> Scene {
        self.scene
    }
}

pub fn show(scene: Scene) -> Effect {
    Effect::new(scene)
}

pub fn fade_in(scene: Scene) -> Effect {
    Effect::new(scene.with_opacity(tween(0.0, 1.0)))
}

pub fn fade_out(scene: Scene) -> Effect {
    Effect::new(scene.with_opacity(tween(1.0, 0.0)))
}

pub fn reveal(scene: Scene) -> Effect {
    Effect::new(scene.reveal(tween(0.0, 1.0)))
}

pub fn try_transform(from: Scene, to: Scene) -> Result<Effect, SceneTransformError> {
    from.try_lerp_to(&to).map(Effect::new)
}
