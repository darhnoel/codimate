# Archimedes' Principle — one box, four materials

```bash
.venv/bin/python python/examples/archimedes/main.py        # results/archimedes.mp4
.venv/bin/python python/examples/archimedes/main.py km     # results/archimedes-km.mp4
```

One box of volume V, four times over: water, ice, solid iron, and the same
iron with its walls thinned until most of what is inside its outside is air.
Ice settles at 91.7% under, the derivation arrives *after* that number rather
than before it, and the iron box floats without ever changing size.

## What it teaches

**One name, and one size, the whole film.** `"body"` is a box of water, then
ice, then solid iron, then hollow iron. It is never replaced and never
resized — only its contents change — so the Engine tweens between them, and a
name is what moves. Every shape it takes is sampled to exactly `POINTS` points,
because two polygons only interpolate when their point counts match (ADR 0010):
a hollow box with eight corners and a solid one with four still correspond
point for point when both are walked by distance. The corner rounding lives
inside that walk for the same reason — a rounded rect could not morph.

**One formula for every object.** `density(rho, wall)` is the average over the
box's outside. At `SOLID` the walls meet in the middle, the metal is the whole
volume, and it gives back exactly the density of the stuff — 1,000 for water,
917 for ice, 7,850 for iron. Thin the walls and it falls, because most of what
is inside the outside is now air. That is the entire last section, and it is
the same line of code the first section runs on.

**The picture is solved, not drawn.** Nothing in `world.py` is placed by eye
except the two pools and the box:

| number | where it comes from |
| --- | --- |
| 91.7% submerged | `917 / 1000`, not typed anywhere |
| 8.6% of the iron left | what a 6px wall holds, over the box's area |
| average density 672 kg/m³ | iron, times the fraction of it still there |
| 67.2% draft | that density over water's |
| lift-off at 90% | where the average first drops under 1,000 |

The ship floating is therefore a consequence of the drawing, not a figure
chosen to make the point come out. `world.py` checks it: at every floating
depth the push up equals the weight down to a part in 10¹², the hull holds the
block's steel to a fraction of a pixel, and the ship both fits the glass and
does not push the water over the rim.

**Depth and waterline answer each other.** Sinking the object raises the
surface, and the higher surface swallows more of the object. Driving the level
directly would let the two drift apart the moment anything moved, so `surface()`
solves the one equation instead:

    L = REST - w (bottom - L) / WIDTH

**One scale for every arrow.** `arrow_length` converts force to pixels once for
the whole film. Steel genuinely runs off the end at that scale, so the arrow is
capped and marked with an axis break — the alternative is rescaling between
scenes, which would quietly make steel look no heavier than ice.

## Things that were wrong first

**The rise you cannot see.** A box of volume V raises a tank of width W by
`V / W` — 46 pixels in the beaker, which still reads as very little. Two fixes were tried and rejected:
a second beaker catching the overflow (which never fills, because the tank has
far more headroom than the box has volume), and a brim-full overflow can (which
fills, but then the level cannot rise, so there is nothing to mark). A closed
tank with a dashed "before" line and a bracket keeps both readings. The payoff
is the ship, whose 189px rise needs no help at all.

**A bracket that lied by staying still.** It read *water displaced = V* in every
scene, but only the fully sunk water box displaces V: the floating ice displaces
0.917 V and the ship 7.85 V. It now reads the multiple off the geometry, which
also makes it the same fact as the 91.7% on the other side of the tank.

**The ship, and why it is gone.** The film used to end with a hull: the same
steel spread nine times wider until steel and air together came out lighter
than water. It is the right physics and it wrecked everything around it. A hull
nine times the box needs a tank sized for the hull, and in a tank that size the
box is a quarter of the width and lost in water it never reaches. Two pools
were tried — a beaker the box filled, growing into a basin for the ship — and
that was worse still, because **a container cannot inflate**.

Three impossible things went with the ship: the glass stretching, the sunk
block rising off the floor with nothing to lift it, and a solid steel slab
balanced on the waterline while it widened to nine times its own area.

The box that floats is the same box. Nothing in the film is ever bigger than V,
so the tank is small and the box fills nearly half of it — and the last section
became the most honest part of the film rather than the least.

**What it costs, said plainly.** Iron is 7.85 times water, so a box of iron can
only float once all but an eighth of the iron is gone. The hollow box holds
8.6% of what the solid one held. "Not one gram added or taken away" was the
ship's claim and it is not this one: here the claim is that the *outside* never
changed, and the average over that outside is what decides.

**The label under the arrow.** The force arrows run through the object's centre
and so does its name, so `WATER` / `ICE` / `STEEL` sat under the shaft. A dark
plate behind the text fixes it only if the plate is on a *higher* layer than
the arrows — on the same layer the plate drew first and the arrow straight over
it, which is the whole thing the plate exists to prevent.

**A label that landed on the caption.** The weight arrow's label was below its
tip, and for steel the tip is near the floor of the tank — so the label went
through the caption. Labels are always above the tip now, and the arrows are
switched off before the block drops the last stretch to the floor.

**Spreading and hollowing are one motion, not two.** They were two, and that
was a lie the section was telling about itself: a *solid* block cannot simply
widen, because that multiplies the metal ninefold — the one thing the section
claims does not happen. So `thickness` solves

    w h - (w - 2t)(h - t) = V

