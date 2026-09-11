# The helical model — and what the popular video gets wrong

```bash
.venv/bin/python python/examples/helical_solar_system/main.py
```

The Sun is not still. It carries the whole system through the galaxy at about
230 km/s, so relative to anything outside, a planet does not trace an ellipse —
it traces a helix around the Sun's path.

## What it teaches

**Two claims, and only one of them is true.** The widely-shared "helical model /
solar system is a vortex" video is right that the orbits are helices in a
galactic frame. It is wrong about the geometry in two ways, and this example
does both properly:

| the video | actually |
|---|---|
| the orbital plane is square to the direction of travel | it is inclined about **60°** to the galactic plane |
| planets trail behind the Sun like a comet's tail | **half of every orbit is ahead of it** |

Watch Venus fall behind while Earth runs out in front. That is the tell.

**The compression is stated, not hidden — and it decides whether you see
orbiting at all.** At true speeds the Sun covers 49 AU while Earth goes round
once, so one turn of Earth's helix would be 24 times longer than it is wide: a
straight line with a faint ripple.

How far you compress it is not a free choice. A coil only closes into a *loop*
when its pitch — travel × the planet's period — is shorter than its orbit's
diameter. Otherwise it stretches into a wave and the planets look like they are
trailing away rather than going round:

| travel | Earth: pitch / diameter | Mars: pitch / diameter |
|---|---|---|
| 2.6 AU/yr | 2.60 / 2.0 — **no** | 4.89 / 3.0 — **no** |
| 1.8 | 1.80 / 2.0 — yes | 3.39 / 3.0 — no |
| **1.2** | **1.20 / 2.0 — yes** | **2.26 / 3.0 — yes** |

So `TRAVEL = 1.2`, a 41× compression, and both visibly go round. `TRAIL` is
1.3 years for the same reason: shorter than an orbit and a planet never
completes a turn on screen.

**The camera was chosen by searching, not by eye.** Two things fight: a view
along the Sun's path collapses the orbits to a line, and a view down the
orbital normal hides the travel. Sweeping azimuth and elevation for the pair
that keeps both readable lands on azimuth 120°, elevation −60° — travel running
down-right at 27°, orbital axes projecting to 0.90 and 0.98, so the orbit stays
nearly circular rather than edge-on.

**The camera travels with the Sun**, which is the frame the helix is most
legible in: the Sun sits still in the middle, planets wind around it, trails
stream away behind.

**So the sky must stream the other way.** A fixed starfield would be painted
on, and would quietly say the Sun is standing still — the one thing this is
arguing against. Each star gives back a share of the camera's travel, and a
nearer star gives back more, which is parallax:

| star | drift over the 2.8 years shown |
|---|---|
| nearest | 238px |
| middling | 133px |
| furthest | 27px |

A real star's parallax would be a millionth of that, but the Sun's travel is
compressed 18× here too, so the sky is compressed with it. What matters is that
the drift varies with depth rather than sliding as one sheet.

**A trail is a record, not a thing.** Each segment is named after the tick it
records, so it is fixed the moment it is drawn. New ones fade in at the head,
the oldest fade out at the tail, and the Engine needs no more instruction than
that.

```text
space.py    the measurements, the camera, and how 3D reaches a flat screen
orbits.py   where every body is, tick by tick — no Codimate in it
view.py     the drawing
main.py     the four pieces
```

## Try changing

| Change | What happens |
|---|---|
| `INCLINATION = 90°` | the video's geometry: orbits square to the travel |
| `TRAVEL = 49.0` | the true speed ratio — a straight line with a ripple, which is why nobody draws it |
| `TRAIL = 200` | long helices that overlap into the vortex look |
| `AXIS_*` from another azimuth/elevation | the orbits collapse edge-on, or the travel vanishes |
