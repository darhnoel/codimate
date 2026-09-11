# Bernoulli's principle — and the story that goes with it

```bash
.venv/bin/python python/examples/bernoulli_lift/main.py
```

Air flows past a wing. Parcels are coloured by speed, and two marked ones are
released side by side at the nose — one destined to go over, one under.

## What it teaches

**Half of the familiar diagram is right.** The air over the top really is
faster, and faster really does mean lower pressure. Measured on the exact
solution, at mid-chord:

| | speed | pressure coefficient |
|---|---|---|
| above | 1.34 | **−0.80** |
| below | 0.82 | **+0.33** |

Low above, high below — a net force upwards. Bernoulli is fine.

**The reason usually given is not.** "The air over the top has further to go,
so it must speed up to meet its partner at the back" requires the two halves to
arrive together. They do not:

```
over  the wing: 5.79
under the wing: 7.80     the upper one arrived 1.35x sooner
```

The upper parcel is long gone by the time the lower one reaches the trailing
edge. Nothing makes them meet, and nothing ever did. If path length were the
cause, a symmetric wing could not lift and no aircraft could fly inverted.

**Where lift actually comes from.** The circulation — fixed by the Kutta
condition, which requires the flow to leave the sharp trailing edge smoothly
rather than whipping around it. That is the one free constant in the solution,
and setting it is what produces the speed difference, not the other way round.

**And the half the diagram leaves out.** The wing turns the air **19°
downwards**: it arrives at +17.6° and leaves at −2.5°. It throws air down, so
the air throws it up. Newton's third law explains the same lift, and mentions
path length just as little.

## The flow is not drawn

It is the exact potential-flow solution for a Joukowski airfoil, known since
1910: a circle in one complex plane maps to a wing in another, the flow around
a circle can be written down, and the map carries it across. `airfoil.py` is
forty lines of complex arithmetic and imports nothing but `cmath`.

So the streamlines, the speeds, the pressures and the transit times are all
consequences rather than choices. The only thing chosen is where to release the
parcels.

**The dividing streamline was measured, not assumed.** It is not at y = 0 — the
circulation pulls it up from well below, and *where* you measure it matters: it
passes y = −0.95 at the release line and y = −0.61 by x = −3. Releasing the
marked pair either side of the wrong value puts both of them over the wing and
the demonstration silently shows nothing.

**The air comes in level, and the wing sits nose-up in it.** The maths puts
the angle of attack into the free stream and leaves the wing horizontal, which
draws air arriving uphill. The picture is rotated back by the same angle — the
view everyone means, and the one an aircraft has. Nothing physical changes;
only which of the two you are standing still relative to.

What is left is real: the flow still meets the wing tilted *upwards* at the
nose, because the wing pulls air up to meet it before turning it down. It
levels off with distance, which is the check that the rotation is right and the
tilt is not:

| upstream | flow angle on screen |
|---|---|
| x = −4 | +8.6° — upwash |
| x = −20 | +1.3° |
| x = −300 | +0.1° — level |

**The band of air shown was solved for, not guessed.** Each limit comes from
tracing that streamline and asking where it reaches on the screen, so the flow
stays clear of the title above and the captions below. Nothing is clamped: no
air crosses a streamline, so a parcel released inside the band stays inside it.
Measured over a whole run, every parcel stayed within screen y 125–596, against
a title at 48 and captions at 606.

**The wing is filled a column at a time.** A circle can be filled and a
rectangle can be filled, but the Authoring Surface has no polygon, so the wing
is rasterised into 140 vertical spans — the same trick as `dharma_wheel`
building rings out of discs.

```text
airfoil.py       the wing and the exact velocity field
flow.py          parcels carried along it
scene_setup.py   where it sits on screen, and the speed colours
view.py          the drawing
```

## Try changing

| Change | What happens |
|---|---|
| `ALPHA = 0°` with `CENTRE.imag = 0` | a symmetric wing at zero incidence: circulation goes to zero, and so does lift |
| `ALPHA = -7°` | flying inverted — still lifts, which path length cannot explain |
| `CENTRE = -0.10 + 0j` | symmetric, no camber, lift only from angle of attack |
| `MARK_GAP = 0.4` | the marked pair released far apart, so they never split at the nose |
