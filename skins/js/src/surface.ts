// Generated from api/surface.toml by tools/generate.py. Do not edit.
//
// The JavaScript Skin's side of the Interface Manifest: each method sends
// its arguments across the Waist and keeps what comes back.

import { filmFont, to, type At, type Frame, type Key, type Point, type ShapeData, type Slot } from "./waist.ts";
import { call, height, paired, width } from "./waist.ts";
import type { Character, Group, Handle } from "./scene.ts";

export abstract class GroupSurface {
  protected abstract _frame(): Frame;
  protected abstract _build(op: string, data: object): Handle;
  protected abstract _buildCharacter(op: string, data: object): Character;
  protected abstract _nest(frame: Frame): Group;
  protected abstract _keepMarked(answer: any): any;

  /**
   * A rectangle. Say where with a point, a Slot, or `cm.at(...)`.
   *
   *     scene.rect("box", h=40, at=cm.at(bottom=0)).fill("blue")
   *
   * Round the corners with `.round(12)`.
   */
  rect(key: Key, options: { h: number; w?: number; at?: At }): Handle {
    return this._build("rect", { group: this._frame(), key: to.key(key), h: to.number(options?.h), w: to.number(options?.w), at: to.at(options?.at) });
  }

  /** A filled circle. `color="none"` with `.fill(edge=...)` draws a ring. */
  circle(key: Key, options: { r: number; at?: At }): Handle {
    return this._build("circle", { group: this._frame(), key: to.key(key), r: to.number(options?.r), at: to.at(options?.at) });
  }

  /**
   * Text, centred horizontally. You never deal with baselines.
   *
   *     scene.text("title", "Bubble sort", size=48, font="fonts/Inter-Bold.ttf")
   *
   * ``font`` is a ``.ttf``/``.otf`` file or an installed font's name, for this
   * text only; unset, the film's font (``cm.font``).
   */
  text(key: Key, content: unknown, options?: { size?: number; font?: string; at?: At }): Handle {
    return this._build("text", { group: this._frame(), film_font: filmFont(), key: to.key(key), content: to.text(content), size: to.number((options?.size ?? 16)), font: to.string(options?.font), at: to.at(options?.at) });
  }

  /**
   * A line between two points. `w` is its thickness.
   *
   * A line has no centre to anchor, so it takes its two ends directly. Each
   * may be a Slot — a line joins the middles of two places — or an `(x, y)`.
   */
  line(key: Key, options: { start: Point; end: Point; w?: number }): Handle {
    return this._build("line", { group: this._frame(), key: to.key(key), start: to.point(options?.start), end: to.point(options?.end), w: to.number((options?.w ?? 2.0)) });
  }

  /**
   * An arrow, as a single filled shape.
   *
   * One shape rather than a line plus a separate head, so it carries one
   * name and travels as one thing. `w` is the shaft, `head` the point.
   */
  arrow(key: Key, options: { start: Point; end: Point; w?: number; head?: number }): Handle {
    return this._build("arrow", { group: this._frame(), key: to.key(key), start: to.point(options?.start), end: to.point(options?.end), w: to.number((options?.w ?? 4.0)), head: to.number((options?.head ?? 16.0)) });
  }

  /**
   * A shape with corners: a triangle, a wedge, an arrow head, a wing.
   *
   *     scene.polygon("tri", cm.ngon(3, r=50, at=(640, 360))).fill("green")
   *
   * `points` is a sequence of `(x, y)`. Closed and filled by default;
   * `closed=False` leaves an open outline, which shows only with an edge.
   *
   * Two polygons tween only if they have the same number of corners —
   * interpolating a triangle into a pentagon has no answer worth inventing,
   * so the later shape stands for the whole beat instead.
   */
  polygon(key: Key, points: readonly Point[], options?: { closed?: boolean }): Handle {
    return this._build("polygon", { group: this._frame(), key: to.key(key), points: to.points(points), closed: to.bool((options?.closed ?? true)) });
  }

  /**
   * A smooth line through every one of `points`.
   *
   *     scene.curve("plot", samples, w=3).fill("cyan")
   *
   * The points are on the curve, not control points: you hand it samples —
   * a function you plotted, a path something travelled — and it draws a
   * smooth line through them. `w` is the stroke width.
   *
   * Open by default, and an open curve is *drawn* rather than filled, the
   * way a line is. `closed=True` joins the ends and makes it a fillable
   * shape, like a polygon with rounded-off corners.
   *
   * Two curves tween only if they have the same number of points, the same
   * rule polygons follow; otherwise the later one stands for the beat.
   */
  curve(key: Key, points: readonly Point[], options?: { w?: number; closed?: boolean }): Handle {
    return this._build("curve", { group: this._frame(), key: to.key(key), points: to.points(points), w: to.number((options?.w ?? 2.0)), closed: to.bool((options?.closed ?? false)) });
  }

