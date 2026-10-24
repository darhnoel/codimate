"""Generated from api/surface.toml by tools/generate.py. Do not edit.

The Python Skin's side of the Interface Manifest: each method sends its
arguments across the Waist and keeps what comes back."""

from __future__ import annotations

from typing import TYPE_CHECKING

from ._waist import _as_points, _as_size, _at, _bool, _canvas, _film_font, _group, _int, _items, _key, _keys, _number, _pair, _paired, _point, _points, _radius, _slot, _string, _style, _text, call

if TYPE_CHECKING:
    from .scene import Group, Handle


class _GroupSurface:
    def rect(self, key, *, h, w=None, at=None) -> "Handle":
        """A rectangle. Say where with a point, a Slot, or `cm.at(...)`.

            scene.rect("box", h=40, at=cm.at(bottom=0)).fill("blue")

        Round the corners with `.round(12)`.
        """
        return self._build("rect", {"group": self._frame(), "key": _key(key), "h": _number(h), "w": _number(w), "at": _at(at)})

    def circle(self, key, *, r, at=None) -> "Handle":
        """A filled circle. `color="none"` with `.fill(edge=...)` draws a ring."""
        return self._build("circle", {"group": self._frame(), "key": _key(key), "r": _number(r), "at": _at(at)})

    def text(self, key, content, *, size=16, font=None, at=None) -> "Handle":
        """Text, centred horizontally. You never deal with baselines.

            scene.text("title", "Bubble sort", size=48, font="fonts/Inter-Bold.ttf")

        ``font`` is a ``.ttf``/``.otf`` file or an installed font's name, for this
        text only; unset, the film's font (``cm.font``).
        """
        return self._build("text", {"group": self._frame(), "film_font": _film_font(), "key": _key(key), "content": _text(content), "size": _number(size), "font": _string(font), "at": _at(at)})

    def line(self, key, *, start, end, w=2.0) -> "Handle":
        """A line between two points. `w` is its thickness.

        A line has no centre to anchor, so it takes its two ends directly. Each
        may be a Slot — a line joins the middles of two places — or an `(x, y)`.
        """
        return self._build("line", {"group": self._frame(), "key": _key(key), "start": _point(start), "end": _point(end), "w": _number(w)})

    def arrow(self, key, *, start, end, w=4.0, head=16.0) -> "Handle":
        """An arrow, as a single filled shape.

        One shape rather than a line plus a separate head, so it carries one
        name and travels as one thing. `w` is the shaft, `head` the point.
        """
        return self._build("arrow", {"group": self._frame(), "key": _key(key), "start": _point(start), "end": _point(end), "w": _number(w), "head": _number(head)})

    def polygon(self, key, points, *, closed=True) -> "Handle":
        """A shape with corners: a triangle, a wedge, an arrow head, a wing.

            scene.polygon("tri", cm.ngon(3, r=50, at=(640, 360))).fill("green")

        `points` is a sequence of `(x, y)`. Closed and filled by default;
        `closed=False` leaves an open outline, which shows only with an edge.

        Two polygons tween only if they have the same number of corners —
        interpolating a triangle into a pentagon has no answer worth inventing,
        so the later shape stands for the whole beat instead.
        """
        return self._build("polygon", {"group": self._frame(), "key": _key(key), "points": _points(points), "closed": _bool(closed)})

    def curve(self, key, points, *, w=2.0, closed=False) -> "Handle":
        """A smooth line through every one of `points`.

            scene.curve("plot", samples, w=3).fill("cyan")

        The points are on the curve, not control points: you hand it samples —
        a function you plotted, a path something travelled — and it draws a
        smooth line through them. `w` is the stroke width.

        Open by default, and an open curve is *drawn* rather than filled, the
        way a line is. `closed=True` joins the ends and makes it a fillable
        shape, like a polygon with rounded-off corners.

        Two curves tween only if they have the same number of points, the same
        rule polygons follow; otherwise the later one stands for the beat.
        """
        return self._build("curve", {"group": self._frame(), "key": _key(key), "points": _points(points), "w": _number(w), "closed": _bool(closed)})

    def arc(self, key, *, r, sweep, at=None) -> "Handle":
        """A slice of a circle — an angle mark, a pie, a dial, an orbit.

            scene.arc("angle", r=90, sweep=(0, 50)).fill("none", edge="cyan",
                                                          edge_w=3)
            scene.arc("slice", r=120, sweep=(0, 120)).fill("orange").round(1)

        `sweep` is `(start, end)` in degrees, clockwise from twelve o'clock —
        the same zero `cm.ngon` uses. `r` is the radius, or `(rx, ry)` for an
        ellipse.

        Open by default, so it draws as a curved line. `.round(1)` closes it
        back to the centre and makes a pie slice you can fill.

        Two arcs always tween, however far apart their angles are, so an angle
        mark grows and a pie fills smoothly. That is what this cannot be done
        with `curve`: a curve through points on a circle changes its point
        count when the sweep changes, and then it snaps instead of sweeping.
        """
        return self._build("arc", {"group": self._frame(), "key": _key(key), "r": _radius(r), "sweep": _pair(sweep), "at": _at(at)})

    def group(self, key, slot=None, *, at=None, anchor=None, w=None) -> "Group":
        """A place to draw a thing made of several shapes.

            bar = scene.group(item.id, slot)
            bar.rect("bar", h=item.value * 70, at=cm.at(bottom=0))
            bar.text("label", item.value, at=cm.at(top=20))

        Give it a Slot and it sits where the Slot says, or place it with ``at``
        like any shape. Inside it, ``0`` is that point, and everything drawn on
        it moves as one.
        """
        return self._nest(call("group", {"group": self._frame(), "key": _key(key), "slot": _slot(slot), "at": _at(at), "anchor": _string(anchor), "w": _number(w)}))

    def formula(self, key, latex, *, size=16.0, at=None) -> "Handle":
        r"""Real mathematics, written as LaTeX.

            scene.formula("eq", r"\frac{QK^{T}}{\sqrt{d_k}}", size=34)

        Use a raw string, or every backslash needs doubling. `size` means what
        it means for :meth:`text`. Draw it on with `.write(reveal=, pen=)`.

        The result is glyph outlines, not a font — so it moves, fades and
        recolours like any other shape, and is typeset once when the video is
        built. Needs the `typst` binary on PATH. Rendering uses the bundled
        `ffmpeg`.
        """
        return self._build("formula", {"group": self._frame(), "key": _key(key), "latex": _string(latex), "size": _number(size), "at": _at(at)})

    def image(self, key, file, *, size=None, at=None) -> "Handle":
        """A picture — a photo, a screenshot, a figure — drawn into the frame.

            scene.image("paper", "figures/attention.png", size=(520, 300))
            scene.image("shot", "screen.png", size=400).on(opacity=0.4)

        PNG and JPEG, read by content rather than by extension. `size` is a box
        it fits inside, one number or `(w, h)`, aspect always kept; leave it out
        and the picture is drawn at its own pixel size.

        It moves, scales, turns and fades like anything else, and `focus()`
        frames it. For a logo or a diagram prefer `svg`, which arrives as
        geometry you can recolour and draw on — an image is pixels, so it can
        only be placed.
        """
        return self._build("image", {"group": self._frame(), "key": _key(key), "file": _text(file), "size": _radius(size), "at": _at(at)})

    def svg(self, key, file, *, size=120.0, at=None) -> "Handle":
        """Vector art from a file, drawn as real geometry.

            scene.svg("logo", "brand.svg", size=90, at=cm.at(x=1180, top=24))
            scene.svg("chart", "flow.svg", size=(900, 420))

        Because it arrives as geometry rather than pixels it behaves like
        anything else you draw: it tweens, `.turn()` and `.grow()` transform
        it, `focus()` frames it, and `.write(pen=2)` draws it on stroke by
        stroke.

        `size` is a box it fits inside — one number for a square, or `(w, h)`.
        The aspect ratio is always kept, so a wide diagram and a tall one both
        land inside the box you named.

        The artwork keeps its own colours. `.fill(colour)` overrides all of
        them at once, which flattens it to a silhouette on purpose.

        Labels come across as real text, shaped by the same code that draws
        every `scene.text`. Layout that cannot be read is refused rather than
        guessed at — a `tspan`, a `textPath`, or a turned label, since the
        renderer cannot turn glyphs.
        """
        return self._build("svg", {"group": self._frame(), "key": _key(key), "file": _text(file), "size": _radius(size), "at": _at(at)})

    def marker(self, key, *, w, h, at=None) -> "Handle":
        """A highlighter stroke over any box on screen.

            scene.marker("note", w=260, h=70, at=(640, 300)).turn(-8)
            scene.marker("note", w=260, h=70, at=(640, 300)).write(reveal=0.5)

        It is not tied to text. `.write(reveal=0..1)` draws it on from its left
        end, and `.turn(degrees)` sets its angle and so the way it draws. Colour
        with `.fill(name)`: yellow, orange, pink, green, blue or purple, or any
        `#rrggbb`. It is slightly translucent and sits under layer 10, where
        the words go. Drawn by hand: straight edges, a slight tilt and
        chisel-cut ends, the same every time for the same name.
        """
        return self._keep_marked(call("marker", {"group": self._frame(), "key": _key(key), "w": _number(w), "h": _number(h), "at": _at(at)}))

    def highlight(self, key, content, *, size=16.0, font=None, at=None) -> "tuple[Handle, Handle]":
        """Words under a highlighter stroke, the stroke sized to them.

            band, words = scene.highlight("pivot", "Pivot", size=30, at=(640, 360))
            band.write(reveal=0.0)       # rubbed out; the words turn white
            words.fill("#171717")        # on a white page, keep them dark

        A marker over the words' measured box, padded by a quarter of `size`
        on every side, with the words drawn as their own text shape on top.
        Marked, the words are near-black; `band.write(reveal=...)` blends them
        back to their own colour as the stroke rubs out, in step with it. Their
        own colour is white unless `words.fill(...)` says otherwise. `font` is
        as for `text`, and the stroke is sized to the words in it. Give each
        word its own name and keep its words fixed under that name: the stroke
        can then move and draw on or off between words without the letters
        overflowing it.
        """
        return self._keep_marked(call("highlight", {"group": self._frame(), "film_font": _film_font(), "key": _key(key), "content": _text(content), "size": _number(size), "font": _string(font), "at": _at(at)}))


