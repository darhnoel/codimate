# ADR 0020 — A compact index, read lazily

**Status:** Accepted — 2026-10-04

## Context

The index ([ADR 0018](0018-a-previewer-that-reads-an-index.md)) is what lets the
Previewer say what is at 0:55 and what put it there. It was a JSON file, and for a
film where everything moves every beat it was hundreds of megabytes: 387 MB estimated
for `examples/year`, 470 MB for `orbits`, 1.3 GB for a denser sketch, each beside a
video of a few megabytes. `render(index="auto")` answered that by not writing it past
50 MB, which turned the feature off for exactly the films that most needed it.

Two things were wrong, and they are separate.

**The format.** Decimal text repeats every field name for every shape and cannot be
compressed well: the digits of a float do not repeat. Measured on 300 beats of `year`:

| Encoding | Size |
|---|---|
| JSON, indented | 107 MB |
| JSON, no whitespace | 51.8 MB |
| JSON + gzip / + lzma | 13.6 MB / 10.0 MB |
| MessagePack, 64-bit floats | 18.8 MB |
| MessagePack + lzma | 0.89 MB |
| columns of integer differences + lzma | 0.87 MB |

The Maxwell film's index (10.5 MB as JSON) showed the same: 0.25 MB as MessagePack plus
lzma, 0.03 MB as columns of differences. BSON came out the same size as JSON (counted
at 4.58 MB against 4.66), Avro would need a dependency and has no random access, and
SQLite stored the same facts in about 4 MB.

**The way it was used.** The Previewer, given a script, built the whole index in memory
(about 4 s and 0.6 GB on top of the 1.35 GB the film itself takes on `year`) to answer
for one beat at a time. The scenes were already there. Asking one for its shapes took
about a millisecond.

## Decision

**Three forms behind one small interface** (`codimate.index.Index`: `canvas`, `beats`,
`shapes_at(beat)`), so the Previewer asks the same question of all of them.

- **`LiveIndex`**, for a film in memory. It builds only the per-beat table (when, event,
  state, covered, chapter) and reads shapes off the scene when asked. No file is read.
- **`CompactIndex`**, a file, `<video>.index.bin`, which `render` now writes.
- **`DictIndex`**, the readable JSON, kept so nothing written before breaks and for
  `index="json"`.

**The compact format** stores a box to 1/16 of a pixel as the difference from that
shape's previous box, names and kinds once per chunk, and everything as little-endian
columns under lzma: int32, and int64 for boxes, which a zoom can make enormous (the
first version used int32 and overflowed on Cavendish's zoom-out). Beats are grouped in chunks of 32, each starting with
everything on screen, so any beat decodes from one chunk and no beat depends on the
ones before it. The per-beat table is a footer, so opening the file reads that and
nothing else. The writer emits a chunk at a time, so its memory does not grow with the
film. Fixed-width columns rather than variable-length integers: the compressed size is
the same (0.31 against 0.32 MB on 128 beats of `year`) and `array.frombytes` decodes
a column without a Python loop.

`render(index=)` is `"auto"` or `True` for the compact file, `"json"` for the readable
one, `False` for neither; anything else is an error before a frame is drawn. The 50 MB
cutoff and `INDEX_LIMIT` are gone. `exp.index()` still returns the same list, and
`preview.index_of`, `at`, `on_screen` and `covered` accept a dict or an `Index`.

To read a compact file by eye: `python -m codimate.preview --json
[--beats 100:120] film.mp4` prints it as the JSON the old file held.

## Measured

| | Readable JSON | Compact |
|---|---|---|
| `maxwell`, 2,861 beats | 10.5 MB | 0.14 MB |
| `year`, 1,553 beats | about 387 MB | 5.3 MB (73x smaller) |

On `year`: written in 13 s with 127 MB of extra memory; opened in 2 ms; the shapes at a
beat decode in about 30 ms from a cold chunk and 2 ms from a cached one; the largest
difference from the scene itself is 1/32 of a pixel. In a script preview a beat takes
about 1 ms and no index is built.

## Consequences

- **The index exists for every film.** It no longer disappears when it is most useful.
- **A box is approximate.** To 1/16 of a pixel, which is far finer than hit-testing
  needs, but it is not the number the Engine drew with. `index="json"` is as exact as
  ever.
- **It is not readable by eye.** Hence `--json`. It cannot be opened in an editor or
  searched with grep.
- **A published format**, versioned in its magic number and footer. A reader refuses a
  file it does not recognise rather than guessing.
- **Writing costs seconds on a dense film** (13 s for `year`), because every shape's box
  is measured. It does not cost memory.
- **Stdlib only:** `lzma`, `array`, `struct`, `json`.

## Alternatives rejected

**SQLite.** Good at questions, about 4 MB on Maxwell and 40 times the size of this
format. The questions the Previewer asks are one lookup by beat, which chunks answer.

**MessagePack, BSON, Avro, CBOR.** Different ways to write the same records. They keep
the repeated field names and absolute numbers, which is where the size is.

**Keep JSON and gzip it.** 13.6 MB for 300 beats of `year`, and no random access.

**A GPU renderer, or a browser player, to fix this.** Unrelated: the index is data
about a film, not a way of drawing one.