  /**
   * A slice of a circle — an angle mark, a pie, a dial, an orbit.
   *
   *     scene.arc("angle", r=90, sweep=(0, 50)).fill("none", edge="cyan",
   *                                                   edge_w=3)
   *     scene.arc("slice", r=120, sweep=(0, 120)).fill("orange").round(1)
   *
   * `sweep` is `(start, end)` in degrees, clockwise from twelve o'clock —
   * the same zero `cm.ngon` uses. `r` is the radius, or `(rx, ry)` for an
   * ellipse.
   *
   * Open by default, so it draws as a curved line. `.round(1)` closes it
   * back to the centre and makes a pie slice you can fill.
   *
   * Two arcs always tween, however far apart their angles are, so an angle
   * mark grows and a pie fills smoothly. That is what this cannot be done
   * with `curve`: a curve through points on a circle changes its point
   * count when the sweep changes, and then it snaps instead of sweeping.
   */
  arc(key: Key, options: { r: number | Point; sweep: readonly [number, number]; at?: At }): Handle {
    return this._build("arc", { group: this._frame(), key: to.key(key), r: to.radius(options?.r), sweep: to.pair(options?.sweep), at: to.at(options?.at) });
  }

  /**
   * A place to draw a thing made of several shapes.
   *
   *     bar = scene.group(item.id, slot)
   *     bar.rect("bar", h=item.value * 70, at=cm.at(bottom=0))
   *     bar.text("label", item.value, at=cm.at(top=20))
   *
   * Give it a Slot and it sits where the Slot says, or place it with ``at``
   * like any shape. Inside it, ``0`` is that point, and everything drawn on
   * it moves as one.
   */
  group(key: Key, slot?: Slot, options?: { at?: At; anchor?: string; w?: number }): Group {
    return this._nest(call("group", { group: this._frame(), key: to.key(key), slot: to.slot(slot), at: to.at(options?.at), anchor: to.string(options?.anchor), w: to.number(options?.w) }));
  }

  /**
   * Real mathematics, written as LaTeX.
   *
   *     scene.formula("eq", r"\frac{QK^{T}}{\sqrt{d_k}}", size=34)
   *
   * Use a raw string, or every backslash needs doubling. `size` means what
   * it means for :meth:`text`. Draw it on with `.write(reveal=, pen=)`.
   *
   * The result is glyph outlines, not a font — so it moves, fades and
   * recolours like any other shape, and is typeset once when the video is
   * built. Needs the `typst` binary on PATH. Rendering uses the bundled
   * `ffmpeg`.
   */
  formula(key: Key, latex: string, options?: { size?: number; at?: At }): Handle {
    return this._build("formula", { group: this._frame(), key: to.key(key), latex: to.string(latex), size: to.number((options?.size ?? 16.0)), at: to.at(options?.at) });
  }

  /**
   * A picture — a photo, a screenshot, a figure — drawn into the frame.
   *
   *     scene.image("paper", "figures/attention.png", size=(520, 300))
   *     scene.image("shot", "screen.png", size=400).on(opacity=0.4)
   *
   * PNG and JPEG, read by content rather than by extension. `size` is a box
   * it fits inside, one number or `(w, h)`, aspect always kept; leave it out
   * and the picture is drawn at its own pixel size.
   *
   * It moves, scales, turns and fades like anything else, and `focus()`
   * frames it. For a logo or a diagram prefer `svg`, which arrives as
   * geometry you can recolour and draw on — an image is pixels, so it can
   * only be placed.
   */
  image(key: Key, file: unknown, options?: { size?: number | Point; at?: At }): Handle {
    return this._build("image", { group: this._frame(), key: to.key(key), file: to.text(file), size: to.radius(options?.size), at: to.at(options?.at) });
  }

  /**
   * Vector art from a file, drawn as real geometry.
   *
   *     scene.svg("logo", "brand.svg", size=90, at=cm.at(x=1180, top=24))
   *     scene.svg("chart", "flow.svg", size=(900, 420))
   *
   * Because it arrives as geometry rather than pixels it behaves like
   * anything else you draw: it tweens, `.turn()` and `.grow()` transform
   * it, `focus()` frames it, and `.write(pen=2)` draws it on stroke by
   * stroke.
   *
   * `size` is a box it fits inside — one number for a square, or `(w, h)`.
   * The aspect ratio is always kept, so a wide diagram and a tall one both
   * land inside the box you named.
   *
   * The artwork keeps its own colours. `.fill(colour)` overrides all of
   * them at once, which flattens it to a silhouette on purpose.
   *
   * Labels come across as real text, shaped by the same code that draws
   * every `scene.text`. Layout that cannot be read is refused rather than
   * guessed at — a `tspan`, a `textPath`, or a turned label, since the
   * renderer cannot turn glyphs.
   */
  svg(key: Key, file: unknown, options?: { size?: number | Point; at?: At }): Handle {
    return this._build("svg", { group: this._frame(), key: to.key(key), file: to.text(file), size: to.radius((options?.size ?? 120.0)), at: to.at(options?.at) });
  }

