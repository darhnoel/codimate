// Generated from api/surface.toml by tools/generate.py. Do not edit.
//
// The JavaScript Skin's side of the Interface Manifest: each method sends
// its arguments across the Waist and keeps what comes back.

import { filmFont, to, type At, type Frame, type Key, type Point, type ShapeData, type Slot } from "./waist.ts";
import { call } from "./waist.ts";
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
