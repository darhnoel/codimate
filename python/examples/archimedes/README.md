# Archimedes' Principle — one box, four materials

```bash
.venv/bin/python python/examples/archimedes/main.py
```

A minute and a half, from a steel block on the bottom of a tank to a steel ship
floating in the same water. Ice settles at 91.7% under, the derivation arrives
*after* that number rather than before it, and the ship is the same steel
spread twelve times wider.

## What it teaches

**One name, the whole film.** `"body"` is a box of water, then ice, then steel,
then a ship's hull. It is never replaced, only changed, so the Engine tweens
between them — a name is what moves. That is also why the box and the hull are
both sampled to exactly `POINTS` points: two polygons only interpolate when
their point counts match (ADR 0010), so a hull with eight corners and a box
with four still correspond point for point when both are walked by distance.

**The picture is solved, not drawn.** Nothing in `world.py` is placed by eye
except the tank and the box:

| number | where it comes from |
| --- | --- |
| 91.7% submerged | `917 / 1000`, not typed anywhere |
| hull wall 11.86px | solved so `wh - (w-2t)(h-t)` equals the block's area exactly |
| average density 654 kg/m³ | steel over the hull's *outer* volume |
| 65.4% draft | that density over water's |
| spread 12× | outer area over block area |

The ship floating is therefore a consequence of the drawing, not a figure
chosen to make the point come out. `world.py` checks it: at every floating
depth the push up equals the weight down to a part in 10¹², the hull holds the
block's steel to a fraction of a pixel, and the ship both fits the glass and
does not push the water over the rim.

**Depth and waterline answer each other.** Sinking the object raises the
surface, and the higher surface swallows more of the object. Driving the level
directly would let the two drift apart the moment anything moved, so `surface()`
solves the one equation instead:

    L = REST - w (bottom - L) / TANK_W

**One scale for every arrow.** `arrow_length` converts force to pixels once for
the whole film. Steel genuinely runs off the end at that scale, so the arrow is
capped and marked with an axis break — the alternative is rescaling between
scenes, which would quietly make steel look no heavier than ice.

## Things that were wrong first

**The rise you cannot see.** A box of volume V raises a tank of width W by
`V / W` — here 21 pixels, which reads as nothing. Two fixes were tried and
rejected: a second beaker catching the overflow (which never fills, because the
tank had 121,520px² of headroom against 13,000px² of displacement), and a
brim-full overflow can (which fills, but then the level cannot rise, so there
is nothing to mark). A closed tank with a dashed "before" line and a bracket
labelled *water displaced = V* keeps both readings. The payoff is the ship,
whose 164px rise needs no help at all.

**The label under the arrow.** The force arrows run through the object's centre
and so does its name, so `WATER` / `ICE` / `STEEL` sat under the shaft. A dark
plate behind the text fixes it only if the plate is on a *higher* layer than
the arrows — on the same layer the plate drew first and the arrow straight over
it, which is the whole thing the plate exists to prevent.

**A label that landed on the caption.** The weight arrow's label was below its
tip, and for steel the tip is near the floor of the tank — so the label went
through the caption. Labels are always above the tip now, and the arrows are
switched off before the block drops the last stretch to the floor.

**A morph inside a swap beat.** Box to hull happened in the 0.26s beat that
changes everything else at once, and read as a glitch rather than as the
answer. It has its own two-second beat.

## Why two beats per section

A shape entering or leaving a Scene fades, and the fade takes the **whole**
beat. A panel appearing at the top of a six-second section spends six seconds
arriving. So every section is a short `swap` beat (0.26s) that changes what is
shown, followed by a long one where nothing changes and the picture simply
sits. Continuous motion is the other half of the same rule: `glide()` hands the
depth over a step at a time, because a position set once tweens in a straight
line and the object would slide while the water it displaces jumped.

## The shape of it

| file | what it knows |
| --- | --- |
| `world.py` | densities, depths, the hull solve, the arrow scale. No Codimate; checks itself. |
| `main.py` | the trace and the view. |

## What to try changing

- `RHO_ICE`. Everything follows — the depth, the bracket, the percentage in the
  payoff, the summary table. Nothing needs editing twice.
- `HULL_W`, `HULL_H`. The wall thickness re-solves to keep the steel constant,
  and the ship floats higher or lower on its own. Shrink it far enough and the
  assertions stop you before the render does.
