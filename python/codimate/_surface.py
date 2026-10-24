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
        """A rectangle. Say where with a point, a Slot, or `at(...)`. Round the corners with `.round(12)`."""
        return self._build("rect", {"group": self._frame(), "key": _key(key), "h": _number(h), "w": _number(w), "at": _at(at)})

    def circle(self, key, *, r, at=None) -> "Handle":
        """A filled circle. `fill("none", edge=...)` draws a ring."""
        return self._build("circle", {"group": self._frame(), "key": _key(key), "r": _number(r), "at": _at(at)})

    def text(self, key, content, *, size=16, font=None, at=None) -> "Handle":
        """Text, centred horizontally. `font` is a font file or an installed font's name, for this text only; unset, the film's font."""
        return self._build("text", {"group": self._frame(), "film_font": _film_font(), "key": _key(key), "content": _text(content), "size": _number(size), "font": _string(font), "at": _at(at)})

    def line(self, key, *, start, end, w=2.0) -> "Handle":
        """A line between two points. `w` is its thickness. Each end may be a Slot (a line joins the middles of two places) or a point."""
        return self._build("line", {"group": self._frame(), "key": _key(key), "start": _point(start), "end": _point(end), "w": _number(w)})

    def arrow(self, key, *, start, end, w=4.0, head=16.0) -> "Handle":
        """An arrow, as a single filled shape: one name, travelling as one thing. `w` is the shaft, `head` the point."""
        return self._build("arrow", {"group": self._frame(), "key": _key(key), "start": _point(start), "end": _point(end), "w": _number(w), "head": _number(head)})

    def polygon(self, key, points, *, closed=True) -> "Handle":
        """A shape with corners. `points` is a sequence of points. Closed and filled by default; `closed=False` leaves an open outline. Two polygons tween only if they have the same number of corners."""
        return self._build("polygon", {"group": self._frame(), "key": _key(key), "points": _points(points), "closed": _bool(closed)})

    def curve(self, key, points, *, w=2.0, closed=False) -> "Handle":
        """A smooth line through every one of `points`: samples, not control points. `w` is the stroke width. Open by default; `closed=True` makes a fillable shape. Two curves tween only if they have the same number of points."""
        return self._build("curve", {"group": self._frame(), "key": _key(key), "points": _points(points), "w": _number(w), "closed": _bool(closed)})

    def arc(self, key, *, r, sweep, at=None) -> "Handle":
        """A slice of a circle: an angle mark, a pie, a dial, an orbit. `sweep` is (start, end) in degrees, clockwise from twelve o'clock; `r` is the radius, or (rx, ry) for an ellipse. Open by default; `.round(1)` closes it into a pie slice."""
        return self._build("arc", {"group": self._frame(), "key": _key(key), "r": _radius(r), "sweep": _pair(sweep), "at": _at(at)})

    def group(self, key, slot=None, *, at=None, anchor=None, w=None) -> "Group":
        """A place to draw a thing made of several shapes. Give it a Slot and it sits where the Slot says (at `anchor`: center, top or bottom), or place it with `at` like any shape. Inside it, 0 is that point, and everything drawn on it moves as one."""
        return self._nest(call("group", {"group": self._frame(), "key": _key(key), "slot": _slot(slot), "at": _at(at), "anchor": _string(anchor), "w": _number(w)}))

    def formula(self, key, latex, *, size=16.0, at=None) -> "Handle":
        """Real mathematics, written as LaTeX. `size` means what it means for text. Draw it on with `write(reveal=, pen=)`. Typeset once when the film is built."""
        return self._build("formula", {"group": self._frame(), "key": _key(key), "latex": _string(latex), "size": _number(size), "at": _at(at)})

    def image(self, key, file, *, size=None, at=None) -> "Handle":
        """A picture (PNG or JPEG) drawn into the frame. `size` is a box it fits inside, one number or (w, h), aspect always kept; leave it out and it is drawn at its own pixel size."""
        return self._build("image", {"group": self._frame(), "key": _key(key), "file": _text(file), "size": _radius(size), "at": _at(at)})

    def svg(self, key, file, *, size=120.0, at=None) -> "Handle":
        """Vector art from a file, drawn as real geometry: it tweens, turns, grows and draws on like anything else. `size` is a box it fits inside, one number or (w, h). It keeps its own colours unless `fill` overrides them."""
        return self._build("svg", {"group": self._frame(), "key": _key(key), "file": _text(file), "size": _radius(size), "at": _at(at)})

    def marker(self, key, *, w, h, at=None) -> "Handle":
        """A highlighter stroke over any box on screen. `.write(reveal=0..1)` draws it on from its left end; `.turn(degrees)` sets its angle; `.fill(name)` colours it: yellow, orange, pink, green, blue or purple, or any #rrggbb. Slightly translucent, under the words' layer, drawn by hand the same way every time for the same name."""
        return self._keep_marked(call("marker", {"group": self._frame(), "key": _key(key), "w": _number(w), "h": _number(h), "at": _at(at)}))

    def highlight(self, key, content, *, size=16.0, font=None, at=None) -> "tuple[Handle, Handle]":
        """Words under a highlighter stroke, the stroke sized to them: gives (band, words). Marked, the words are near-black; `band.write(reveal=...)` blends them back to their own colour (white unless `words.fill(...)` says) as the stroke rubs out."""
        return self._keep_marked(call("highlight", {"group": self._frame(), "film_font": _film_font(), "key": _key(key), "content": _text(content), "size": _number(size), "font": _string(font), "at": _at(at)}))