class _HandleSurface:
    def fill(self, color="white", *, edge=None, edge_w=None) -> "Handle":
        """Colour it. `edge` outlines it; `color="none"` leaves it unfilled.

        Saying nothing about the outline leaves the outline alone, so
        recolouring a shape does not silently erase the edge it was given —
        by an earlier `fill`, or by `curve`, which carries its stroke width
        there.
        """
        self._change("fill", {"shape": self._shape(), "color": _string(color), "edge": _string(edge), "edge_w": _number(edge_w)})
        return self

    def turn(self, degrees, pivot="center") -> "Handle":
        """Rotate it. `pivot` is center, top, bottom, left or right.

        Text moves but does not turn — rotating glyphs is renderer work that
        has not been done.
        """
        self._change("turn", {"shape": self._shape(), "degrees": _number(degrees), "pivot": _string(pivot)})
        return self

    def grow(self, scale) -> "Handle":
        """Scale it: a number for both axes, or `(sx, sy)` to stretch."""
        self._change("grow", {"shape": self._shape(), "scale": _radius(scale)})
        return self

    def on(self, layer=None, opacity=None) -> "Handle":
        """Which layer it draws on, and how solid it is."""
        self._change("on", {"shape": self._shape(), "layer": _number(layer), "opacity": _number(opacity)})
        return self

    def round(self, radius) -> "Handle":
        """Round a rectangle's corners, clamped to half its short side."""
        self._change("round", {"shape": self._shape(), "radius": _number(radius)})
        return self

    def write(self, reveal=None, pen=0.0) -> "Handle":
        """How much of a formula or an imported SVG shows, and whether a pen
        draws it on. On a marker, how far its stroke has been drawn on.

        The pen rides in a different field for the two kinds — `w` for a
        formula, `size` for an SVG, whose `w` is already the box it fits
        inside. The payload reuses fields per kind by design (ADR 0008); this
        is the one place that reuse is visible from Python.
        """
        self._change("write", {"shape": self._shape(), "reveal": _number(reveal), "pen": _number(pen)})
        return self


