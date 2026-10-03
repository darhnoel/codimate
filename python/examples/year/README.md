# year

One year of the Sun, the Earth and the Moon — 1 January to 31 December 2025, 365
days in 23 seconds of motion — with the motion taken from the sky and only the sizes
made up.

```bash
.venv/bin/python python/examples/year/main.py
```

Small points on the orbit mark the six places where something happens: closest and
furthest from the Sun, the two equinoxes and the two solstices. The year runs until
the Earth reaches one, **stops**, and a caption says what is special there, a word
at a time at reading pace, as `archimedes` does it. Then it runs on. The pauses are
why the film is about 56 seconds and not 23.

## What is real

`sky.py` holds the astronomy and knows nothing about drawing. It is Meeus's mean
orbital elements (*Astronomical Algorithms*, ch. 25 and 47) handed to Kepler's
equation by `science.Orbit`:

- the Earth's orbit has its true eccentricity (0.0167) and closest approach in
  early January, so the Earth goes quicker then and the distance on screen runs
  from about 147 to 152 million km;
- it reaches the equinoxes and solstices when the almanac says it does (within a
  hundredth of a degree — the test checks all four);
- the Moon's orbit leans 5.1° and **does not stay put**: its perigee runs forward
  41° and its node back 19° across the year;
- the phases come from the Sun's light falling on a ball, not from a table, and
  fall within a degree or so of the almanac's new and full moons;
- the Earth's axis leans 23.4° toward one fixed direction all year. That is the
  entire cause of the seasons, and the two stubs at the poles are there so it
  can be seen.

## What is not

The Earth is 12,700 km across and its orbit 300 million km wide. At true scale it
is a fraction of a pixel and the Moon's orbit is smaller than that, so the bodies
and the Moon's distance are exaggerated, and so is the spinning: the Sun turns every
25 days and the Earth every one, which at sixteen days a second is 16 turns a second
for the Earth. They turn slowly enough to see instead (arrows ride each equator, and the
Earth's lean with its axis), and the opening captions explain the changes to size,
Moon distance, and apparent spin. The Moon's arrows are the exception: it keeps one face to the Earth, so it
turns once a month, 13.2° a day, and that is drawn at the real rate. The Moon's mean elements leave out its biggest wobbles, so
it is a degree or two off, not exact.

## Things to try

- `DAYS_PER_SECOND = 2` and `DAYS = 90` for one season, slowly.
- Move `START` and `JD_START` in `sky.py` to another year; the whole film follows.
- A film like this is 1,400 moments of 1,400 shapes. Building it takes about a minute,
  and rendering peaked at 6.4 GB of memory (the Engine's copy of every shape comes
  on top of Python's), so close other things first. `render` skips the index, which
  would be hundreds of megabytes.
