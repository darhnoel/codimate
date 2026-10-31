# ADR 0011 — Frames are rendered in parallel, in batches

**Status:** Accepted — 2026-09-14

## Context

The exporter resolved one frame, rasterized it, blocked writing it to ffmpeg's
stdin, and repeated. One thread, on a machine with ten cores.

A profile of `fourier_einstein` — 74 Scenes, 141,806 shapes, 1,200 frames —
put 8,344 samples on that one thread:

    8344  render                            (all)
    7511    export_mp4                      90%
    5205      rasterize_commands            62%
    3522        render_path → stroke_path   42%
     633      write_all to ffmpeg            8%
     833    building the explanation        10%

Ninety percent sits inside the frame loop, and most of that is stroke
tessellation — turning a stroked path into a fillable outline. That cost
tracks the number of path segments, not the number of pixels, which is why
render time is flat from 320x180 to 1280x720 and only rises at 1080p. It also
means the work is arithmetic, not memory bandwidth, and arithmetic is what
more cores are for.

## Decision

**Resolve and rasterize frames across cores in batches of eight, and write
them to ffmpeg in order.**

The soundness argument is the one the whole engine already rests on. ADR 0008's
Invariant 1 says `f(t) → Scene` is pure: a frame is a function of its instant
and nothing else. No frame can observe another, so no frame cares what order
they are computed in.

The state frames do share is small and already safe. The formula glyph cache
is a `Mutex<HashMap>` and is on the hit path after the first frame. The font
registry is immutable `&'static` data.

### Batches, not the whole timeline

`par_iter` over every frame would hold every bitmap: `frames × width × height
× 4`. For a two-minute 1080p render that is tens of gigabytes.

A batch bounds it at `BATCH × frame`, about 66 MB at 1080p. Measured on
`fourier_einstein`:

    BATCH=4    9.53s   399 MB
    BATCH=8    8.27s   425 MB
    BATCH=16   8.02s   457 MB

Eight is where the curve flattens. Sixteen buys 3% for another 32 MB.

### Tweens become `Send + Sync`

`Animated<T>` wrapped `Arc<dyn Fn(f32) -> T>`, which is neither. Adding the
bounds costs nothing — Invariant 1 already requires the closure to be pure, and
a pure function of one `f32` has nothing to share — but it does mean the bound
now appears in the signature of `Animated::new`, `Animated::map`,
`Animated::ease` and `Scene::ease`.

That is the real price of this decision: the escape hatch for custom motion now
demands a thread-safe closure. A closure that captures an `Rc`, or anything
else non-`Sync`, no longer compiles. Given the closure was already required to
be pure, anything this rejects was already violating Invariant 1 — the compiler
simply says so now instead of the renderer misbehaving later.

### Frame instants stay accumulated

The sampling loop advanced with `elapsed += step`. Computing `i / fps` instead
would be tidier and would parallelise without a separate iterator, but the two
disagree in the last bits of a float once `i` grows, and every rendered frame
after that point differs. `FrameTimes` reproduces the accumulation exactly, so
the parallel exporter samples the same instants the serial one did.

## Consequences

- **`fourier_einstein` renders in 8.27s against 19.12s**, a 2.3x speedup on a
  machine with four performance cores and six efficiency cores. Not the 4-6x a
  core count suggests: eight percent of the loop is a blocking write to ffmpeg,
  ten percent is building the explanation before any frame is drawn, and the
  efficiency cores are much slower than the performance ones.
- **Peak memory rises from 372 MB to 425 MB** on that example, the batch of
  bitmaps in flight. That gives back most of what the previous change saved, so
  the honest summary of the two together is: same memory as before, 2.3x faster.
- **Output is unchanged.** `bubble_sort`, `pendulum` and `fourier_einstein` all
  render byte-identical to the serial exporter, at every batch size tried.
  `pendulum` is the one that matters, since it uses `focus()` and `overlay()`.
- **`export_mp4` now requires `Playable + Sync`.**
- **rayon is a new dependency** of `codimate-export`, and therefore of the
  wheel.

## Alternatives rejected

**Cache path geometry between frames.** A shape whose geometry has not changed
is re-tessellated every frame, and `stroke_path` is 42% of the render. Caching
attacks the same cost without threads. Rejected *for now* rather than on the
merits: it needs a key that is exactly as strict as the geometry, and a wrong
key is a silently wrong picture rather than a crash. Parallelism is safe by
construction here, so it goes first. The two compose.

**A thread pool with a channel and a reorder buffer.** Strictly better
throughput — no sync point between batches, so a slow frame does not stall
seven others. Rejected as more machinery than the measurement justifies: the
gap between BATCH=8 and BATCH=16 is 3%, which caps what perfect pipelining
could win here.

**Let ffmpeg do more of the work.** libx264 already uses fifteen threads and is
not the bottleneck — it keeps up at `speed=3.23x` while one core rasterizes.
Nothing to win.

**Leave it serial.** The honest default, and it held while the engine was
young. What changed is that examples got big enough to notice: a 1,200-frame
render at 20 seconds is slow enough to discourage iterating on it, and
iterating on it is the whole point.

## Amended 2026-10-31: frame instants are exact, not accumulated

"Frame instants stay accumulated" above kept renders byte-identical to the
serial exporter. The price turned out to be a clock that drifts. Adding
`1 / fps` in `f32` rounds at every step, and the error piles up with the
length of the film:

    83 s at 60 fps     frames up to   3 ms from where they are shown
    600 s at 60 fps    frames up to 102 ms from where they are shown,
                       and 36,008 frames encoded where 36,001 belong

Narration starts are read off the event timeline, not off the frames, so on a
long film the picture slid out of step with its sound by more than the ~45 ms
at which lip sync starts to show.

`FrameTimes` now samples frame `i` at `i / fps`, worked out in `f64`, and the
last frame still lands on the duration. A duration a hair over a whole number
of frames (1.1 s at 30 fps is 33.0000007 frames) counts as that number rather
than gaining an extra frame. Frames can still be resolved in any order, which
is all the parallel exporter needs.

**This changes every render's bytes once**, by the last bits of a float in each
sampled instant: the trade the original decision refused, accepted now that it
buys a clock that keeps time.