class _HandleSurface:
    def fill(self, color="white", *, edge=None, edge_w=None) -> "Handle":
        """Colour it. `edge` outlines it; `color="none"` leaves it unfilled. Saying nothing about the outline leaves it alone."""
        self._change("fill", {"shape": self._shape(), "color": _string(color), "edge": _string(edge), "edge_w": _number(edge_w)})
        return self

    def turn(self, degrees, pivot="center") -> "Handle":
        """Rotate it. `pivot` is center, top, bottom, left or right. Text moves but does not turn."""
        self._change("turn", {"shape": self._shape(), "degrees": _number(degrees), "pivot": _string(pivot)})
        return self

    def grow(self, scale) -> "Handle":
        """Scale it: a number for both axes, or (sx, sy) to stretch."""
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
        """How much of a formula or an imported SVG shows (`reveal`, 0 to 1), and whether a pen draws it on (`pen`, its width). On a marker, how far its stroke has been drawn on."""
        self._change("write", {"shape": self._shape(), "reveal": _number(reveal), "pen": _number(pen)})
        return self


class _SceneSurface:
    def focus(self, *names, pad=40.0, least=240.0) -> "None":
        """Look at these things; the camera works out where to stand. No names frames the whole canvas again. `pad` is room around them; `least` is the smallest thing the camera fills the frame with. Anything on an overlay stays where it is put."""
        self._aim(call("focus", {"names": _keys(names), "pad": _number(pad), "least": _number(least)}))

    def overlay(self) -> "Group":
        """A place for things the camera does not move: titles and narration. The diagram pans and zooms underneath; whatever is drawn here stays put, in plain screen coordinates."""
        return self._open_overlay(call("overlay", {"group": self._frame()}))


class Rule:
    """How things matching `pattern` change; the first matching rule wins. `position` is the path: straight, linear, fall, lift_carry_drop or arc. `ease` is linear, in, out, in_out, back, spring or bounce. `delay` and `stagger` are fractions of the beat; `order` is screen, reverse, name or centre."""

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
    """Text appearance and travel distance shared by a preset's words or letters. `size` is in canvas points; `font` and `fill` are optional; `distance` is how far each starts from its place."""

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
    """Text drawn as individually moving words or letters. `preset` is words_up, words_down, letters_up, letters_down, words_left, words_right or words_up_blur. Draw it in a state (hidden or shown; blurred too for words_up_blur) and pass its rules() to explain."""

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
        """The motion rules for this preset's words or letters, and its blur."""
        return tuple(Rule._of(made) for made in call("text_preset_rules", {"preset": self._fields, "ease": _string(ease), "stagger": _number(stagger), "order": _string(order)}))


class Axes:
    """A map from your numbers to pixels. `x` and `y` are (low, high); `size` is (w, h) in pixels, or one number for a square, defaulting to most of the canvas; `at` places the box's centre, and `within` fits it to part of the canvas. It returns points rather than drawing, so what you plot is a shape with a name of yours."""

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
        """Restyle the frame, ticks and labels. Returns a new Axes."""
        return Axes._of(call("axes_looks", {"axes": self._fields, "ink": _string(ink), "label": _number(label), "layer": _number(layer)}))

    def at(self, x, y):
        """One data point as a pixel pair, for any shape's `at`."""
        return _as_size(call("axes_at", {"axes": self._fields, "x": _number(x), "y": _number(y)}))

    def line(self, f, steps=200, over=None):
        """steps + 1 points along y = f(x), as pixels, for a curve. `over=(lo, hi)` samples part of the range. Every sample is kept, even outside the box: curves only tween when their point counts match."""
        xs = call("axes_samples", {"axes": self._fields, "steps": _int(steps), "over": _pair(over)})
        return _as_points(call("axes_points", {"axes": self._fields, "xs": xs, "ys": [float(f(x)) for x in xs]}))

    def ticks(self, axis="x", about=6):
        """(n, value, label) per tick, where n, the step multiple, is what a tick is named after: an integer cannot drift."""
        return [tuple(tick) for tick in call("axes_ticks", {"axes": self._fields, "axis": _string(axis), "about": _int(about)})]

    def draw(self, group, name="plot", *, about=6, grid=False):
        """Draw the frame, ticks and labels, each named under `name`. Returns the Axes, for chaining. Colours and sizes come from `looks`."""
        group._adopt(call("axes_draw", {"axes": self._fields, "film_font": _film_font(), "group": _group(group), "name": _key(name), "about": _int(about), "grid": _bool(grid)}))
        return self


