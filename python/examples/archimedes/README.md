# Archimedes' Principle — one box, four materials

```bash
.venv/bin/python python/examples/archimedes/main.py             # archimedes.mp4
.venv/bin/python python/examples/archimedes/main.py km          # archimedes-km.mp4
.venv/bin/python python/examples/archimedes/main.py km --clean  # ...-km-clean.mp4
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
| 10% of the iron left | what a 5px wall holds, over the box's area |
| average density 783 kg/m³ | iron, times the fraction of it still there |
| 78.3% draft | that density over water's |
| lift-off at 92% | where the average first drops under 1,000 |

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

**The box is closed.** An open one was drawn first, and it cost an
idealisation: a vessel open at the top, held under water, would fill, and the
film had to take the air inside it for granted. A sealed box of thin iron
around air simply floats, and there is nothing to excuse.

It is also simpler to draw. The outline never changes — not its size and not
its shape — because hollowing a closed box cuts no notch in it. The cavity is
a second shape drawn inside, and the walls are what is left showing between
the two. A polygon cannot have a hole in it, and it turns out not to need one.

**The hollowing happens at the surface.** It used to happen on the tank floor,
where the change was hard to see and the finished box sat in the dark. The box
is at the waterline now, its top exactly on the surface, and the walls thin
there. Getting it there is a **cut** — nothing lifts it, because a solid iron
box cannot come back up on its own and the film does not pretend otherwise. It
starts the next thought somewhere else, the way it moves between any two
scenes. (The alternative was the yellow hand from section 5, lifting it and
then being needed less and less until it lets go. That is more honest and it
is one more scene.)

**Nothing is told to rise.** `settles` is asked where a body with that outside
belongs, and the answer changes from "the floor" to "floating" the moment the
outside is big enough to carry the metal. The lift-off is a consequence. It
also happens late — always at 86% of the final area, since that is where
`rho_water x outside` passes `rho_steel x V` — and the equilibrium *jumps* 213
pixels when it crosses, so the last of the opening and the rise are walked
together rather than one after the other. `_the_body_finds_its_own_level`
checks that it leaves the floor exactly once and never sinks back.

## The template

Four bands down the frame, and nothing crosses between them. Every `y` in
`main.py` is one of these or is derived from `W.TANK`; a scene that wants to
put something somewhere puts it in a band rather than picking a number.

| band | y | what lives there |
| --- | --- | --- |
| `TITLE_Y` | 26 | the scene's name — what it is *for* |
| `BANNER_Y` | 110 | the one thing this scene is shouting: a big number, or the law |
| the stage | 150 | the tank and everything in it, `W.TANK` |
| `SAY_Y` | 636 | the caption on its plate — what is *happening* |

The notes column, to the right of the tank, is the only thing outside them.
Before there were bands, the `91.7% SUBMERGED` readout was moved three times —
into the water, onto the caption, and onto the floating box — because each new
tank size moved something it had been dodging.

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

## The caption runs itself

The subtitle sits on a rounded plate sized to the line it holds, and a mark
runs along it a word at a time. Both are lifted from the `attention` example,
which put it best: a plate of fixed width would look like chrome rather than
like the narration having a place of its own.

Three things make it work.

**Each piece is its own item, keyed by its own text.** A word *arrives*, rather
than one long string swapping its contents. The plate is keyed by the whole
line for the same reason — keyed by place it tweens to the next line's width
while the words change instantly, and for a quarter of a second the line hangs
off both ends of its own plate.

**The mark is a colour, not a plate.** A highlight rectangle was tried first
and is wrong for Khmer: its words are set flush, so a highlight has no gap of
its own to sit in and ends up under its neighbours. The word the line has got
to is simply the bright one, and the rest are dim.

**Khmer does not put spaces between its words.** That is the rule the first
attempt broke — it gave every segmenter boundary a space, which is Khmer with
the spacing of English. A boundary the segmenter found is invisible and stays
invisible; only a space the author actually typed becomes a space.

**Khmer still needs to be told where its words are.** Finding out needs a
dictionary and a Viterbi search — far too much to carry into a render. So
`segment.py` does it once, at authoring time, and writes the answer into the
line as U+200B ZERO WIDTH SPACE, the character Khmer already uses for a word
boundary. The video needs no segmenter and no model: the text arrives knowing
where its own words end, and `chunks()` splits on it. A line that has never
been through the segmenter still runs, one orthographic cluster at a time —
choppier, but not broken.

**Reading sets the length of the scene, not the other way round.** A section
lasts exactly as long as its own caption takes to read: a word costs a fixed
moment plus a little per letter, and a clause ending costs a rest. There is no
hand-tuned hold any more — a line of four words is a short scene and a line of
seventeen is a long one, which is the only pacing a viewer actually feels.
`reads()` registers each word's own duration under its own event name, because
`Timing` looks a duration up by name.

The rate is about eleven characters a second, the middle of the range
broadcast subtitles use. It is the single number that sets the film's length:
at this rate the English film runs 2:57, and dropping to sixteen characters a
second would bring it to about 2:25.

**A repeated line is held, not read again.** If the caption has not changed
since the last scene it stays on screen, whole and bright, while the picture
carries on. The derivation shows four algebra steps under one sentence and
reads it once; the Khmer scenes that share a line stop stuttering it.

**No two lines are ever on screen together.** The old caption leaves during
the change beat and the new one arrives after it, with its first word already
bright. Left to overlap, a subtitle reads as handed over from the scene before
rather than belonging to this one — and for a quarter second both are legible
through each other.

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

## The voice

```bash
python python/examples/archimedes/narrate.py           # what would be said
KIRI_API_KEY=... python .../narrate.py --write         # record the missing
.venv/bin/python python/examples/archimedes/main.py km # render, writing cues
python python/examples/archimedes/mix.py               # lay the voice on
```

The Khmer film is narrated; the English one is silent and keeps the reading
rate. Kiri's voices are all Khmer, and the Khmer version is the one whose
audience most needs to hear it rather than read while watching the tank.

**Speech sets the length of the scene.** A reading rate is a guess; a
recording is a fact. Where a caption has been spoken, its section becomes the
recording plus half a second, and the words keep their proportions inside it
so the mark is on the word being said rather than near it. Without the audio
the film still runs, at the reading rate — the voice is an addition, never
something the film depends on.

**Where the mark goes is measured, not guessed.** Each recording is sent back
through Kiri, which returns the words it heard with millisecond timings, and
those are lined up against the caption the recording was made from.
`align.py` writes `audio/timing.json`: for each caption, the moment each of its
words is said, counted from the start of **its own clip**.

From the clip's start and not the film's — that is the whole design. An SRT of
the finished film was tried first, and it goes stale the moment anything is
re-timed: the transcript to hand was of an earlier cut and its clock had
drifted by up to twenty-two seconds. A clip's own timings never drift, because
the film places the clip and the words follow it.

Three things that had to be survived:

- **The transcript is not the script.** It is what a machine heard — `ដូច្នេះវា`
  came back as `មិញនេះ វាគ` — so words are matched by *character*, letting the
  long agreeing runs carry the alignment, and a word it never found is placed
  between the two it did. `custom_vocabulary` biases it toward the spellings
  the captions actually use.
- **A clip opens on a breath.** The first word is often a third of a second
  in, so nothing is marked until the voice arrives; otherwise the first word
  lights before it is spoken.
- **Transcribing is not free.** Every answer is cached under `audio/heard/`,
  so re-running after editing one caption costs one request.

All 21 lines are timed this way, none left to the guess — though the guess is
still there, and a caption with no recording still runs on it.

**The cues come from the same walk as the picture.** `main.py` keeps a running
clock through every emit and writes `audio/cues.json` as it renders, so the
sound cannot disagree with the picture: both were walked from the same events
in the same order. 21 lines, 86.6 seconds of speech, and no two overlap.

**Only subtitles are spoken.** Titles are labels on the picture, not
narration, and a line the film *holds* rather than re-reads is recorded once.

**The words and the voice are separate switches.** Every combination of them
is a film somebody wants:

| flag | picture | words | voice |
| --- | --- | --- | --- |
| *(none)* | yes | yes | yes |
| `--no-subtitle` | yes | — | yes |
| `--silent` | yes | yes | — |
| `--clean` | yes | — | — |

The timing is the same in all four, so the cuts are frame for frame each other
with one thing or another taken off. That is what makes them worth having:
dub your own voice over a picture whose pauses are already the right length,
show the narrated film to someone who does not want to read, or watch the tank
make its own case without being told.

`cues.json` carries the name of the cut it was walked for, so `mix.py` lays the
voice onto that one. Laying it onto the wrong cut would sound right while
showing the other film.

The recordings are not committed — 8MB from a paid API, and not ours to
publish. `narrate.py` caches by a hash of the text, the voice and the speed,
so editing one caption later costs one request rather than twenty-one.

## The shape of it

| file | what it knows |
| --- | --- |
| `world.py` | densities, depths, the hull solve, the arrow scale. No Codimate; checks itself. |
| `vocabulary.py` | every scene's title and subtitle, in English and Khmer, and the unit. No Codimate; checks itself. |
| `segment.py` | marks Khmer word boundaries in `vocabulary.py`. Authoring-time only. |
| `narrate.py` | speaks the Khmer captions with Kiri TTS, and measures them. Authoring-time only. |
| `align.py` | reads a transcript of the narration and learns how fast each word is said. Authoring-time only. |
| `mix.py` | lays the recordings onto the rendered video with ffmpeg. |
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
