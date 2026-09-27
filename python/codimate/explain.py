"""Putting it together: motion rules, timing, and the render call."""

from __future__ import annotations

from .layout import height, width
from .scene import Scene, _key
from typing import Callable

from .trace import Event, Frame, Trace


View = Callable[[Frame], Scene]

PATHS = ("straight", "linear", "fall", "lift_carry_drop")

# Set by the Previewer while it runs a script: `render` hands the Explanation
# here and stops the script, instead of drawing a video nobody asked for yet.
_capture: "Callable[[Explanation], None] | None" = None


class _Captured(BaseException):
    """Stops a script at its render call. BaseException, so a script's own
    `except Exception` cannot swallow it."""


class Rule:
    """How things matching ``pattern`` travel. First matching rule wins.

        cm.Rule("*", position="lift_carry_drop", clearance=90)

    ``pattern`` matches a shape's full name with ``*`` and ``?``. A shape
    inside a group is named ``group/child``, so ``"3/*"`` targets one group and
    ``"*"`` targets everything.

    Paths:

    * ``straight`` — a straight line, easing in and out. The default, and what
      you want when each event is a distinct step.
    * ``linear`` — a straight line at constant speed. Use it when a thing is
      mid-journey at every event, like something turning: easing would make it
      accelerate and stop inside each segment.
    * ``fall`` — a parabola: sideways at a constant rate, downwards
      accelerating. What a dropped thing does, and what each hop of a falling
      ball needs, since an eased path would settle gently instead of arriving.
    * ``lift_carry_drop`` — arcs up and over, then falls. Takes ``clearance``.
    """

    def __init__(self, pattern, position: str = "straight", **options: float) -> None:
        if position not in PATHS:
            raise ValueError(
                f"unknown path {position!r} — use one of {', '.join(PATHS)}")
        self.pattern = _key(pattern) if isinstance(pattern, tuple) else str(pattern)
        self.position = position
        self.options = {k: float(v) for k, v in options.items()}

    def _payload(self):
        return (self.pattern, self.position, self.options)

    def __repr__(self):
        return f"Rule({self.pattern!r}, position={self.position!r}, **{self.options})"


class Timing:
    """How long each event lasts, in seconds."""

    def __init__(
        self,
        *,
        default: float = 0.6,
        events: "dict[str, float] | None" = None,
        opening: float = 0.8,
        final_hold: float = 1.2,
    ) -> None:
        self.default = default
        self.events = events or {}
        self.opening = opening
        self.final_hold = final_hold

    def for_event(self, event: Event) -> float:
        """How long ``event`` lasts: its own entry, or ``default``.

        A name with no entry takes ``default`` silently — which is what a
        default is for, but it means a misspelled event name costs you the
        default duration rather than an error.
        """
        return self.events.get(event.name, self.default)


def ease(t: float) -> float:
    """The easing curve the Engine applies between two moments.

        cm.ease(0.5)  ->  0.5

    This calls into the Engine, so it is the same curve your animation is
    actually using — not a copy of it.
    """
    from . import _codimate

    return _codimate.ease(float(t))


def _quoted(name: str, room: int = 32) -> str:
    return f'"{name}"' if len(name) <= room else f'"{name[:room - 1]}…"'


def _plain(value, depth: int = 0):
    """`value` as something JSON can hold, without pretending to be complete.

    An Item keeps its id, because that is its identity. Anything this does not
    recognise becomes its `repr` rather than raising — an index that refuses to
    be written for one odd object in the State is no use to anybody.
    """
    from .trace import Item

    if depth > 6:
        return repr(value)
    if isinstance(value, Item):
        return {"value": _plain(value.value, depth + 1), "id": value.id}
    if isinstance(value, dict):
        return {str(k): _plain(v, depth + 1) for k, v in value.items()}
    if isinstance(value, (list, tuple, set)):
        return [_plain(v, depth + 1) for v in value]
    if value is None or isinstance(value, (str, int, float, bool)):
        return value
    if hasattr(value, "__dict__"):
        return {k: _plain(v, depth + 1) for k, v in vars(value).items()}
    return repr(value)