class _SceneSurface:
    def focus(self, *names, pad=40.0, least=240.0) -> "None":
        """Look at these things. The camera works out where to stand.

            scene.focus("cat")                    # frame this shape
            scene.focus(("cell", 2, 1), pad=60)   # any name, with room around it
            scene.focus("The", "cat", "sat")      # several — all of them in shot
            scene.focus()                         # the whole canvas again

        You never write a camera coordinate. The Engine knows where everything
        is and how big it is, so it derives the framing — and because each
        moment is framed on its own, a camera that moves between two moments is
        just two sets of coordinates that differ, which the usual tween
        animates. Camera movement needs nothing new.

        Naming rather than positioning also means the shot survives a layout
        change: move the shape and the camera follows, because the name is the
        part that is stable.

        ``least`` is the smallest thing the camera will fill the frame with, so
        focusing on a full stop does not magnify it into abstraction.

        Anything on an :meth:`overlay` stays where it is put.
        """
        self._aim(call("focus", {"names": _keys(names), "pad": _number(pad), "least": _number(least)}))

    def overlay(self) -> "Group":
        """A place for things the camera does not move.

            hud = scene.overlay()
            hud.text("title", "Scaled dot-product attention", x=640, y=52)

        The diagram pans and zooms underneath; whatever is drawn here stays
        put. Titles and narration belong here — the moment the picture zooms, a
        caption that zooms with it is wrong, and usually off the edge of the
        frame entirely.

        Content is in the world unless it says otherwise, because in an
        explanation most of the frame is diagram and only a little is
        narration.
        """
        return self._open_overlay(call("overlay", {"group": self._frame()}))