  /**
   * A poseable figure from a rigged SVG.
   *
   *     guide = scene.character("guide", cm.characters.codimate, size=360,
   *                              at=(640, 360))
   *     guide.part("right-arm").turn(-25)
   *     guide.expression("thinking")
   *
   * Use `cm.characters.codimate` for the bundled mascot, or give the path to
   * your own SVG. Mark joint groups with an `id` and `data-pivot="x y"`; the
   * pivot is measured in the root SVG's `viewBox`. Nested joints carry their
   * children. Put named facial variants in groups with
   * `data-expression="name"`, and optionally set
   * `data-default-expression="name"` on the rig root. Set the pose in each
   * Scene; Codimate animates between them.
   */
  character(key: Key, file: unknown, options?: { size?: number | Point; at?: At }): Character {
    return this._buildCharacter("character", { group: this._frame(), key: to.key(key), file: to.text(file), size: to.radius(options?.size), at: to.at(options?.at) });
  }

  /**
   * A highlighter stroke over any box on screen.
   *
   *     scene.marker("note", w=260, h=70, at=(640, 300)).turn(-8)
   *     scene.marker("note", w=260, h=70, at=(640, 300)).write(reveal=0.5)
   *
   * It is not tied to text. `.write(reveal=0..1)` draws it on from its left
   * end, and `.turn(degrees)` sets its angle and so the way it draws. Colour
   * with `.fill(name)`: yellow, orange, pink, green, blue or purple, or any
   * `#rrggbb`. It is slightly translucent and sits under layer 10, where
   * the words go. Drawn by hand: straight edges, a slight tilt and
   * chisel-cut ends, the same every time for the same name.
   */
  marker(key: Key, options: { w: number; h: number; at?: At }): Handle {
    return this._keepMarked(call("marker", { group: this._frame(), key: to.key(key), w: to.number(options?.w), h: to.number(options?.h), at: to.at(options?.at) }));
  }

  /**
   * Words under a highlighter stroke, the stroke sized to them.
   *
   *     band, words = scene.highlight("pivot", "Pivot", size=30, at=(640, 360))
   *     band.write(reveal=0.0)       # rubbed out; the words turn white
   *     words.fill("#171717")        # on a white page, keep them dark
   *
   * A marker over the words' measured box, padded by a quarter of `size`
   * on every side, with the words drawn as their own text shape on top.
   * Marked, the words are near-black; `band.write(reveal=...)` blends them
   * back to their own colour as the stroke rubs out, in step with it. Their
   * own colour is white unless `words.fill(...)` says otherwise. `font` is
   * as for `text`, and the stroke is sized to the words in it. Give each
   * word its own name and keep its words fixed under that name: the stroke
   * can then move and draw on or off between words without the letters
   * overflowing it.
   */
  highlight(key: Key, content: unknown, options?: { size?: number; font?: string; at?: At }): [Handle, Handle] {
    return this._keepMarked(call("highlight", { group: this._frame(), film_font: filmFont(), key: to.key(key), content: to.text(content), size: to.number((options?.size ?? 16.0)), font: to.string(options?.font), at: to.at(options?.at) }));
  }
}

export abstract class HandleSurface {
  protected abstract _shape(): ShapeData;
  protected abstract _change(op: string, data: object): void;

  /**
   * Colour it. `edge` outlines it; `color="none"` leaves it unfilled.
   *
   * Saying nothing about the outline leaves the outline alone, so
   * recolouring a shape does not silently erase the edge it was given —
   * by an earlier `fill`, or by `curve`, which carries its stroke width
   * there.
   */
  fill(color?: string, options?: { edge?: string; edgeW?: number }): this {
    this._change("fill", { shape: this._shape(), color: to.string((color ?? "white")), edge: to.string(options?.edge), edge_w: to.number(options?.edgeW) });
    return this;
  }

  /**
   * Rotate it. `pivot` is center, top, bottom, left or right.
   *
   * Text moves but does not turn — rotating glyphs is renderer work that
   * has not been done.
   */
  turn(degrees: number, pivot?: string): this {
    this._change("turn", { shape: this._shape(), degrees: to.number(degrees), pivot: to.string((pivot ?? "center")) });
    return this;
  }

  /** Scale it: a number for both axes, or `(sx, sy)` to stretch. */
  grow(scale: number | Point): this {
    this._change("grow", { shape: this._shape(), scale: to.radius(scale) });
    return this;
  }