class Explanation:
    """A trace, a view and a timing, ready to render.

    Built by :func:`explain` rather than directly. Holds one Scene per Trace
    Event and the gap between each pair; :meth:`render` hands all of it to the
    Engine once, and everything per-frame happens in there.
    """
    def __init__(
        self,
        *,
        trace: Trace,
        view: View,
        motion: "list[Rule] | None" = None,
        timing: "Timing | None" = None,
    ) -> None:
        self.timing = timing or Timing()
        self.motion = motion or []
        self.trace = trace

        # The view runs once per event, not per frame — this is the whole
        # reason Python is fast enough to be the authoring language.
        self.scenes = [view(Frame(state=trace.initial, event=None))]
        for event in trace.events:
            self.scenes.append(view(Frame(state=event.state, event=event)))

        self.durations = [self.timing.for_event(e) for e in trace.events]

        # A held opening and a held ending, expressed as segments that go
        # nowhere. No special case in the Engine.
        self.scenes.insert(0, self.scenes[0])
        self.durations.insert(0, self.timing.opening)
        self.scenes.append(self.scenes[-1])
        self.durations.append(self.timing.final_hold)

    @property
    def duration(self) -> float:
        return sum(self.durations)

    def _payloads(self) -> dict:
        """The scenes as the Engine takes them, built once.

        Building them is most of what a single frame costs — 100ms of 157 on
        a 929-scene film — so a Previewer drawing frame after frame, or a
        sheet of twelve, pays it once rather than every time.
        """
        if getattr(self, "_built", None) is None:
            self._built = {
                "scenes": [s._payload() for s in self.scenes],
                "cameras": [s._camera() for s in self.scenes],
                "rules": [r._payload() for r in self.motion],
            }
        return self._built

    def render(self, output: str, *, fps: float = 30, scale: float = 1.0,
               index: bool = True) -> str:
        """Draw every frame and write the video.

        Coordinates always mean what `cm.canvas()` says — `scale` only changes
        how many pixels each one becomes, so nothing in your view has to move:

            .render("out.mp4", fps=60, scale=1.5)   # 1080p60 from the default

        Frames are rasterized at the larger size rather than upscaled
        afterwards, so 1080p is genuinely drawn at 1080p.

        The folder is created if it does not exist, so `render("results/x.mp4")`
        works on a fresh clone.
        """
        from pathlib import Path

        if _capture is not None:
            _capture(self)
            raise _Captured

        from . import _codimate  # imported here so the pure Python is testable

        sounds = self.sounds()      # a missing clip fails now, not after the picture
        _find_encoder()
        Path(output).expanduser().resolve().parent.mkdir(parents=True, exist_ok=True)

        _codimate.render(
            **self._payloads(),
            durations=self.durations,
            output=output,
            width=width(),
            height=height(),
            fps=float(fps),
            scale=float(scale),
        )

        if sounds:
            self._lay_sound(output, sounds)
        if index:
            self.write_index(output)
        report = self.report()
        if report:
            import sys
            print(report, file=sys.stderr)
        return output

    def sounds(self) -> "list[tuple[float, str]]":
        """Every clip, as `(start, path)`: `emit("said", sound="said.wav")`
        starts `said.wav` as the beat for "said" begins.

        The start is read off :meth:`timeline`, so a sound cannot disagree
        with the picture: both are timed by the same events. Sound is never
        part of a Scene (ADR 0007) — it rides on the event, beside it.
        """
        from pathlib import Path

        beats = self.timeline()
        found = []
        for k, event in enumerate(self.trace.events):
            clip = event.data.get("sound")
            if clip is None:
                continue
            clip = Path(clip).expanduser().resolve()
            if not clip.exists():
                raise FileNotFoundError(f"no sound {clip} (emitted by {event.name!r})")
            # Beat 0 is the opening hold; event k is beat k + 1.
            found.append((round(beats[k + 1][0], 3), str(clip)))
        return found

    def chapters(self) -> "list[tuple[float, str]]":
        """Where each part of the film starts, as `(start, name)`:
        `emit("water", chapter="Water")` starts one as that beat begins.

        A film is hundreds of events — a highlighted word is one — so the
        Previewer cannot show a picture of each. A chapter is the author saying
        where a part begins, which nothing downstream could work out.
        """
        beats = self.timeline()
        return [(round(beats[k + 1][0], 3), str(event.data["chapter"]))
                for k, event in enumerate(self.trace.events)
                if event.data.get("chapter")]

    def mix_sound(self, output: str) -> "str | None":
        """Every clip laid at its start and summed, as one audio file —
        what `render` puts under the picture, without the picture. `None` if
        nothing makes a sound."""
        import subprocess

        sounds = self.sounds()
        if not sounds:
            return None
        inputs, graph = _mixing(sounds, first=0)
        subprocess.run([_ffmpeg(), "-y", "-v", "error", *inputs,
                        "-filter_complex", graph, "-map", "[a]", output],
                       check=True)
        return output

    def _lay_sound(self, output: str, sounds) -> None:
        """Mux the clips under a rendered video. The picture is copied, not
        re-encoded; ffmpeg cannot write where it reads, hence the temporary."""
        import subprocess
        from pathlib import Path

        video = Path(output)
        out = video.with_name(f".{video.stem}.sound.tmp{video.suffix}")
        inputs, graph = _mixing(sounds, first=1)
        subprocess.run(
            [_ffmpeg(), "-y", "-v", "error", "-i", str(video), *inputs,
             "-filter_complex", graph, "-map", "0:v", "-map", "[a]",
             # No -shortest: the picture runs past the last clip by design,
             # into the final hold, and truncating would cut the ending.
             "-c:v", "copy", "-c:a", "aac", "-b:a", "160k", str(out)],
            check=True)
        out.replace(video)

    def covered(self) -> "list[tuple[float, str, str]]":
        """Every label something was drawn on: `(seconds, above, below)`.

        Checked once per Trace Event, on the picture the event settles into
        (ADR 0017). A collision that lasts is reported where it starts, not
        at every beat it survives.
        """
        from .scene import covered

        found, before = [], set()
        for i, (at, secs, _) in enumerate(self.timeline()):
            now = set(covered(self._arrives(i)))
            found += [(round(at + secs, 2), a, b) for a, b in now - before]
            before = now
        return sorted(found)

    def report(self) -> str:
        """:meth:`covered`, worded — what `render` prints when it is not empty."""
        found = self.covered()
        if not found:
            return ""
        many = len(found) != 1
        lines = [f"{len(found)} label{'s are' if many else ' is'} covered:"]
        for at, above, below in found:
            stamp = f"{int(at // 60)}:{at % 60:05.2f}"
            lines.append(f"  {stamp}  {_quoted(above)} is drawn over {_quoted(below)}")
        return "\n".join(lines) + "\n\nrendered anyway."

    def _arrives(self, beat: int):
        """The Scene beat `beat` travels to — what its event produced."""
        return self.scenes[min(beat + 1, len(self.scenes) - 1)]

    def write_index(self, output: str) -> str:
        """Write :meth:`index` beside `output`, as `<name>.index.json`.

        The file wraps the beats with the canvas size — without it a click on
        the video cannot be turned back into the coordinates the boxes are in
        — and a version, because other tools read this and it will change.
        """
        import json
        from pathlib import Path

        video = Path(output)
        beside = video.with_name(f"{video.stem}.index.json")
        beside.parent.mkdir(parents=True, exist_ok=True)
        written = {"version": 1, "canvas": [width(), height()],
                   "beats": self.index()}
        beside.write_text(json.dumps(written, ensure_ascii=False, indent=2) + "\n")
        return str(beside)

    def sheet(self, times, output: str = "sheet.png", *,
              columns: int = 3, scale: float = 0.5) -> str:
        """Several moments at once, tiled into one picture.

            cm.explain(...).sheet([8, 22, 54, 96], "look.png")

        `frame_at` answers "what does 0:54 look like"; this answers "does the
        whole thing hang together", which is the question actually being asked
        when somebody renders a film and scrubs through it. The tiling is done
        by the ffmpeg that rendering already needs.

        `scale` shrinks the sheet, and does the shrinking in ffmpeg rather
        than in the Engine, which will not rasterize below 1:1 — so a whole
        film fits on a screen instead of arriving four thousand pixels wide.
        """
        import subprocess
        import tempfile
        from pathlib import Path

        ffmpeg = _ffmpeg()

        times = list(times)
        rows = -(-len(times) // columns)
        with tempfile.TemporaryDirectory() as folder:
            for i, at in enumerate(times):
                self.frame_at(at, f"{folder}/f{i:03d}.png")
            Path(output).expanduser().resolve().parent.mkdir(
                parents=True, exist_ok=True)
            subprocess.run(
                [ffmpeg, "-y", "-v", "error", "-i", f"{folder}/f%03d.png",
                 "-vf", f"scale=iw*{scale}:ih*{scale},tile={columns}x{rows}",
                 output],
                check=True)
        return output

    def frame_at(self, seconds: float, output: str = "frame.png",
                 scale: float = 1.0) -> str:
        """Save a single moment as a PNG, without rendering the video.

            cm.explain(...).frame_at(12.5, "check.png")

        The same scenes, timing and arithmetic as :meth:`render`, resolved at
        one instant. Checking a frame by rendering the whole video and seeking
        into it costs a minute to look at one second.

        ``scale`` matches ``render``'s, so the debug frame is rasterized the
        way the video is — worth passing when you are checking text, which is
        the thing that has historically differed between the two.
        """
        from pathlib import Path

        from . import _codimate

        Path(output).expanduser().resolve().parent.mkdir(parents=True, exist_ok=True)
        # Built once and kept: building the film is most of a frame's cost —
        # seconds, on a thousand scenes of a thousand shapes — and drawing one
        # moment of it is milliseconds.
        if getattr(self, "_frames", None) is None:
            self._frames = _codimate.Frames(
                **self._payloads(), durations=self.durations,
                width=width(), height=height())
        self._frames.png(float(seconds), output, float(scale))
        return output

    def index(self) -> "list[dict]":
        """Every beat, as data: when, what happened, and what was on screen.

            for beat in cm.explain(...).index():
                print(beat["at"], beat["event"], len(beat["shapes"]))

        One entry per beat, in step with :meth:`timeline`, carrying the State
        behind it — so a question like "what is at 0:55, and what put it
        there?" has an answer without rendering a frame and squinting at it.

        **Shapes are a difference, not a list.** Each beat carries only what
        changed: `shapes["set"]` is the ones that arrived or moved, and
        `shapes["gone"]` the names that left. Apply them in order from the
        first beat to know what is on screen at any of them. A whole list per
        beat is the obvious format and it is 94% repetition — in a hundred
        second film that is seven megabytes of the same rectangle.

        `covered` lists the `[above, below]` pairs where something is drawn
        on a label at that beat (ADR 0017), and `chapter` is there only on a
        beat that starts one (:meth:`chapters`).

        A shape's box is `None` only for a formula on a machine without
        Typst, which is what measures one (ADR 0005).

        `render` writes this beside the video by default (ADR 0018).
        """
        from .scene import box, covered

        states = ([self.trace.initial]
                  + [e.state for e in self.trace.events]
                  + [self.trace.events[-1].state if self.trace.events
                     else self.trace.initial])
        out, before = [], {}
        for i, (at, secs, name) in enumerate(self.timeline()):
            # Beat `i` travels from scenes[i] to scenes[i + 1]; what it
            # arrives at is what the event named here produced.
            scene = self._arrives(i)
            now = {
                shape.item: {"name": shape.item, "kind": shape.kind,
                             "layer": shape.layer, "box": box(shape)}
                for shape in scene._shapes.values()
            }
            entry = {
                "at": at,
                "secs": round(secs, 3),
                "event": name,
                "state": _plain(states[min(i, len(states) - 1)]),
                "shapes": {
                    "set": [s for n, s in now.items() if before.get(n) != s],
                    "gone": [n for n in before if n not in now],
                },
                "covered": [list(pair) for pair in covered(scene)],
            }
            # Beat 0 is the opening hold; beat i + 1 is event i.
            events = self.trace.events
            event = events[i - 1] if 0 < i <= len(events) else None
            if event is not None and event.data.get("chapter"):
                entry["chapter"] = str(event.data["chapter"])
            out.append(entry)
            before = now
        return out

    def timeline(self) -> "list[tuple[float, float, str]]":
        """Every beat as ``(start, duration, event name)``, in seconds.

            for start, length, name in cm.explain(...).timeline():
                print(f"{start:6.2f}  {length:4.2f}  {name}")

        What is on screen at 0:42, and how long each beat actually lasts —
        the two questions you have when a video feels wrong. Pair it with
        :meth:`frame_at` to look at the moment you find.
        """
        # Read from `durations`, which already carries the held opening and
        # ending, rather than recomputing them — a second copy of that
        # arithmetic is how a timeline starts disagreeing with the video.
        names = ["(opening)"] + [e.name for e in self.trace.events] + ["(final hold)"]
        out, at = [], 0.0
        for name, length in zip(names, self.durations):
            out.append((round(at, 3), length, name))
            at += length
        return out


def _mixing(sounds, first: int):
    """ffmpeg inputs and a filter graph that lays each clip at its start and
    sums them. `first` is the input number of the first clip. `normalize=0`
    keeps every clip at its recorded level — amix otherwise divides by the
    number of inputs, and the voice fades as the film gets longer."""
    inputs = [arg for _, clip in sounds for arg in ("-i", clip)]
    ids = range(first, first + len(sounds))
    delays = "".join(f"[{i}:a]adelay={round(start * 1000)}:all=1[d{i}];"
                     for i, (start, _) in zip(ids, sounds))
    summed = "".join(f"[d{i}]" for i in ids)
    return inputs, f"{delays}{summed}amix=inputs={len(sounds)}:normalize=0[a]"


def _ffmpeg() -> str:
    """The ffmpeg the render uses, for the jobs done around it."""
    import os
    import shutil

    _find_encoder()
    found = os.environ.get("CODIMATE_FFMPEG") or shutil.which("ffmpeg")
    if not found:
        raise RuntimeError("this needs ffmpeg — install it, or set CODIMATE_FFMPEG")
    return found


def _find_encoder() -> None:
    """Make sure the engine can find an ffmpeg to run.

    Rendering needs ffmpeg, which pip cannot install. Rather than let
    `pip install codimate` succeed and then fail on someone's first render —
    the worst moment to learn about a prerequisite — fall back to the static
    build that ships with `imageio-ffmpeg`.

    A system ffmpeg is preferred: it is usually newer, hardware-accelerated,
    and the one the author already expects to be used. The bundled copy is a
    safety net, not the default.

    Silent when nothing is found; the engine raises its own error naming both
    the install and the override.
    """
    import os
    import shutil

    if os.environ.get("CODIMATE_FFMPEG") or shutil.which("ffmpeg"):
        return

    try:
        import imageio_ffmpeg
    except ImportError:
        return

    try:
        os.environ["CODIMATE_FFMPEG"] = imageio_ffmpeg.get_ffmpeg_exe()
    except Exception:  # noqa: BLE001 — a broken fallback must not mask the real error
        pass


def explain(
    *,
    trace: Trace,
    view: View,
    motion: "list[Rule] | None" = None,
    timing: "Timing | None" = None,
) -> Explanation:
    """Gather an algorithm, a view and a timing into something renderable.

    ``trace`` is what a ``@cm.trace()``-marked function returns: the moments
    your algorithm passed through. ``view`` is called once per moment and
    returns the picture of it. ``motion`` and ``timing`` are optional —
    without them every shape travels in a straight line and every event lasts
    the same.

        cm.explain(trace=flip(tally), view=view).render("results/coins.mp4")

    Nothing is computed here; the work happens in :meth:`Explanation.render`.
    """
    return Explanation(trace=trace, view=view, motion=motion, timing=timing)