class _ExplanationSurface:
    def timeline(self):
        """Every beat as ``(start, duration, event name)``, in seconds.

            for start, length, name in cm.explain(...).timeline():
                print(f"{start:6.2f}  {length:4.2f}  {name}")

        What is on screen at 0:42, and how long each beat actually lasts —
        the two questions you have when a video feels wrong. Pair it with
        :meth:`frame_at` to look at the moment you find.
        """
        return [tuple(row) for row in call("timeline", {**self._film()})]

    def chapters(self):
        """Where each part of the film starts, as `(start, name)`:
        `emit("water", chapter="Water")` starts one as that beat begins.

        A film is hundreds of events — a highlighted word is one — so the
        Previewer cannot show a picture of each. A chapter is the author saying
        where a part begins, which nothing downstream could work out.
        """
        return [tuple(row) for row in call("chapters", {**self._film()})]

    def covered(self):
        """Every label something was drawn on: `(seconds, above, below)`.

        Checked once per Trace Event, on the picture the event settles into
        (ADR 0017). A collision that lasts is reported where it starts, not
        at every beat it survives.
        """
        return [tuple(row) for row in call("film_covered", {**self._film()})]

    def report(self):
        """:meth:`covered`, worded — what `render` prints when it is not empty."""
        return call("report", {**self._film()})