  /** Which layer it draws on, and how solid it is. */
  on(layer?: number, opacity?: number): this {
    this._change("on", { shape: this._shape(), layer: to.number(layer), opacity: to.number(opacity) });
    return this;
  }

  /** Round a rectangle's corners, clamped to half its short side. */
  round(radius: number): this {
    this._change("round", { shape: this._shape(), radius: to.number(radius) });
    return this;
  }

  /**
   * How much of a formula or an imported SVG shows, and whether a pen
   * draws it on. On a marker, how far its stroke has been drawn on.
   *
   * The pen rides in a different field for the two kinds — `w` for a
   * formula, `size` for an SVG, whose `w` is already the box it fits
   * inside. The payload reuses fields per kind by design (ADR 0008); this
   * is the one place that reuse is visible from Python.
   */
  write(reveal?: number, pen?: number): this {
    this._change("write", { shape: this._shape(), reveal: to.number(reveal), pen: to.number((pen ?? 0.0)) });
    return this;
  }
}

/** A Scene's own methods, mixed into a Group class. */
export function SceneSurface<B extends abstract new (...args: any[]) => any>(Base: B) {
  abstract class SceneMethods extends Base {
    protected abstract _frame(): Frame;
    protected abstract _aim(aim: unknown): void;
    protected abstract _openOverlay(frame: Frame): Group;

    /**
     * Look at these things. The camera works out where to stand.
     *
     *     scene.focus("cat")                    # frame this shape
     *     scene.focus(("cell", 2, 1), pad=60)   # any name, with room around it
     *     scene.focus("The", "cat", "sat")      # several — all of them in shot
     *     scene.focus()                         # the whole canvas again
     *
     * You never write a camera coordinate. The Engine knows where everything
     * is and how big it is, so it derives the framing — and because each
     * moment is framed on its own, a camera that moves between two moments is
     * just two sets of coordinates that differ, which the usual tween
     * animates. Camera movement needs nothing new.
     *
     * Naming rather than positioning also means the shot survives a layout
     * change: move the shape and the camera follows, because the name is the
     * part that is stable.
     *
     * ``least`` is the smallest thing the camera will fill the frame with, so
     * focusing on a full stop does not magnify it into abstraction.
     *
     * Anything on an :meth:`overlay` stays where it is put.
     */
    focus(names: readonly Key[], options?: { pad?: number; least?: number }): void {
      this._aim(call("focus", { names: to.keys(names), pad: to.number((options?.pad ?? 40.0)), least: to.number((options?.least ?? 240.0)) }));
    }

    /**
     * A place for things the camera does not move.
     *
     *     hud = scene.overlay()
     *     hud.text("title", "Scaled dot-product attention", x=640, y=52)
     *
     * The diagram pans and zooms underneath; whatever is drawn here stays
     * put. Titles and narration belong here — the moment the picture zooms, a
     * caption that zooms with it is wrong, and usually off the edge of the
     * frame entirely.
     *
     * Content is in the world unless it says otherwise, because in an
     * explanation most of the frame is diagram and only a little is
     * narration.
     */
    overlay(): Group {
      return this._openOverlay(call("overlay", { group: this._frame() }));
    }
  }
  return SceneMethods;
}

export abstract class ExplanationSurface {
  protected abstract _film(): object;

  /**
   * Every beat as ``(start, duration, event name)``, in seconds.
   *
   *     for start, length, name in cm.explain(...).timeline():
   *         print(f"{start:6.2f}  {length:4.2f}  {name}")
   *
   * What is on screen at 0:42, and how long each beat actually lasts —
   * the two questions you have when a video feels wrong. Pair it with
   * :meth:`frame_at` to look at the moment you find.
   */
  timeline(): any[][] {
    return call("timeline", { ...this._film() });
  }

  /**
   * Where each part of the film starts, as `(start, name)`:
   * `emit("water", chapter="Water")` starts one as that beat begins.
   *
   * A film is hundreds of events — a highlighted word is one — so the
   * Previewer cannot show a picture of each. A chapter is the author saying
   * where a part begins, which nothing downstream could work out.
   */
  chapters(): any[][] {
    return call("chapters", { ...this._film() });
  }

  /**
   * Every label something was drawn on: `(seconds, above, below)`.
   *
   * Checked once per Trace Event, on the picture the event settles into
   * (ADR 0017). A collision that lasts is reported where it starts, not
   * at every beat it survives.
   */
  covered(): any[][] {
    return call("film_covered", { ...this._film() });
  }

  /** :meth:`covered`, worded — what `render` prints when it is not empty. */
  report(): string {
    return call("report", { ...this._film() });
  }
}

