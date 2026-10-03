# maxwell — how did Maxwell find his equations?

```bash
.venv/bin/python python/examples/maxwell/main.py    # results/maxwell.mp4
```

About ninety seconds, in Khmer captions, on one question: nobody invents four
equations from nothing, so how did Maxwell get there? He took laws other people
had found, noticed a hole between them, patched it, and the patch predicted that
light is a wave. Nine scenes, each built on a real object:

1. **The question** — four equations on screen, and the one term added later.
2. **Electricity moves a compass** — a current, and a needle that swings (1820).
3. **Magnetism makes electricity** — a magnet pushed through a coil (1831).
4. **Faraday's picture** — field lines that fill space and never end.
5. **The hole** — a charging capacitor: one loop, two surfaces shown one after the other.
   The wire pierces the first and nothing pierces the second, so Ampère gives two answers.
6. **Maxwell's patch** — a changing E in the gap acts like a current.
7. **The wave** — E and B as two sine curves at right angles, rising and falling
   together and moving along x, drawn flat with `science.Camera`, with the two
   equations that make each other (Faraday, and Ampère with Maxwell's term).
8. **What ε₀ and μ₀ are** — two bench experiments: two charges pushing, two wires
   pulling. Each constant is how hard.
9. **The speed** — neither experiment has light in it, yet 1/√(μ₀ε₀) is the speed
   of light.

## What it teaches

**Equations are LaTeX, and arrive when their discovery does.** Each is a
`scene.formula`, written on with `reveal`. Maxwell's own term is a *separate*
formula (`MAXWELL_TERM`), so scene 1 can hold it back and scene 6 can write it on
alone, in orange. `side_by_side` sets two formulas end to end by measuring them
with `cm.measure_math`.

**Use `∂`, not `\partial`.** The LaTeX bridge turns `\partial` into a Typst
symbol that Typst 0.15 does not have, so the equation fails to typeset. The
Unicode `∂` passes straight through. `tests/test_maxwell.py` typesets every
formula in the film, which is how this was found.

**A picture driven by one number.** Each scene is a function of `u`, how far
through it is. The compass needle follows a damped swing, the meter needle follows
the *speed* of the magnet rather than where it is, and the E arrows in the
capacitor grow with the charge. `ramp(u, a, b)` is the only timing tool, and a
scene lasts as long as its caption takes to read.

**One moment per tick.** The film runs at 30 fps and emits a moment every frame,
so nothing is interpolated between two draw orders.

## Check before trusting

Dates and numbers (1820, 1831, 3.1 × 10⁸ m/s from Weber and Kohlrausch 1856,
3.15 × 10⁸ m/s from Fizeau 1849) were checked against published sources, and the Khmer captions are a first draft. Check both before the film
states them. Scenes were checked as still frames, not as video.

## Things to try

- Narrate it as `year/speak.py` does; the caption already paces to reading.
- Scene 8 could show the speed shrinking in glass by raising `ε₀`.