class Rule:
    """How things matching ``pattern`` change. First matching rule wins.

        cm.Rule("*", position="lift_carry_drop", clearance=90)
        cm.Rule("bar/*", ease="spring", stagger=0.08)
        cm.Rule("card", position="arc", bend=-0.4, ease="out")

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
    * ``arc`` — a curve bowed to one side. ``bend`` is how far, as a fraction of
      the distance travelled (default 0.3); negative bows the other way.

    Timing and feel, for everything about the shape — where it is, its size,
    colour, opacity, turn, and a marker's reveal — not only its position:

    * ``ease`` — ``linear``, ``in``, ``out``, ``in_out``, ``back`` (overshoots and
      settles), ``spring`` (takes ``bounciness`` 0..1) or ``bounce``. Unset, a
      shape moves as it always has: eased position, the rest linear.
    * ``delay`` — wait this fraction of the beat before starting.
    * ``stagger`` — matching shapes start this fraction of the beat apart, in
      ``order``: ``screen`` (left to right, then top to bottom; the default),
      ``reverse``, ``name`` or ``centre`` (from the middle out). They all take
      the same time, and the last one finishes as the beat ends.
    """

    def __init__(self, pattern, position="straight", *, ease=None, order="screen", **options) -> None:
        self._set(call("rule", {"pattern": _key(pattern), "position": _string(position), "ease": _string(ease), "order": _string(order), "options": options}))

    @classmethod
    def _of(cls, made: dict) -> "Rule":
        """One the Authoring Core has already made."""
        value = cls.__new__(cls)
        value._set(made)
        return value

    def _set(self, made: dict) -> None:
        self._fields = {k: v for k, v in made.items() if k != "payload"}
        self._made = tuple(made["payload"])
        self.pattern = made["pattern"]
        self.position = made["position"]
        self.ease = made["ease"]
        self.order = made["order"]
        self.options = made["options"]

    def _payload(self) -> tuple:
        """What the Engine takes."""
        return self._made

    def __repr__(self) -> str:
        return "Rule(" + f"pattern={self.pattern!r}" + ", " + f"position={self.position!r}" + ", " + f"ease={self.ease!r}" + ", " + f"order={self.order!r}" + ", " + f"options={self.options!r}" + ")"


class TextPresetStyle:
    """Text appearance and travel distance shared by a preset's units.

    ``size`` is in canvas points. ``font`` and ``fill`` are optional, and
    ``distance`` is how far each word or letter starts from its final place.
    """

    def __init__(self, *, size=16.0, font=None, fill=None, distance=48.0) -> None:
        self._set(call("text_preset_style", {"size": _number(size), "font": _string(font), "fill": _string(fill), "distance": _number(distance)}))

    @classmethod
    def _of(cls, made: dict) -> "TextPresetStyle":
        """One the Authoring Core has already made."""
        value = cls.__new__(cls)
        value._set(made)
        return value

    def _set(self, made: dict) -> None:
        self._fields = {k: v for k, v in made.items() if k != "payload"}
        self.size = made["size"]
        self.font = made["font"]
        self.fill = made["fill"]
        self.distance = made["distance"]

    def __repr__(self) -> str:
        return "TextPresetStyle(" + f"size={self.size!r}" + ", " + f"font={self.font!r}" + ", " + f"fill={self.fill!r}" + ", " + f"distance={self.distance!r}" + ")"


class TextPreset:
    """Draw text as individually moving words or letters.

    Use a named helper such as ``codimate.text.words_up`` for a fixed motion.
    This class is the configurable form when the preset name is data::

        title = cm.TextPreset(
            "title", "Hello world", preset, style=cm.TextPresetStyle(size=72)
        )

        def view(frame):
            scene = cm.Scene()
            title.draw(scene, state="shown" if frame.state.ready else "hidden")
            return scene

        cm.explain(..., motion=title.rules(stagger=0.12))

    ``preset`` is one of ``words_up``, ``words_down``, ``letters_up``,
    ``letters_down``, ``words_left``, ``words_right``, or ``words_up_blur``.

    ``state`` is ``"hidden"`` or ``"shown"``. For ``words_up_blur``, it may
    also be ``"blurred"``; switch from that state to ``"shown"`` to resolve the
    soft halo into sharp text. The halo uses small translucent text copies, so
    it needs no image or optional imaging dependency.

    ``at`` is the text block's centre (a point or Slot); line breaks are
    centred independently. ``letters_*`` keeps combining marks and joined
    emoji together as one moving unit.
    """

    def __init__(self, name, content, preset, *, style=None, at=None) -> None:
        self._set(call("text_preset", {"name": _string(name), "content": _text(content), "preset": _string(preset), "style": _style(style), "at": _at(at)}))

    @classmethod
    def _of(cls, made: dict) -> "TextPreset":
        """One the Authoring Core has already made."""
        value = cls.__new__(cls)
        value._set(made)
        return value

    def _set(self, made: dict) -> None:
        self._fields = {k: v for k, v in made.items() if k != "payload"}
        self.name = made["name"]
        self.content = made["content"]
        self.preset = made["preset"]
        self.style = None if made["style"] is None else TextPresetStyle._of(made["style"])
        self.at = made["at"]

    def __repr__(self) -> str:
        return "TextPreset(" + f"name={self.name!r}" + ", " + f"content={self.content!r}" + ", " + f"preset={self.preset!r}" + ", " + f"style={self.style!r}" + ", " + f"at={self.at!r}" + ")"

    def draw(self, group, *, state="hidden"):
        """Draw this preset into a Scene or Group at one of its stable states."""
        return group._adopt(call("text_preset_draw", {"preset": self._fields, "film_font": _film_font(), "group": _group(group), "state": _string(state)}))

    def rules(self, *, ease="out", stagger=0.12, order="screen"):
        """Return motion rules for this preset's words, letters, and blur."""
        return tuple(Rule._of(made) for made in call("text_preset_rules", {"preset": self._fields, "ease": _string(ease), "stagger": _number(stagger), "order": _string(order)}))