/**
 * The corners of a regular polygon, for `Group.polygon`.
 *
 *     scene.polygon("tri", cm.ngon(3, r=60, at=(640, 360)))
 *
 * A triangle is three sides, a hexagon six. ``turn`` rotates it in degrees —
 * the first corner otherwise points straight up.
 *
 * Returns points rather than drawing, so it composes: you can shift them,
 * hand them to `polygon`, or measure them yourself.
 */
export function ngon(sides: number, r: number, options?: { at?: Point; turn?: number }): [number, number][] {
  return call("ngon", { sides: to.int(sides), r: to.number(r), at: to.point((options?.at ?? [0, 0])), turn: to.number((options?.turn ?? 0.0)) });
}

/**
 * The corners of a star, for `Group.polygon`.
 *
 *     scene.polygon("s", cm.star(5, r=80, at=(640, 360))).fill("yellow")
 *
 * ``inner`` is the radius of the valleys; it defaults to a proportion that
 * looks like a star rather than a gear.
 */
export function star(points: number, r: number, options?: { inner?: number; at?: Point; turn?: number }): [number, number][] {
  return call("star", { points: to.int(points), r: to.number(r), inner: to.number(options?.inner), at: to.point((options?.at ?? [0, 0])), turn: to.number((options?.turn ?? 0.0)) });
}

/**
 * One Slot per item, evenly spaced and centred on the canvas.
 *
 *     for slot, item in cm.row(values, gap=40, size=190):
 *         ...
 *
 * ``size`` is one number for the Slot's width (its height follows), or
 * ``(w, h)`` for both.
 *
 * A row lays things out on a shared baseline, so its Slots anchor at
 * bottom-centre — hand one straight to ``scene.group()``.
 *
 * Pass ``within=slot`` to divide up part of the canvas instead of all of it,
 * so a chart can live in its own corner without any arithmetic of yours.
 *
 * Yields ``(slot, item)`` pairs, or bare Slots if you passed a count.
 */
export function row(items: number | Iterable<unknown>, options?: { gap?: number; size?: number | Point; at?: At; within?: Slot }): any[] {
  return paired(items, call("row", { canvas: [width(), height()], items: to.items(items), gap: to.number((options?.gap ?? 40.0)), size: to.radius(options?.size), at: to.at(options?.at), within: to.slot(options?.within) }));
}

/**
 * One Slot per item, stacked vertically and centred on the canvas.
 *
 *     for slot in cm.column(4, gap=40, at=cm.at(x=640)):
 *         ...
 *
 * A column stacks things around a centre line, so its Slots anchor at their
 * centre. ``x`` places the column horizontally; it defaults to mid-canvas.
 *
 * Pass ``within=slot`` to stack inside part of the canvas instead of all
 * of it.
 *
 * Yields ``(slot, item)`` pairs, or bare Slots if you passed a count.
 */
export function column(items: number | Iterable<unknown>, options?: { gap?: number; size?: number | Point; at?: At; within?: Slot }): any[] {
  return paired(items, call("column", { canvas: [width(), height()], items: to.items(items), gap: to.number((options?.gap ?? 40.0)), size: to.radius(options?.size), at: to.at(options?.at), within: to.slot(options?.within) }));
}

/**
 * How wide and tall ``text`` will be at ``size``: ``(w, h)``.
 *
 * For drawing a box around a label without guessing::
 *
 *     w, h = cm.measure(label, size=30)
 *     scene.rect("box", w=w + 24, h=h + 12, at=(x, y)).round(6)
 *     scene.text("label", label, size=30, at=(x, y))
 *
 * Measured by the engine with the real fonts, including fallback, so it is
 * right for Khmer and anything else that is not plain ASCII — which is why
 * estimating ``len(text) * size * k`` is not good enough.
 *
 * The height is the line height, the same for "cat" and "Qgy", so a row of
 * boxes lines up instead of jittering with whatever letters it holds.
 *
 * ``font`` is measured in, as for ``scene.text``; unset, the film's font.
 */
export function measure(text: unknown, size?: number, font?: string): [number, number] {
  return call("measure", { film_font: filmFont(), text: to.text(text), size: to.number((size ?? 16.0)), font: to.string(font) });
}

/**
 * The easing curve the Engine applies between two moments.
 *
 *     cm.ease(0.5)  ->  0.5
 *
 * This calls into the Engine, so it is the same curve your animation is
 * actually using — not a copy of it.
 */
export function ease(t: number): number {
  return call("ease", { t: to.number(t) });
}

