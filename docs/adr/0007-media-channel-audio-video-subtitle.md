# ADR 0007 — Media: audio is a sibling channel, never inside `Scene`

**Status:** Accepted — 2026-06-09

## Context
Image, video, audio, and subtitle are must-have features. Three of the four are
not the same kind of thing, and naively adding them would quietly break the One
Law (`f(t) → Scene`). A `Scene` is "what is on screen at instant `t`." Sound is
not on screen. The moment audio leaks into `Scene`, `f(t) → Scene` becomes a lie
and `ConcreteScene` (a clean, `PartialEq`-testable visual value) gains a
non-visual passenger.

## Decision

### 1. The One Law stays visual-only; it *generalizes* rather than breaks
```
TODAY:   render(t) = visual(t) -> Scene
v-next:  render(t) = { visual(t) -> Scene ,  audio(t) -> AudibleClips }
                       ^pure, unchanged       ^second pure function, beside it
```
Export samples **both** and muxes via the ffmpeg already used for frames
([ADR 0001](0001-rendering-backend.md)).

### 2. The top-level authoring unit becomes `Movie`
```rust
struct Movie {
    visual: Box<dyn Playable>,   // f(t) -> Scene   (unchanged, pure)
    audio:  AudioTrack,          // f(t) -> AudibleClips   (new, pure)
}
render(&movie, "out.mp4")   // samples both, ffmpeg muxes
```
The split is internal — `render()` takes a `Movie`, and a silent animation is
just a `Movie` with an empty `AudioTrack`. The beginner never sees the seam.

### 3. New shared concept: `Clip { source, in, out, rate }`
A media clip has a *source file*, a *trim* (in/out), and a *playback rate*. Audio
uses `Clip` directly. Video uses it twice: once as `Geometry::Video` (picture) in
the Scene, once as an audio `Clip` on the `AudioTrack`. Authoring
`video("x.mp4")` fans out to **both** channels behind one call.

This is a richer timeline than today's pure `Sequence/Parallel/Stagger`, which
have no source/trim/rate. It is additive, and it lives at the Composition/export
layer — `Movie`, `AudioTrack`, and `Clip` in `codimate-animation`.

### 4. Media is authored in real seconds, reconciled at the timeline
A clip has its own internal time (trim/rate) but is placed inside an `animation`
that resolves on normalized `t ∈ [0,1]`. Clips are placed by **absolute seconds**
on the `AudioTrack`/timeline; the Scene-side `Geometry::Video` maps scene-`t` →
source frame. The timeline reconciles seconds against normalized `t`. Media is
*not* forced onto normalized `t`.

### 5. How each of the four maps
| Feature | Visual? | In a `Scene`? | Where it lives |
|---|---|---|---|
| **Image** | yes | yes — `Geometry::Image { src, w, h }` | pure data in `core`; tiny-skia decodes at render |
| **Subtitle** | yes | yes — timed `Text` | pure core + `.srt → Vec<Text>` loader sugar |
| **Video (picture)** | yes | yes — `Geometry::Video` | pure core; sample frame at scene-`t` |
| **Video (sound)** | no | no | audio `Clip` on `AudioTrack` |
| **Audio** | no | no | `AudioTrack` (seconds-based), ffmpeg mux |

- **Subtitle is refused as a primitive.** It is timed `Text` + an `.srt` loader.
  Burn-in (motionity-style), not soft-subs.
- `Geometry::Image`/`Geometry::Video` are **pure data** in `core` (just `src` +
  dims/trim/rate); decoding happens render-side, so `core` stays std-only
  ([ADR 0004](0004-layers-as-modules-not-crates.md) invariant preserved).

## Consequences
- `codimate-render` grows a **video decoder** dependency (ffmpeg-side). This is
  the first non-trivial dep into the render path — `render` is no longer "just
  tiny-skia." Accepted for Phase 2.
- `codimate-export` gains audio decode + A/V muxing.
- A new `Clip`/`AudioTrack`/`Movie` vocabulary at Layer 3, distinct from the pure
  composition primitives — clips carry source/trim/rate; `Sequence` et al. do not.
- The visual core, `Scene`, and `ConcreteScene` are untouched by sound. If a
  future reviewer proposes putting audio into `Scene` for "one unified tree," the
  bar is: does the One Law survive? It does not. Keep the sibling channel.