class Axes:
    """A map from your numbers to pixels. Built by :func:`axes`.

    It draws nothing you did not ask it to and owns none of your names — see
    :meth:`at`. Its own shapes (frame, ticks, labels) are drawn by
    :meth:`draw`, under names you choose the prefix of.
    """

    def __init__(self, *, x, y, at=None, size=None, within=None) -> None:
        self._set(call("axes", {"canvas": _canvas(), "x": _pair(x), "y": _pair(y), "at": _at(at), "size": _radius(size), "within": _slot(within)}))

    @classmethod
    def _of(cls, made: dict) -> "Axes":
        """One the Authoring Core has already made."""
        value = cls.__new__(cls)
        value._set(made)
        return value

    def _set(self, made: dict) -> None:
        self._fields = {k: v for k, v in made.items() if k != "payload"}
        self.x0 = made["x0"]
        self.x1 = made["x1"]
        self.y0 = made["y0"]
        self.y1 = made["y1"]
        self.left = made["left"]
        self.top = made["top"]
        self.w = made["w"]
        self.h = made["h"]
        self.ink = made["ink"]
        self.label = made["label"]
        self.layer = made["layer"]
        self.right = made["right"]
        self.bottom = made["bottom"]

    def __repr__(self) -> str:
        return "Axes(" + f"x0={self.x0!r}" + ", " + f"x1={self.x1!r}" + ", " + f"y0={self.y0!r}" + ", " + f"y1={self.y1!r}" + ", " + f"left={self.left!r}" + ", " + f"top={self.top!r}" + ", " + f"w={self.w!r}" + ", " + f"h={self.h!r}" + ", " + f"ink={self.ink!r}" + ", " + f"label={self.label!r}" + ", " + f"layer={self.layer!r}" + ", " + f"right={self.right!r}" + ", " + f"bottom={self.bottom!r}" + ")"

    def looks(self, ink=None, label=None, layer=None):
        """Restyle the frame, ticks and labels. Returns a new Axes.

            plot = cm.axes(x=(0, 10), y=(0, 5)).looks(ink="#334", label=16)

        A separate call rather than more arguments on :meth:`draw`, which the
        shapes made the same choice about: no call in this library takes more
        than five things.
        """
        return Axes._of(call("axes_looks", {"axes": self._fields, "ink": _string(ink), "label": _number(label), "layer": _number(layer)}))

    def at(self, x, y):
        """One data point as a pixel pair, for any shape's ``at=``.

            scene.circle("dot", r=8, at=plot.at(2.0, 4.0))

        Handing back pixels rather than drawing is the whole design: what you
        do with them is an ordinary shape with a name of yours, so it tweens,
        `focus` frames it, and a motion Rule can be aimed at it.
        """
        return _as_size(call("axes_at", {"axes": self._fields, "x": _number(x), "y": _number(y)}))

    def line(self, f, steps=200, over=None):
        """``steps + 1`` points along ``y = f(x)``, as pixels.

            scene.curve("f", plot.line(lambda x: x * x), w=4).fill("red")

        ``over=(lo, hi)`` samples part of the range instead of all of it.

        An open `curve` is *drawn* rather than filled, so its colour is
        ``fill()`` and its thickness is ``w=`` — ``fill("none", edge=...)``
        draws nothing at all.

        Every sample is returned, including any that fall outside the box.
        Dropping them would be prettier and is wrong: two curves with different
        point counts do not interpolate (ADR 0010), so a curve that shed a
        point as it left the frame would stop animating. Keep the count and
        clamp the range with ``over=`` if you need it inside.
        """
        xs = call("axes_samples", {"axes": self._fields, "steps": _int(steps), "over": _pair(over)})
        return _as_points(call("axes_points", {"axes": self._fields, "xs": xs, "ys": [float(f(x)) for x in xs]}))

    def ticks(self, axis="x", about=6):
        """``(n, value, label)`` per tick, where ``n`` is the step multiple.

        ``n`` is what a tick is named after, not the value. A float that drifts
        by one part in a billion is a different name, and a renamed shape
        leaves and re-enters — which is a fade, on a picture that did not move.
        An integer count of steps cannot drift.
        """
        return [tuple(tick) for tick in call("axes_ticks", {"axes": self._fields, "axis": _string(axis), "about": _int(about)})]

    def draw(self, group, name="plot", *, about=6, grid=False):
        """Draw the frame, ticks and labels. Returns the Axes, for chaining.

            plot = cm.axes(x=(-4, 4), y=(-2, 6)).draw(scene)

        Every shape is named ``(name, ...)``, so two plots on one canvas do not
        collide and you can restyle or omit any of them by drawing your own.
        Colours and sizes come from :meth:`looks`.
        """
        group._adopt(call("axes_draw", {"axes": self._fields, "film_font": _film_font(), "group": _group(group), "name": _key(name), "about": _int(about), "grid": _bool(grid)}))
        return self