at every instant instead of once for the finished hull. The walls thin as the
outside grows, starting from a block where they meet in the middle and there is
no cavity at all. `_the_reshape_conserves_steel` checks all forty steps.

**Nothing is told to rise.** `settles` is asked where a body with that outside
belongs, and the answer changes from "the floor" to "floating" the moment the
outside is big enough to carry the metal. The lift-off is a consequence. It
also happens late — always at 86% of the final area, since that is where
`rho_water x outside` passes `rho_steel x V` — and the equilibrium *jumps* 213
pixels when it crosses, so the last of the opening and the rise are walked
together rather than one after the other. `_the_body_finds_its_own_level`
checks that it leaves the floor exactly once and never sinks back.

## Title and subtitle have different jobs

Every scene has both, and neither does the other's work. The **title** names
what the scene is *for* — a few words, holding still while the scene plays.
The **subtitle** describes what is happening in front of you, and changes as it
happens. A sentence in the title slot is a subtitle that got lost.

That is enforced rather than remembered, because it has come apart twice —
once when the titles grew into sentences, and again when a translation's own
grouping of its lines was mapped straight into the slots. `beat()` takes the
scene's *name* and reads both lines out of the vocabulary, so a beat cannot
quietly acquire a sentence for a title or a title with nothing under it, and
`vocabulary.py` refuses any scene with a blank slot or a title over 40
characters.

`KM_SCENES` holds no text of its own — every entry points at a line in `KM`,
which is the translator's, or at one in `DRAFT`, which is not. `untranslated()`
names the scenes still carrying a draft, so "which of these words are mine" has
an answer without reading two languages side by side.

Both lines are shrunk to fit at draw time rather than trusted to a size chosen
in advance, because `text` has no newlines to wrap at — and, without a
dictionary, nowhere safe to break a script that puts no spaces between its
words.

## Units are mathematics, not words

`kg/m³` is typeset with `scene.formula` everywhere it appears — the swatches,
the chart axis, the ship's average density. Spelt out it has to be abbreviated
differently in every language and the exponent stops being an exponent:
`គ.ក./ម៉.គូប` says the same thing as `kg/m3` only to someone who already knows
which it is. Typeset, it is the same symbol in both films.

The one place this costs something is layout. A formula is typeset by Typst
when the video is built, so there is nothing to `cm.measure` — the chart's axis
label puts the word on one side of the water line and the quantity on the
other rather than trying to set them end to end.

## Why two beats per section

A shape entering or leaving a Scene fades, and the fade takes the **whole**
beat. A panel appearing at the top of a six-second section spends six seconds
arriving. So every section is a short `swap` beat (0.26s) that changes what is
shown, followed by a long one where nothing changes and the picture simply
sits. Continuous motion is the other half of the same rule: `walk()` hands each
changing value over a step at a time — depth, outline, and the five numbers of
the pool alike — because a value set once tweens in a straight line, and the
object would slide while the water it displaces jumped.

## The shape of it

| file | what it knows |
| --- | --- |
| `world.py` | densities, depths, the hull solve, the arrow scale. No Codimate; checks itself. |
| `vocabulary.py` | every scene's title and subtitle, in English and Khmer, and the unit. No Codimate; checks itself. |
| `main.py` | the trace and the view. |

## The Khmer one

Same film, same physics, same layout — only the words fork, so a fix to the
picture cannot land in one language and not the other. `vocabulary.py` checks
on import that neither language has a key the other lacks, and that no
translation lost a `{placeholder}`: a hole would otherwise surface minutes into
a render, as a caption printing its own template.

Two things are worth knowing if you add a third language.

**Almost nothing in the view needed changing.** `cm.measure` goes through the
engine's real fonts including fallback, so every label plate, bracket and chart
row sizes itself around Khmer exactly as it does around ASCII. Khmer sentences
do run much longer, but that is handled by shrinking a line to fit at draw time
rather than by giving each language its own sizes to keep in step.

**The title card had to learn what a character is.** It types on, and typing
by code point tears Khmer apart: a vowel sign is its own code point, and COENG
(U+17D2) turns the letter *after* it into a subscript. Split those off their
base and the shaper is handed fragments — `គោលការណ៍` types out as `គ លេក រណអ៍`.
So `clusters()` groups a base with everything that belongs to it, and the card
is revealed a cluster at a time. The prefix is then drawn as **one growing
shape** rather than one shape per character, with only its left edge pinned:
measuring characters and placing them individually would drop the shaping
*between* clusters, which for a script with ligatures is not the same text.

Numerals stay Western in both. A chart with `១,០០០` on its axis and `1000` in
the algebra beside it would be two different claims to read.

## What to try changing

- `RHO_ICE`. Everything follows — the depth, the bracket, the percentage in the
  payoff, the summary table. Nothing needs editing twice.
- `HULL_W`, `HULL_H`. The wall thickness re-solves to keep the steel constant,
  and the ship floats higher or lower on its own. Shrink it far enough and the
  assertions stop you before the render does.
- `TANK` and `REST_LEVEL`. Every depth, waterline and bracket follows, and the
  assertions will tell you before the render does if the ship stops fitting.