def words_up(name, content, *, style=None, at=None) -> "TextPreset":
    """A text preset that brings words up into place."""
    return TextPreset(name, content, 'words_up', style=style, at=at)


def words_down(name, content, *, style=None, at=None) -> "TextPreset":
    """A text preset that brings words down into place."""
    return TextPreset(name, content, 'words_down', style=style, at=at)


def letters_up(name, content, *, style=None, at=None) -> "TextPreset":
    """A text preset that brings letter clusters up into place."""
    return TextPreset(name, content, 'letters_up', style=style, at=at)


def letters_down(name, content, *, style=None, at=None) -> "TextPreset":
    """A text preset that brings letter clusters down into place."""
    return TextPreset(name, content, 'letters_down', style=style, at=at)


def words_left(name, content, *, style=None, at=None) -> "TextPreset":
    """A text preset that brings words in from the left."""
    return TextPreset(name, content, 'words_left', style=style, at=at)


def words_right(name, content, *, style=None, at=None) -> "TextPreset":
    """A text preset that brings words in from the right."""
    return TextPreset(name, content, 'words_right', style=style, at=at)


def words_up_blur(name, content, *, style=None, at=None) -> "TextPreset":
    """A text preset that brings soft, blurred words up into focus."""
    return TextPreset(name, content, 'words_up_blur', style=style, at=at)


def axes(*, x, y, at=None, size=None, within=None) -> "Axes":
    """A coordinate map from your numbers to pixels: see Axes. `x` and `y` are (low, high)."""
    return Axes(x=x, y=y, at=at, size=size, within=within)


def ngon(sides, r, *, at=(0.0, 0.0), turn=0.0):
    """The corners of a regular polygon, for `polygon`. A triangle is three sides, a hexagon six; `turn` rotates it in degrees, the first corner otherwise pointing straight up."""
    return _as_points(call("ngon", {"sides": _int(sides), "r": _number(r), "at": _point(at), "turn": _number(turn)}))


def star(points, r, *, inner=None, at=(0.0, 0.0), turn=0.0):
    """The corners of a star, for `polygon`. `inner` is the radius of the valleys; it defaults to a proportion that looks like a star rather than a gear."""
    return _as_points(call("star", {"points": _int(points), "r": _number(r), "inner": _number(inner), "at": _point(at), "turn": _number(turn)}))


def row(items, *, gap=40.0, size=None, at=None, within=None):
    """One Slot per item, evenly spaced on a shared baseline, anchored at bottom-centre. `size` is the Slot's width (its height follows) or (w, h). `within` divides part of the canvas instead of all of it. Gives (slot, item) pairs, or bare Slots for a count."""
    items = items if isinstance(items, int) else list(items)
    return _paired(items, call("row", {"canvas": _canvas(), "items": _items(items), "gap": _number(gap), "size": _radius(size), "at": _at(at), "within": _slot(within)}))


def column(items, *, gap=40.0, size=None, at=None, within=None):
    """One Slot per item, stacked vertically around a centre line, anchored at their centres. `x` places the column; it defaults to mid-canvas. Gives (slot, item) pairs, or bare Slots for a count."""
    items = items if isinstance(items, int) else list(items)
    return _paired(items, call("column", {"canvas": _canvas(), "items": _items(items), "gap": _number(gap), "size": _radius(size), "at": _at(at), "within": _slot(within)}))


def measure(text, size=16.0, font=None):
    """How wide and tall `text` will be at `size`: (w, h). Measured by the Engine with the real fonts, fallback included, so it is right for Khmer and anything else that is not plain ASCII. The height is the line height, the same for every word."""
    return _as_size(call("measure", {"film_font": _film_font(), "text": _text(text), "size": _number(size), "font": _string(font)}))


def ease(t):
    """The easing curve the Engine applies between two moments: the same curve your animation uses, not a copy of it."""
    return call("ease", {"t": _number(t)})
