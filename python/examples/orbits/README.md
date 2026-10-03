# Orbits — the science kit, end to end

```bash
.venv/bin/python python/examples/orbits/main.py
```

Close on the Earth and its Moon, then the camera backs away and pans to the Sun
until a whole year is one ellipse. Thirteen seconds, and every picture is built
from [`codimate.science`](../../../docs/reference.md#the-science-kit): a `Camera`,
three `Sphere`s, two `Orbit`s, a `World` that draws them together, `RingArrows` so
a turning ball can be seen to turn, and `tag`s kept clear of one another.

## What it teaches

**The kit hands back pixels, and the film is still ordinary Codimate.** The trace
holds four numbers — how far the pull-back has got, the year, the month, the spin —
and nothing else. `view` turns them into a `Camera` and some `Sphere`s and asks a
`World` to draw. Nothing moves itself; the trace says where everything *is*, one
moment at a time, which is the only way anything moves here.

**One moment per frame.** `Timing(default=1 / FPS)`, rendered at `FPS`. Draw order
is resolved once per segment, so a ball turning faster than the moments arrive
re-sorts its faces in jumps. This film did it wrong first — twenty moments a second
at thirty frames — and a ring seemed to cut straight through a planet. No single
moment ever showed that: it was two valid orderings blended between frames.

**A pull-back is `dolly`, not `moved`.** `dolly(distance)` keeps the lens, so
things shrink as the camera backs off. `moved(distance=...)` keeps their size and
changes only the perspective. The film wants the first.

**The Earth is not drawn as a circle on a path.** `Orbit` is an ellipse with the Sun
at a focus, and `point(angle)` takes mean anomaly, so the year is not uniform: Earth
quickens near the Sun with no code to say so. The Earth's axis is tilted and *stays*
tilted as it goes round, which is the whole of the seasons.

## Things that were wrong first

**Names that follow their bodies collide.** A name stands over its body, and the
Moon passes above the Earth, so the two plates meet — as do the Earth's ring arrows
and the Moon's name. A fixed offset cannot fix it, because the bodies move. Two
things did: the names sit above the arrows (`layer=science.ABOVE + 10`), and
`clear_of` moves a name the shortest way out of the ones before it, sliding rather
than jumping. A fixed legend with a line to each body also works and never
collides; this film does not use one because a name over its object reads faster.

**The night side was black.** The Sun is the light, so the lit side follows the
Earth round — and this camera mostly looks at the side facing away. `ambient=0.3`
keeps it readable; the default of 0.12 is right for a lit subject.

**A ring that crosses a planet's face.** An orbit really does pass in front of its
planet at some angles, and drawn there it looks broken. `hide=("earth",)` cuts the
path wherever the planet's silhouette would be, which is a decision about how it
looks rather than a claim about what is in front.

## The shape of it

| file | what it knows |
| --- | --- |
| `main.py` | the trace and the view. Everything else is `codimate.science`. |

## What this is not

To scale. The Sun is 1.4 times the Earth, not 109; the Moon is at 2.6 Earth radii,
not 60. True proportions would make the Earth and Moon invisible, so the Moon is the
right *size* beside the Earth and the distances are compressed — which is what every
diagram of this has to do, and is better said than hidden.

A world is a few thousand named shapes a frame, and the previewer's index records
every shape of every moment: 470 MB for this film, against a 1.5 MB video. `render`
skips it for a film like this and says so; preview `main.py` itself and the
previewer builds the index in memory. `index=True` writes the file anyway.