/**
 * How things matching ``pattern`` change. First matching rule wins.
 *
 *     cm.Rule("*", position="lift_carry_drop", clearance=90)
 *     cm.Rule("bar/*", ease="spring", stagger=0.08)
 *     cm.Rule("card", position="arc", bend=-0.4, ease="out")
 *
 * ``pattern`` matches a shape's full name with ``*`` and ``?``. A shape
 * inside a group is named ``group/child``, so ``"3/*"`` targets one group and
 * ``"*"`` targets everything.
 *
 * Paths:
 *
 * * ``straight`` — a straight line, easing in and out. The default, and what
 *   you want when each event is a distinct step.
 * * ``linear`` — a straight line at constant speed. Use it when a thing is
 *   mid-journey at every event, like something turning: easing would make it
 *   accelerate and stop inside each segment.
 * * ``fall`` — a parabola: sideways at a constant rate, downwards
 *   accelerating. What a dropped thing does, and what each hop of a falling
 *   ball needs, since an eased path would settle gently instead of arriving.
 * * ``lift_carry_drop`` — arcs up and over, then falls. Takes ``clearance``.
 * * ``arc`` — a curve bowed to one side. ``bend`` is how far, as a fraction of
 *   the distance travelled (default 0.3); negative bows the other way.
 *
 * Timing and feel, for everything about the shape — where it is, its size,
 * colour, opacity, turn, and a marker's reveal — not only its position:
 *
 * * ``ease`` — ``linear``, ``in``, ``out``, ``in_out``, ``back`` (overshoots and
 *   settles), ``spring`` (takes ``bounciness`` 0..1) or ``bounce``. Unset, a
 *   shape moves as it always has: eased position, the rest linear.
 * * ``delay`` — wait this fraction of the beat before starting.
 * * ``stagger`` — matching shapes start this fraction of the beat apart, in
 *   ``order``: ``screen`` (left to right, then top to bottom; the default),
 *   ``reverse``, ``name`` or ``centre`` (from the middle out). They all take
 *   the same time, and the last one finishes as the beat ends.
 */
export class Rule {
  pattern: any;
  position: any;
  ease: any;
  order: any;
  options: any;
  _fields: any;
  private made: unknown[] = [];

  constructor(pattern: Key, position?: string, options?: { ease?: string; order?: string } & Record<string, unknown>) {
    const { ease, order, ...rest } = options ?? {};
    this._set(call("rule", { pattern: to.key(pattern), position: to.string((position ?? "straight")), ease: to.string(ease), order: to.string((order ?? "screen")), options: rest }));
  }

  /** @internal One the Authoring Core has already made. */
  static _of(made: any): Rule {
    const value = Object.create(Rule.prototype);
    value._set(made);
    return value;
  }

  private _set(made: any): void {
    const { payload, ...fields } = made;
    this._fields = fields;
    this.made = made.payload;
    this.pattern = made.pattern;
    this.position = made.position;
    this.ease = made.ease;
    this.order = made.order;
    this.options = made.options;
  }

  /** @internal What the Engine takes. */
  _payload(): unknown[] {
    return this.made;
  }

}

/**
 * Text appearance and travel distance shared by a preset's units.
 *
 * ``size`` is in canvas points. ``font`` and ``fill`` are optional, and
 * ``distance`` is how far each word or letter starts from its final place.
 */
export class TextPresetStyle {
  size: any;
  font: any;
  fill: any;
  distance: any;
  _fields: any;
  private made: unknown[] = [];

  constructor(options?: { size?: number; font?: string; fill?: string; distance?: number }) {
    const { size, font, fill, distance } = options ?? {};
    this._set(call("text_preset_style", { size: to.number((size ?? 16.0)), font: to.string(font), fill: to.string(fill), distance: to.number((distance ?? 48.0)) }));
  }

  /** @internal One the Authoring Core has already made. */
  static _of(made: any): TextPresetStyle {
    const value = Object.create(TextPresetStyle.prototype);
    value._set(made);
    return value;
  }

  private _set(made: any): void {
    const { payload, ...fields } = made;
    this._fields = fields;
    this.size = made.size;
    this.font = made.font;
    this.fill = made.fill;
    this.distance = made.distance;
  }

}

/**
 * Draw text as individually moving words or letters.
 *
 * Use a named helper such as ``codimate.text.words_up`` for a fixed motion.
 * This class is the configurable form when the preset name is data::
 *
 *     title = cm.TextPreset(
 *         "title", "Hello world", preset, style=cm.TextPresetStyle(size=72)
 *     )
 *
 *     def view(frame):
 *         scene = cm.Scene()
 *         title.draw(scene, state="shown" if frame.state.ready else "hidden")
 *         return scene
 *
 *     cm.explain(..., motion=title.rules(stagger=0.12))
 *
 * ``preset`` is one of ``words_up``, ``words_down``, ``letters_up``,
 * ``letters_down``, ``words_left``, ``words_right``, or ``words_up_blur``.
 *
 * ``state`` is ``"hidden"`` or ``"shown"``. For ``words_up_blur``, it may
 * also be ``"blurred"``; switch from that state to ``"shown"`` to resolve the
 * soft halo into sharp text. The halo uses small translucent text copies, so
 * it needs no image or optional imaging dependency.
 *
 * ``at`` is the text block's centre (a point or Slot); line breaks are
 * centred independently. ``letters_*`` keeps combining marks and joined
 * emoji together as one moving unit.
 */
