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

  /** Real mathematics, written as LaTeX. `size` means what it means for text. Draw it on with `write(reveal=, pen=)`. Typeset once when the film is built. */
  formula(key: Key, latex: string, options?: { size?: number; at?: At }): Handle {
    return this._build("formula", { group: this._frame(), key: to.key(key), latex: to.string(latex), size: to.number((options?.size ?? 16.0)), at: to.at(options?.at) });
  }

  /** A picture (PNG or JPEG) drawn into the frame. `size` is a box it fits inside, one number or (w, h), aspect always kept; leave it out and it is drawn at its own pixel size. */
  image(key: Key, file: unknown, options?: { size?: number | Point; at?: At }): Handle {
    return this._build("image", { group: this._frame(), key: to.key(key), file: to.text(file), size: to.radius(options?.size), at: to.at(options?.at) });
  }

  /** Vector art from a file, drawn as real geometry: it tweens, turns, grows and draws on like anything else. `size` is a box it fits inside, one number or (w, h). It keeps its own colours unless `fill` overrides them. */
  svg(key: Key, file: unknown, options?: { size?: number | Point; at?: At }): Handle {
    return this._build("svg", { group: this._frame(), key: to.key(key), file: to.text(file), size: to.radius((options?.size ?? 120.0)), at: to.at(options?.at) });
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

  /** Rotate it. `pivot` is center, top, bottom, left or right. Text moves but does not turn. */
  turn(degrees: number, pivot?: string): this {
    this._change("turn", { shape: this._shape(), degrees: to.number(degrees), pivot: to.string((pivot ?? "center")) });
    return this;
  }

  /** Scale it: a number for both axes, or (sx, sy) to stretch. */
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

  /** How much of a formula or an imported SVG shows (`reveal`, 0 to 1), and whether a pen draws it on (`pen`, its width). On a marker, how far its stroke has been drawn on. */
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

    /** Look at these things; the camera works out where to stand. No names frames the whole canvas again. `pad` is room around them; `least` is the smallest thing the camera fills the frame with. Anything on an overlay stays where it is put. */
    focus(names: readonly Key[], options?: { pad?: number; least?: number }): void {
      this._aim(call("focus", { names: to.keys(names), pad: to.number((options?.pad ?? 40.0)), least: to.number((options?.least ?? 240.0)) }));
    }

    /** A place for things the camera does not move: titles and narration. The diagram pans and zooms underneath; whatever is drawn here stays put, in plain screen coordinates. */
    overlay(): Group {
      return this._openOverlay(call("overlay", { group: this._frame() }));
    }
  }
  return SceneMethods;
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

/** How things matching `pattern` change; the first matching rule wins. `position` is the path: straight, linear, fall, lift_carry_drop or arc. `ease` is linear, in, out, in_out, back, spring or bounce. `delay` and `stagger` are fractions of the beat; `order` is screen, reverse, name or centre. */
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

/** Text appearance and travel distance shared by a preset's words or letters. `size` is in canvas points; `font` and `fill` are optional; `distance` is how far each starts from its place. */
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

/** Text drawn as individually moving words or letters. `preset` is words_up, words_down, letters_up, letters_down, words_left, words_right or words_up_blur. Draw it in a state (hidden or shown; blurred too for words_up_blur) and pass its rules() to explain. */
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

  /** The motion rules for this preset's words or letters, and its blur. */
  rules(options?: { ease?: string; stagger?: number; order?: string }): Rule[] {
    const { ease, stagger, order } = options ?? {};
    return call("text_preset_rules", { preset: this._fields, ease: to.string((ease ?? "out")), stagger: to.number((stagger ?? 0.12)), order: to.string((order ?? "screen")) }).map((made: any) => Rule._of(made));
  }
}

/** A text preset that brings words up into place. */
export function wordsUp(name: string, content: unknown, options?: { style?: TextPresetStyle; at?: At }): TextPreset {
  return new TextPreset(name, content, "words_up", options);
}

/** A text preset that brings words down into place. */
export function wordsDown(name: string, content: unknown, options?: { style?: TextPresetStyle; at?: At }): TextPreset {
  return new TextPreset(name, content, "words_down", options);
}

/** A text preset that brings letter clusters up into place. */
export function lettersUp(name: string, content: unknown, options?: { style?: TextPresetStyle; at?: At }): TextPreset {
  return new TextPreset(name, content, "letters_up", options);
}

/** A text preset that brings letter clusters down into place. */
export function lettersDown(name: string, content: unknown, options?: { style?: TextPresetStyle; at?: At }): TextPreset {
  return new TextPreset(name, content, "letters_down", options);
}

/** A text preset that brings words in from the left. */
export function wordsLeft(name: string, content: unknown, options?: { style?: TextPresetStyle; at?: At }): TextPreset {
  return new TextPreset(name, content, "words_left", options);
}

/** A text preset that brings words in from the right. */
export function wordsRight(name: string, content: unknown, options?: { style?: TextPresetStyle; at?: At }): TextPreset {
  return new TextPreset(name, content, "words_right", options);
}

/** A text preset that brings soft, blurred words up into focus. */
export function wordsUpBlur(name: string, content: unknown, options?: { style?: TextPresetStyle; at?: At }): TextPreset {
  return new TextPreset(name, content, "words_up_blur", options);
}
