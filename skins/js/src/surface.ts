// Generated from api/surface.toml by tools/generate.py. Do not edit.
//
// The JavaScript Skin's side of the Interface Manifest: each method sends
// its arguments across the Waist and keeps what comes back.

import { filmFont, to, type At, type Frame, type Key, type Point, type ShapeData, type Slot } from "./waist.ts";
import { call, height, paired, width } from "./waist.ts";
import type { Group, Handle } from "./scene.ts";

export abstract class GroupSurface {
  protected abstract _frame(): Frame;
  protected abstract _build(op: string, data: object): Handle;
  protected abstract _nest(frame: Frame): Group;

  /** A rectangle. Say where with a point, a Slot, or `at(...)`. Round the corners with `.round(12)`. */
  rect(key: Key, options: { h: number; w?: number; at?: At }): Handle {
    return this._build("rect", { group: this._frame(), key: to.key(key), h: to.number(options?.h), w: to.number(options?.w), at: to.at(options?.at) });
  }

  /** A filled circle. `fill("none", edge=...)` draws a ring. */
  circle(key: Key, options: { r: number; at?: At }): Handle {
    return this._build("circle", { group: this._frame(), key: to.key(key), r: to.number(options?.r), at: to.at(options?.at) });
  }

  /** Text, centred horizontally. `font` is a font file or an installed font's name, for this text only; unset, the film's font. */
  text(key: Key, content: unknown, options?: { size?: number; font?: string; at?: At }): Handle {
    return this._build("text", { group: this._frame(), film_font: filmFont(), key: to.key(key), content: to.text(content), size: to.number((options?.size ?? 16)), font: to.string(options?.font), at: to.at(options?.at) });
  }

  /** A line between two points. `w` is its thickness. Each end may be a Slot (a line joins the middles of two places) or a point. */
  line(key: Key, options: { start: Point; end: Point; w?: number }): Handle {
    return this._build("line", { group: this._frame(), key: to.key(key), start: to.point(options?.start), end: to.point(options?.end), w: to.number((options?.w ?? 2.0)) });
  }

  /** An arrow, as a single filled shape: one name, travelling as one thing. `w` is the shaft, `head` the point. */
  arrow(key: Key, options: { start: Point; end: Point; w?: number; head?: number }): Handle {
    return this._build("arrow", { group: this._frame(), key: to.key(key), start: to.point(options?.start), end: to.point(options?.end), w: to.number((options?.w ?? 4.0)), head: to.number((options?.head ?? 16.0)) });
  }

  /** A shape with corners. `points` is a sequence of points. Closed and filled by default; `closed=False` leaves an open outline. Two polygons tween only if they have the same number of corners. */
  polygon(key: Key, points: readonly Point[], options?: { closed?: boolean }): Handle {
    return this._build("polygon", { group: this._frame(), key: to.key(key), points: to.points(points), closed: to.bool((options?.closed ?? true)) });
  }

  /** A smooth line through every one of `points`: samples, not control points. `w` is the stroke width. Open by default; `closed=True` makes a fillable shape. Two curves tween only if they have the same number of points. */
  curve(key: Key, points: readonly Point[], options?: { w?: number; closed?: boolean }): Handle {
    return this._build("curve", { group: this._frame(), key: to.key(key), points: to.points(points), w: to.number((options?.w ?? 2.0)), closed: to.bool((options?.closed ?? false)) });
  }

  /** A slice of a circle: an angle mark, a pie, a dial, an orbit. `sweep` is (start, end) in degrees, clockwise from twelve o'clock; `r` is the radius, or (rx, ry) for an ellipse. Open by default; `.round(1)` closes it into a pie slice. */
  arc(key: Key, options: { r: number | Point; sweep: readonly [number, number]; at?: At }): Handle {
    return this._build("arc", { group: this._frame(), key: to.key(key), r: to.radius(options?.r), sweep: to.pair(options?.sweep), at: to.at(options?.at) });
  }

  /** A place to draw a thing made of several shapes. Give it a Slot and it sits where the Slot says (at `anchor`: center, top or bottom), or place it with `at` like any shape. Inside it, 0 is that point, and everything drawn on it moves as one. */
  group(key: Key, slot?: Slot, options?: { at?: At; anchor?: string; w?: number }): Group {
    return this._nest(call("group", { group: this._frame(), key: to.key(key), slot: to.slot(slot), at: to.at(options?.at), anchor: to.string(options?.anchor), w: to.number(options?.w) }));
  }
}

export abstract class HandleSurface {
  protected abstract _shape(): ShapeData;
  protected abstract _change(op: string, data: object): void;

  /** Colour it. `edge` outlines it; `color="none"` leaves it unfilled. Saying nothing about the outline leaves it alone. */
  fill(color?: string, options?: { edge?: string; edgeW?: number }): this {
    this._change("fill", { shape: this._shape(), color: to.string((color ?? "white")), edge: to.string(options?.edge), edge_w: to.number(options?.edgeW) });
    return this;
  }
}

/** The corners of a regular polygon, for `polygon`. A triangle is three sides, a hexagon six; `turn` rotates it in degrees, the first corner otherwise pointing straight up. */
export function ngon(sides: number, r: number, options?: { at?: Point; turn?: number }): [number, number][] {
  return call("ngon", { sides: to.int(sides), r: to.number(r), at: to.point((options?.at ?? [0, 0])), turn: to.number((options?.turn ?? 0.0)) });
}

/** The corners of a star, for `polygon`. `inner` is the radius of the valleys; it defaults to a proportion that looks like a star rather than a gear. */
export function star(points: number, r: number, options?: { inner?: number; at?: Point; turn?: number }): [number, number][] {
  return call("star", { points: to.int(points), r: to.number(r), inner: to.number(options?.inner), at: to.point((options?.at ?? [0, 0])), turn: to.number((options?.turn ?? 0.0)) });
}

/** One Slot per item, evenly spaced on a shared baseline, anchored at bottom-centre. `size` is the Slot's width (its height follows) or (w, h). `within` divides part of the canvas instead of all of it. Gives (slot, item) pairs, or bare Slots for a count. */
export function row(items: number | Iterable<unknown>, options?: { gap?: number; size?: number | Point; at?: At; within?: Slot }): any[] {
  return paired(items, call("row", { canvas: [width(), height()], items: to.items(items), gap: to.number((options?.gap ?? 40.0)), size: to.radius(options?.size), at: to.at(options?.at), within: to.slot(options?.within) }));
}

/** One Slot per item, stacked vertically around a centre line, anchored at their centres. `x` places the column; it defaults to mid-canvas. Gives (slot, item) pairs, or bare Slots for a count. */
export function column(items: number | Iterable<unknown>, options?: { gap?: number; size?: number | Point; at?: At; within?: Slot }): any[] {
  return paired(items, call("column", { canvas: [width(), height()], items: to.items(items), gap: to.number((options?.gap ?? 40.0)), size: to.radius(options?.size), at: to.at(options?.at), within: to.slot(options?.within) }));
}

/** How wide and tall `text` will be at `size`: (w, h). Measured by the Engine with the real fonts, fallback included, so it is right for Khmer and anything else that is not plain ASCII. The height is the line height, the same for every word. */
export function measure(text: unknown, size?: number, font?: string): [number, number] {
  return call("measure", { film_font: filmFont(), text: to.text(text), size: to.number((size ?? 16.0)), font: to.string(font) });
}

/** The easing curve the Engine applies between two moments: the same curve your animation uses, not a copy of it. */
export function ease(t: number): number {
  return call("ease", { t: to.number(t) });
}