export class TextPreset {
  name: any;
  content: any;
  preset: any;
  style: any;
  at: any;
  _fields: any;
  private made: unknown[] = [];

  constructor(name: string, content: unknown, preset: string, options?: { style?: TextPresetStyle; at?: At }) {
    const { style, at } = options ?? {};
    this._set(call("text_preset", { name: to.string(name), content: to.text(content), preset: to.string(preset), style: to.style(style), at: to.at(at) }));
  }

  /** @internal One the Authoring Core has already made. */
  static _of(made: any): TextPreset {
    const value = Object.create(TextPreset.prototype);
    value._set(made);
    return value;
  }

  private _set(made: any): void {
    const { payload, ...fields } = made;
    this._fields = fields;
    this.name = made.name;
    this.content = made.content;
    this.preset = made.preset;
    this.style = made.style == null ? null : TextPresetStyle._of(made.style);
    this.at = made.at;
  }

  /** Draw this preset into a Scene or Group at one of its stable states. */
  draw(group: Group, options?: { state?: string }): Handle[] {
    const { state } = options ?? {};
    return group._adopt(call("text_preset_draw", { preset: this._fields, film_font: filmFont(), group: to.group(group), state: to.string((state ?? "hidden")) }));
  }

  /** Return motion rules for this preset's words, letters, and blur. */
  rules(options?: { ease?: string; stagger?: number; order?: string }): Rule[] {
    const { ease, stagger, order } = options ?? {};
    return call("text_preset_rules", { preset: this._fields, ease: to.string((ease ?? "out")), stagger: to.number((stagger ?? 0.12)), order: to.string((order ?? "screen")) }).map((made: any) => Rule._of(made));
  }
}

/** Create a preset that brings words up into place. */
export function wordsUp(name: string, content: unknown, options?: { style?: TextPresetStyle; at?: At }): TextPreset {
  return new TextPreset(name, content, "words_up", options);
}

/** Create a preset that brings words down into place. */
export function wordsDown(name: string, content: unknown, options?: { style?: TextPresetStyle; at?: At }): TextPreset {
  return new TextPreset(name, content, "words_down", options);
}

/** Create a preset that brings letter clusters up into place. */
export function lettersUp(name: string, content: unknown, options?: { style?: TextPresetStyle; at?: At }): TextPreset {
  return new TextPreset(name, content, "letters_up", options);
}

/** Create a preset that brings letter clusters down into place. */
export function lettersDown(name: string, content: unknown, options?: { style?: TextPresetStyle; at?: At }): TextPreset {
  return new TextPreset(name, content, "letters_down", options);
}

/** Create a preset that brings words in from the left. */
export function wordsLeft(name: string, content: unknown, options?: { style?: TextPresetStyle; at?: At }): TextPreset {
  return new TextPreset(name, content, "words_left", options);
}

/** Create a preset that brings words in from the right. */
export function wordsRight(name: string, content: unknown, options?: { style?: TextPresetStyle; at?: At }): TextPreset {
  return new TextPreset(name, content, "words_right", options);
}

/** Create a preset that brings soft, blurred words up into focus. */
export function wordsUpBlur(name: string, content: unknown, options?: { style?: TextPresetStyle; at?: At }): TextPreset {
  return new TextPreset(name, content, "words_up_blur", options);
}

/**
 * A map from your numbers to pixels. Built by :func:`axes`.
 *
 * It draws nothing you did not ask it to and owns none of your names — see
 * :meth:`at`. Its own shapes (frame, ticks, labels) are drawn by
 * :meth:`draw`, under names you choose the prefix of.
 */
export class Axes {
  x0: any;
  x1: any;
  y0: any;
  y1: any;
  left: any;
  top: any;
  w: any;
  h: any;
  ink: any;
  label: any;
  layer: any;
  right: any;
  bottom: any;
  _fields: any;
  private made: unknown[] = [];

  constructor(options?: { x?: readonly [number, number]; y?: readonly [number, number]; at?: At; size?: number | Point; within?: Slot }) {
    const { x, y, at, size, within } = options ?? {};
    this._set(call("axes", { canvas: [width(), height()], x: to.pair(x), y: to.pair(y), at: to.at(at), size: to.radius(size), within: to.slot(within) }));
  }

  /** @internal One the Authoring Core has already made. */
  static _of(made: any): Axes {
    const value = Object.create(Axes.prototype);
    value._set(made);
    return value;
  }

