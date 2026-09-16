# Rubik's cube — a drawing that turned out to be the cube

```bash
.venv/bin/python python/examples/rubiks_cube/main.py
```

A scrambled cube puts itself back, seven moves, and beside it the same cube
drawn as nine overlapping circles. Turn a layer and twelve dots slide three
places along one circle.

## What it teaches

**The drawing was not decoration, and the code proves it rather than assuming
it.** It arrived as a picture: fifty-four coloured dots, some lines, some
circles. Fitting circles through the dots found three centres with three radii
each, and then three facts that cannot be coincidence:

- every circle passes through exactly twelve places,
- every place sits on exactly two circles,
- walking any circle, the other two alternate in runs of three.

That is the cube's nine layers — three slices on each of three axes, twelve
stickers wrapped round each slice, every sticker in two of them. `places.py`
searches for the pairing that makes the drawing and the cube the same object,
finds forty-eight (the drawing's own symmetries), and keeps one. Then it
checks the thing it was after:

```python
# A quarter turn slides one circle's twelve exactly three places along.
moved = {(ring.index(PLACE[after[AT[p]]]) - ring.index(p)) % 12 for p in ring}
assert moved in ({3}, {9})
```

**Continuous motion is sampled, not guessed at.** A quarter turn is handed over
as nine 10-degree steps, the way `dharma_wheel` turns and `pendulum` swings. Ask
the reconciler to get from one picture to a picture ninety degrees later and it
will slide stickers across the screen; hand it ten degrees at a time and each
sticker rotates.

The pace is set by the drawing, not by the cube. Where the cube covers ten
degrees a dot covers a long arc of its circle, so a turn quick enough to look
right on the cube alone throws the dots across the page.

```python
for step in range(1, STEPS + 1):
    state["part"] = step / STEPS
    cm.emit("spin")
state["where"] = cube.turn(state["where"], face)
```

**Painter's order is three questions, and depth is the last of them.** The
first version sorted facelets by their resting depth, so half way through a
turn the layer swung out in front was still drawn behind and the cube came
apart. Sorting by the *turned* depth instead is better and still wrong: a
sticker's middle can come forward while the sticker is behind the face it
overlaps, and it lands on the cube like a sequin.

Two exact splits come first. Which side of the cube a sticker is on is its
normal's business. And the turning layer is cut from the two that stay by a
plane — so whichever side of it the camera is on is wholly in front of the
other, which is when painter's order is exact rather than a guess. Depth is
left to settle stickers *within* a group, where they share a plane and cannot
disagree.

**And nothing may enter or leave.** Not drawing the far side looks like the
obvious saving, and it is why the cube was full of half-transparent stickers:
a facelet crossing the horizon enters or leaves, and the reconciler fades
anything that does either. So all fifty-four are drawn every frame and the far
ones are painted the dark of the cube's inside — which is what you see down the
gap a turn opens, and which tiles the far side exactly. Which side a sticker is
on is its normal's business, not its depth's: the far corner of a face you are
looking straight at is further off than the middle of the cube and still
perfectly visible.

**A move turns twenty-one dots about two different middles.** The twelve on
the circle go round the circle's centre. The other nine are the face's own, and
they go round the face's *middle dot* — which a turn leaves exactly where it
is, the way the middle of a face does. Swinging them about the circle's centre
instead moved them by sixteen degrees, and two of them not at all, so the rim
turned while its inside slid in and out.

**Which of the drawing's symmetric orientations to use is decided by direction,
not colour.** There are 48 ways to lay the drawing onto the cube, and half are
*mirror images* — a mirror passes every structural test there is: twelve to a
circle, two circles to a place, three places to a quarter turn. Only the
direction of travel tells them apart, and choosing by colour resemblance, which
cannot see a mirror, drew four of the six circles turning backwards relative to
the cube beside them. Six of the 48 get all six faces right; `places.SCREEN`
works out which way each layer looks to turn on the page, and that now chooses
before colour does.

The face rosette cannot be saved the same way. Every one of the 48 pairings has
the rim and the inner eight turning the same way on exactly three faces of six,
because the drawing's nine face-places are not a faithful 3×3 — six sit about
32 pixels from the middle dot and two sit about 59, where a real face would be
four and four at a ratio of 1.41. The layer structure is exact; the face
rosette is an approximation, and this is where it shows.

**A prime move is one quarter back, and a double move turns twice.** The model only knows
clockwise, so `U'` is three applications of `U` and the state is right either
way. The picture is not: animated as three quarters it spins most of the way
round and drags the dots three-quarters of their circle with it. The trace
keeps the three steps and sweeps the picture through one quarter of minus
ninety degrees.

`X2` is the other half of that. Animated as a single 180-degree slide the dots
cut straight across, and half way through — where the cube is at a real quarter
turn — they were up to 67 degrees from the places the cube actually occupied.
It is two quarters, so it is animated as two.

**A shape's name is what moves, and here that is the sticker.** Naming the
cube's polygons after facelets looked right and was the worst bug in the
example: a facelet does not move, so at the end of a turn the shape had to come
all the way home, and every move rotated ninety degrees out and ninety degrees
back. Named after the sticker, a finished turn lands exactly where the next
scene starts — to the pixel, which `geometry.py` asserts — and nothing travels
twice.

Corner for corner, too. Ninety degrees lands a square's four corners back on
four corners, but not each on the one it started from: a sticker on the turning
face has come round by one. The reconciler tweens a polygon corner by corner,
so handing it the destination facelet's list in the destination facelet's own
order asked it to spin every square in place at the end of each turn. The shift
(`geo.SHIFT`) is carried with the sticker instead, and the squares sit still.

**Flat colour hides the turn.** Three faces of one colour are one colour, so
nothing says which way a sticker points and a rotating layer reads as diamonds
changing shape rather than a solid object turning. Each sticker is shaded by
its own normal instead, and because the normal turns with the layer the shade
slides through the turn rather than snapping at the end of it.

**Going round beats going across, and the ring must be told which way.** Dots
move in polar coordinates about the turning circle's centre. The twelve on the
circle keep their radius, so they run along it, and they are all sent the same
way: left to take the nearer way each, nine go one way and three go the other,
because the places are not evenly spaced and for those three a three-place step
spans more than half the circle. Twelve dots rotating and three coming to meet
them reads as a swap, which is the one thing the drawing exists not to do. The
nine the turn also carries are not on the circle and take the short way.

All twenty-one of them come forward together and the other thirty-three step
back, because the circle's twelve and the nine on its face are one move and the
drawing should say so.

## The shape of it

| file | what it knows |
| --- | --- |
| `cube.py` | the cube as a permutation of 54 stickers. No drawing at all. |
| `geometry.py` | where a sticker is in space, and what a turn does to it. |
| `graph.py` | the traced drawing: places, lines, circles, colours. |
| `places.py` | which place holds which sticker — searched for, then checked. |
| `main.py` | the trace and the view. |

The solve is the scramble backwards, so there is no solver and it cannot be
wrong.

## One thing the video does not claim

The drawing's own colouring is not a state any cube can be in — six of its
cubies want two opposite colours at once. So the colours come from the cube,
and the scramble was picked to land as near the traced colours as a real cube
can: twenty-seven of the fifty-four places, where guessing would give nine.
