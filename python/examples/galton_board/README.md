# Galton board — where the bell curve comes from

```bash
.venv/bin/python python/examples/galton_board/main.py
```

Four hundred balls are dropped through a triangle of pegs. At each peg a ball goes
left or right with even chance, and lands in the bin counting how many times it
went right. Nothing aims for a bell curve — it is what evenly-weighted coin
flips add up to.

## What it teaches

**Both identity modes, in one picture.** This is the example where the choice
that `bubble_sort` and `neural_net` each show separately appears side by side:

| | named after | because |
|---|---|---|
| pegs | their place — `("peg", row, i)` | they never move |
| bins | their place — `("bin", k)` | they grow, they do not travel |
| balls | the ball — `("ball", id)` | each falls its own path |

Get one of those wrong and the video is wrong in a specific way. Name the balls
after their slot and they would stop falling — each slot would just blink on
and off as different balls passed through it.

**The falling is simulated, not faked.** Balls accelerate under gravity, lose
half their downward speed to every peg they strike, and travel a parabola from
one peg top to the next. The sideways speed a peg imparts is whatever lands the
ball exactly one slot over by the time gravity has carried it down a row — so
the path stays ballistic while the lattice stays exact, and the statistics stay
binomial.

The consequence worth noticing is in the **trace**, not the motion. It samples
at a fixed time step, not one event per row:

```python
while not board.done():
    board.step()        # advances every ball by DT of real time
    cm.emit("tick")
```

Stepping row by row would give every ball the same duration per row, which
means a constant descent rate no matter how the motion between rows is drawn.
Sampling time instead means a ball near the bottom, moving faster, simply
covers more ground per tick. The physics settles into a steady rate — 0.27s for
the first row, 0.16s after — because what gravity adds each row is what the
next peg takes away. Real Galton balls do the same.

**Many things moving at once, independently.** Eight or nine balls are in
flight at any moment, each at a different depth on a different path. The view
says nothing about falling:

```python
for ball_id, level, rights in board.flying:
    scene.circle(("ball", ball_id), x=slot_x(level, rights), y=level_y(level), r=9.0)
```

The trace records where every ball *is* at each tick. Everything between ticks
is derived, for all of them at once, with no more work from the author than
one ball would take.

**A ball that lands just stops being named.** There is no "remove" call. Its
name is absent from the next moment, so the Engine fades it out — and the bin
it landed in is a different shape that simply got taller.

**`linear`, not the default.** The parabola is already in the samples, because
the simulation put it there. Easing between them would add a wobble the physics
never asked for. Same reasoning as `dharma_wheel`: ease when each event is a
discrete step, go linear when a thing is mid-journey at every one.

## Why 400 balls and not 80

Whether the histogram *looks* like a bell is arithmetic, not styling, and two
things set it:

**How many bins.** Six rows give seven bins, and seven bars can only ever be a
coarse bell — the ideal shape is `1.6 · 9.4 · 23.4 · 31.2 · 23.4 · 9.4 · 1.6`
percent. More rows give finer resolution and a denser thicket of pegs; this
example keeps the pegs sparse and buys the shape elsewhere.

**How much noise.** The tallest bar wobbles by about `sqrt(n·p·(1-p))`:

| balls | tallest bar | wobble |
|---|---|---|
| 80 | 25 | ±4.1 — **17%**, which is why it looked lumpy |
| 200 | 62 | ±6.6 — 10% |
| **400** | **125** | **±9.3 — 7%** |

So: same pegs, five times the balls. The grey line over the bars is the
binomial the coin flips predict, drawn so the match is something you can see
rather than something the caption claims.

Bar heights are scaled so the *expected* peak fills the space
(`BIN_UNIT = TALLEST_BAR / max(EXPECTED)`), fixed for the whole render. Bars
that rescaled as counts arrived would animate a lie.

## Reproducible on purpose

`SEED = 3` is fixed, so the video is identical every render — otherwise it
would be a different video each time, which makes it useless to compare against
when you change the engine.

The simulation is unbiased: at 4000 balls the histogram fits the binomial with
a chi-square of 7.9 on 8 degrees of freedom, about as close to expected as you
could ask for.

## Try changing

| Change | What happens |
|---|---|
| `ROWS = 12` | a finer, smoother curve — and a denser, wider board |
| `BALLS = 80` | the noisy version — the shape is there but lumpy |
| `self.rng.random() < 0.65` | a biased peg: the whole curve slides right |
| `BOUNCE = 0.9` | pegs barely slow the balls; they accelerate the whole way down |
| `GRAVITY = 400` | a slow, lunar drop |
| name balls `("ball", level, rights)` | balls stop falling and start blinking |