  private _set(made: any): void {
    const { payload, ...fields } = made;
    this._fields = fields;
    this.x0 = made.x0;
    this.x1 = made.x1;
    this.y0 = made.y0;
    this.y1 = made.y1;
    this.left = made.left;
    this.top = made.top;
    this.w = made.w;
    this.h = made.h;
    this.ink = made.ink;
    this.label = made.label;
    this.layer = made.layer;
    this.right = made.right;
    this.bottom = made.bottom;
  }

  /**
   * Restyle the frame, ticks and labels. Returns a new Axes.
   *
   *     plot = cm.axes(x=(0, 10), y=(0, 5)).looks(ink="#334", label=16)
   *
   * A separate call rather than more arguments on :meth:`draw`, which the
   * shapes made the same choice about: no call in this library takes more
   * than five things.
   */
  looks(ink?: string, label?: number, layer?: number): Axes {
    return Axes._of(call("axes_looks", { axes: this._fields, ink: to.string(ink), label: to.number(label), layer: to.number(layer) }));
  }

  /**
   * One data point as a pixel pair, for any shape's ``at=``.
   *
   *     scene.circle("dot", r=8, at=plot.at(2.0, 4.0))
   *
   * Handing back pixels rather than drawing is the whole design: what you
   * do with them is an ordinary shape with a name of yours, so it tweens,
   * `focus` frames it, and a motion Rule can be aimed at it.
   */
  at(x: number, y: number): [number, number] {
    return call("axes_at", { axes: this._fields, x: to.number(x), y: to.number(y) });
  }

  /**
   * ``steps + 1`` points along ``y = f(x)``, as pixels.
   *
   *     scene.curve("f", plot.line(lambda x: x * x), w=4).fill("red")
   *
   * ``over=(lo, hi)`` samples part of the range instead of all of it.
   *
   * An open `curve` is *drawn* rather than filled, so its colour is
   * ``fill()`` and its thickness is ``w=`` — ``fill("none", edge=...)``
   * draws nothing at all.
   *
   * Every sample is returned, including any that fall outside the box.
   * Dropping them would be prettier and is wrong: two curves with different
   * point counts do not interpolate (ADR 0010), so a curve that shed a
   * point as it left the frame would stop animating. Keep the count and
   * clamp the range with ``over=`` if you need it inside.
   */
  line(f: (x: number) => number, steps?: number, over?: readonly [number, number]): [number, number][] {
    const xs: number[] = call("axes_samples", { axes: this._fields, steps: to.int((steps ?? 200)), over: to.pair(over) });
    return call("axes_points", { axes: this._fields, xs, ys: xs.map((x) => Number(f(x))) });
  }

  /**
   * ``(n, value, label)`` per tick, where ``n`` is the step multiple.
   *
   * ``n`` is what a tick is named after, not the value. A float that drifts
   * by one part in a billion is a different name, and a renamed shape
   * leaves and re-enters — which is a fade, on a picture that did not move.
   * An integer count of steps cannot drift.
   */
  ticks(axis?: string, about?: number): [number, number, string][] {
    return call("axes_ticks", { axes: this._fields, axis: to.string((axis ?? "x")), about: to.int((about ?? 6)) });
  }

  /**
   * Draw the frame, ticks and labels. Returns the Axes, for chaining.
   *
   *     plot = cm.axes(x=(-4, 4), y=(-2, 6)).draw(scene)
   *
   * Every shape is named ``(name, ...)``, so two plots on one canvas do not
   * collide and you can restyle or omit any of them by drawing your own.
   * Colours and sizes come from :meth:`looks`.
   */
  draw(group: Group, name?: Key, options?: { about?: number; grid?: boolean }): this {
    const { about, grid } = options ?? {};
    group._adopt(call("axes_draw", { axes: this._fields, film_font: filmFont(), group: to.group(group), name: to.key((name ?? "plot")), about: to.int((about ?? 6)), grid: to.bool((grid ?? false)) }));
    return this;
  }
}

/**
 * A coordinate map from your numbers to pixels.
 *
 *     plot = cm.axes(x=(-4, 4), y=(-2, 6), size=(760, 420)).draw(scene)
 *     scene.curve("f", plot.line(lambda t: t * t), w=4).fill("orange")
 *     scene.circle("dot", r=8, at=plot.at(t, t * t))
 *
 * ``x`` and ``y`` are ``(low, high)``. ``size`` is ``(w, h)`` in pixels, or
 * one number for a square; it defaults to most of the canvas. ``at`` places
 * the box's centre, and ``within=slot`` fits it to part of the canvas.
 *
 * It returns points rather than drawing, so what you plot is a shape with a
 * name of yours — which is what lets it tween, be framed by `focus`, and be
 * aimed at by a motion Rule. See ADR 0016.
 */
export function axes(options?: { x?: readonly [number, number]; y?: readonly [number, number]; at?: At; size?: number | Point; within?: Slot }): Axes {
  return new Axes(options);
}