def words_up(name, content, *, style=None, at=None) -> "TextPreset":
    """Create a preset that brings words up into place."""
    return TextPreset(name, content, 'words_up', style=style, at=at)


def words_down(name, content, *, style=None, at=None) -> "TextPreset":
    """Create a preset that brings words down into place."""
    return TextPreset(name, content, 'words_down', style=style, at=at)


def letters_up(name, content, *, style=None, at=None) -> "TextPreset":
    """Create a preset that brings letter clusters up into place."""
    return TextPreset(name, content, 'letters_up', style=style, at=at)


def letters_down(name, content, *, style=None, at=None) -> "TextPreset":
    """Create a preset that brings letter clusters down into place."""
    return TextPreset(name, content, 'letters_down', style=style, at=at)


def words_left(name, content, *, style=None, at=None) -> "TextPreset":
    """Create a preset that brings words in from the left."""
    return TextPreset(name, content, 'words_left', style=style, at=at)


def words_right(name, content, *, style=None, at=None) -> "TextPreset":
    """Create a preset that brings words in from the right."""
    return TextPreset(name, content, 'words_right', style=style, at=at)


def words_up_blur(name, content, *, style=None, at=None) -> "TextPreset":
    """Create a preset that brings soft, blurred words up into focus."""
    return TextPreset(name, content, 'words_up_blur', style=style, at=at)


def axes(*, x, y, at=None, size=None, within=None) -> "Axes":
    """A coordinate map from your numbers to pixels.

        plot = cm.axes(x=(-4, 4), y=(-2, 6), size=(760, 420)).draw(scene)
        scene.curve("f", plot.line(lambda t: t * t), w=4).fill("orange")
        scene.circle("dot", r=8, at=plot.at(t, t * t))

    ``x`` and ``y`` are ``(low, high)``. ``size`` is ``(w, h)`` in pixels, or
    one number for a square; it defaults to most of the canvas. ``at`` places
    the box's centre, and ``within=slot`` fits it to part of the canvas.

    It returns points rather than drawing, so what you plot is a shape with a
    name of yours — which is what lets it tween, be framed by `focus`, and be
    aimed at by a motion Rule. See ADR 0016.
    """
    return Axes(x=x, y=y, at=at, size=size, within=within)


