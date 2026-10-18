//! The entry point: every Scene in, one playable explanation out.

use crate::*;

pub struct Explanation {
    /// `(start_seconds, duration, scene)` — scenes hold their own local `t`.
    pub segments: Vec<(f32, f32, Scene)>,
    pub total: f32,
}

impl Playable for Explanation {
    fn name(&self) -> &str {
        "explanation"
    }

    fn duration(&self) -> f32 {
        self.total
    }

    fn resolve(&self, t: f32) -> ConcreteScene {
        let seconds = t.clamp(0.0, 1.0) * self.total;

        for (start, duration, scene) in &self.segments {
            if seconds < start + duration {
                return scene.resolve(((seconds - start) / duration).clamp(0.0, 1.0));
            }
        }

        match self.segments.last() {
            Some((_, _, scene)) => scene.resolve(1.0),
            None => ConcreteScene { children: vec![] },
        }
    }
}

/// Build a playable explanation from the Scenes of every Trace Event.
///
/// `scenes` is one longer than `durations`: the opening Scene, then one per
/// event. Zero-length segments are dropped rather than producing an empty time
/// interval.
///
/// This is the whole surface a frontend needs. Everything else in this crate
/// is how it is done.
pub fn explanation(
    scenes: Vec<Vec<Shape>>,
    cameras: &[Option<Focus>],
    rules: &[Rule],
    durations: &[f32],
    viewport: (f32, f32),
) -> Result<Explanation> {
    if scenes.len() != durations.len() + 1 {
        return Err(Error(format!(
            "got {} scenes for {} durations — expected {}",
            scenes.len(),
            durations.len(),
            durations.len() + 1
        )));
    }

    for shape in scenes.iter().flatten() {
        if !KINDS.contains(&shape.kind.as_str()) {
            return Err(Error(format!(
                "unknown kind {:?} on item {:?} — Codimate draws {}",
                shape.kind,
                shape.item,
                KINDS.join(", ")
            )));
        }
    }

    // Scenes are framed as the diff walks them, not all at once. The diff
    // never looks further than one step back, so two framed Scenes are alive
    // at a time rather than one per event — on a Scene of a few thousand
    // shapes that is the difference between holding the whole payload again
    // and holding a pair of them.
    //
    // `scenes` is consumed for the same reason: framing used to clone every
    // Scene while the caller's copy was still alive, so the payload existed
    // twice before a single frame was drawn.
    let mut scenes = scenes.into_iter();
    let mut prev = framed(
        scenes.next().expect("scene count checked above"),
        cameras.first().and_then(|c| c.as_ref()),
        viewport,
    )?;

    let mut segments = Vec::new();
    let mut cursor = 0.0f32;

    for (i, duration) in durations.iter().enumerate() {
        // Framed even when the segment is dropped: a zero-length event still
        // advances which Scene the next segment diffs against.
        let next = framed(
            scenes.next().expect("scene count checked above"),
            cameras.get(i + 1).and_then(|c| c.as_ref()),
            viewport,
        )?;
        if *duration > 0.0 {
            segments.push((cursor, *duration, build_segment(&prev, &next, rules)?));
            cursor += duration;
        }
        prev = next;
    }

    if segments.is_empty() {
        return Err(Error(
            "nothing to render — no events with a positive duration".into(),
        ));
    }

    Ok(Explanation {
        segments,
        total: cursor,
    })
}
