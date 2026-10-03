# ADR 0018 — A previewer that reads an index, and never writes

**Status:** Accepted — 2026-09-25

## Context

The expensive part of building an explainer is not rendering it. It is saying
what is wrong with it.

Over one film the notes that cost the most were *"the animations and
description here make me feel so unnatural"*, *"fix weird display here"*, *"shit
happened here. not synced at all"* — each entirely fair, each pointing at a
frame, and each needing two or three exchanges before what it referred to was
established. One of them turned out to be three separate impossibilities in the
same eight seconds. Another turned out to be a recording that said its line
twice, which no amount of looking at the picture could have revealed.

The author is looking at pixels. The film is made of named shapes, Trace
Events and State. Nothing connects the two, so every observation has to be
translated into words and translated back, and both translations can be wrong.

There was a `codimate-previewer` crate once; it went with the rest of the Rust
authoring path when Python became the Authoring Surface (`e9b140f`). Today the
only output is an mp4, and inspecting one means driving `ffmpeg` by hand.

`CONTEXT.md` also rules out a GUI editor: *"preview window is viewer only"*.
That line is worth keeping. The single Python file is the source of truth, and a
window that could edit it would be a second front door into the same work —
exactly what the Authoring Surface glossary says there must not be.

## Decision

**Rendering writes an index beside the video, and the Previewer reads it.**

```
results/archimedes-km.mp4
results/archimedes-km.index.json
```

The index is one entry per Trace Event: when it starts, how long it lasts, the
event's name, the State behind it, and every shape's name, box and layer.

The Previewer plays the video, maps a time to an event, and **hit-tests boxes,
topmost layer first**. Click a pixel, learn the name of what is under it, the
event that drew it and the State that produced it. Attach a note, and what
comes out is text:

```
0:55  body_word
  event    more.3
  state    material=ice  wall=85.0  bottom=385.2  said=4
> this label is unreadable
```

**The Previewer never writes.** It produces words about the film; changing the
film stays an edit to the one Python file, made by the author or by an agent
acting on the note.

### Why an index and not the live Explanation

A window driven from the Python process that built the Explanation would need
nothing exported. It would also only exist while that process runs, and a note
would not outlive it. The index is plain data: it can be read a week later, it
can be checked into a bug report, and **an agent can answer "what is at 0:55"
with no window at all** — which makes the window a convenience rather than a
dependency.

### Why hit-testing boxes rather than an identity buffer

Rasterising a second image whose pixels carry shape identities would be exact,
including for overlapping curves. It would also be a new Engine output, a
second render pass, and a per-frame cost paid by everyone. Boxes are already in
the index, and topmost-first resolves the ordinary case. Where two boxes
genuinely overlap the answer may be the wrong one of the two — which is a
worse answer than an identity buffer gives, and a much cheaper one.

## Consequences

- **Every render grows a second file.** It is skippable, and it is the price of
  a note that still means something tomorrow.
- **The index can be large.** `spacetime` has 1,226 shapes and some 1,000
  events. Shapes that never move need not be repeated per event, but the format
  has to think about it rather than assume it away.
  Past 50 MB `render` no longer writes it on its own (`index="auto"`, 2026-10-03,
  after a dense film made 470 MB beside a 1.5 MB video): it says so, and the
  previewer given the script builds the index in memory instead.
- **The index is a published format**, so it has to be versioned like one.
- **The window is optional.** Everything it does, an agent can do by reading
  the file — which is the property that makes this worth building before any
  window exists.

## Alternatives rejected

**A window that can edit.** Drag a label, retime a beat, write back to the
Python. Far more direct, and it is the GUI editor `CONTEXT.md` rules out; it
needs a round-trip from pixels to source that nothing in the design supports,
and it competes with the file as the source of truth.

**Live, in the Python process.** Nothing to export and no format to version.
Rejected because the note cannot outlive the process, and because an agent then
needs a running Python to answer a question about a finished film.

**Re-rendering on scrub.** The most faithful — no video to keep in step with
the index. Rejected because it makes the window a second driver of the Engine.

**An identity buffer.** Exact, and a per-frame cost for everyone to make
clicking exact.

## Amended: a script, not only a video

Two of the rejections above were reversed once the window existed, because
needing a rendered video first turned out to be the larger cost: a film had to
be rendered in full before any of it could be looked at, and rendered again to
look at an edit.

So the Previewer also takes a **script**. It runs it up to `render` and keeps
the Explanation in memory, draws each moment with `frame_at` as it is scrubbed
to, and runs the script again when it is saved. Measured on a 929-scene,
162-second film: 0.8s to build, about 60ms a frame once the scene payloads are
built once rather than per frame.

What the rejections protected still holds. The note is plain text and outlives
the process; the index is the same data either way, so an agent can still ask
about a finished film with no Python running; and the window is still not a
second driver in any sense that matters — it calls the one `frame_at`, never
the Engine directly, and never writes. Given an mp4 it works as first
described, which is also the only way to hear the film.