def ngon(sides, r, *, at=(0.0, 0.0), turn=0.0):
    """The corners of a regular polygon, for `Group.polygon`.

        scene.polygon("tri", cm.ngon(3, r=60, at=(640, 360)))

    A triangle is three sides, a hexagon six. ``turn`` rotates it in degrees —
    the first corner otherwise points straight up.

    Returns points rather than drawing, so it composes: you can shift them,
    hand them to `polygon`, or measure them yourself.
    """
    return _as_points(call("ngon", {"sides": _int(sides), "r": _number(r), "at": _point(at), "turn": _number(turn)}))


def star(points, r, *, inner=None, at=(0.0, 0.0), turn=0.0):
    """The corners of a star, for `Group.polygon`.

        scene.polygon("s", cm.star(5, r=80, at=(640, 360))).fill("yellow")

    ``inner`` is the radius of the valleys; it defaults to a proportion that
    looks like a star rather than a gear.
    """
    return _as_points(call("star", {"points": _int(points), "r": _number(r), "inner": _number(inner), "at": _point(at), "turn": _number(turn)}))


def row(items, *, gap=40.0, size=None, at=None, within=None):
    """One Slot per item, evenly spaced and centred on the canvas.

        for slot, item in cm.row(values, gap=40, size=190):
            ...

    ``size`` is one number for the Slot's width (its height follows), or
    ``(w, h)`` for both.

    A row lays things out on a shared baseline, so its Slots anchor at
    bottom-centre — hand one straight to ``scene.group()``.

    Pass ``within=slot`` to divide up part of the canvas instead of all of it,
    so a chart can live in its own corner without any arithmetic of yours.

    Yields ``(slot, item)`` pairs, or bare Slots if you passed a count.
    """
    items = items if isinstance(items, int) else list(items)
    return _paired(items, call("row", {"canvas": _canvas(), "items": _items(items), "gap": _number(gap), "size": _radius(size), "at": _at(at), "within": _slot(within)}))


def column(items, *, gap=40.0, size=None, at=None, within=None):
    """One Slot per item, stacked vertically and centred on the canvas.

        for slot in cm.column(4, gap=40, at=cm.at(x=640)):
            ...

    A column stacks things around a centre line, so its Slots anchor at their
    centre. ``x`` places the column horizontally; it defaults to mid-canvas.

    Pass ``within=slot`` to stack inside part of the canvas instead of all
    of it.

    Yields ``(slot, item)`` pairs, or bare Slots if you passed a count.
    """
    items = items if isinstance(items, int) else list(items)
    return _paired(items, call("column", {"canvas": _canvas(), "items": _items(items), "gap": _number(gap), "size": _radius(size), "at": _at(at), "within": _slot(within)}))


def measure(text, size=16.0, font=None):
    """How wide and tall ``text`` will be at ``size``: ``(w, h)``.

    For drawing a box around a label without guessing::

        w, h = cm.measure(label, size=30)
        scene.rect("box", w=w + 24, h=h + 12, at=(x, y)).round(6)
        scene.text("label", label, size=30, at=(x, y))

    Measured by the engine with the real fonts, including fallback, so it is
    right for Khmer and anything else that is not plain ASCII — which is why
    estimating ``len(text) * size * k`` is not good enough.

    The height is the line height, the same for "cat" and "Qgy", so a row of
    boxes lines up instead of jittering with whatever letters it holds.

    ``font`` is measured in, as for ``scene.text``; unset, the film's font.
    """
    return _as_size(call("measure", {"film_font": _film_font(), "text": _text(text), "size": _number(size), "font": _string(font)}))


def ease(t):
    """The easing curve the Engine applies between two moments.

        cm.ease(0.5)  ->  0.5

    This calls into the Engine, so it is the same curve your animation is
    actually using — not a copy of it.
    """
    return call("ease", {"t": _number(t)})
