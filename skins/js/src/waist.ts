// The JavaScript Skin's crossing of the Waist (ADR 0032): send one operation
// and its data to the Authoring Core (Rust, as WebAssembly), get the answer
// back. Also the converters the generated methods (surface.ts) use, so every
// language sends the same shapes of data.

import init, * as engine from "../pkg/codimate_wasm.js";

// Loaded once, before anything else: from disk in Node, over the network in a
// browser. Top-level await, so importing codimate is all an author does.
const inNode = typeof process !== "undefined" && !!process.versions?.node;
const wasmFile = new URL("../pkg/codimate_wasm_bg.wasm", import.meta.url);
const exports = inNode
  ? engine.initSync({ module: (await import("node:fs")).readFileSync(wasmFile) })
  : await init({ module_or_path: wasmFile });

export { engine };

/** The engine's memory, where a drawn frame's pixels are read in place. */
export const memory = (): WebAssembly.Memory => exports.memory;

/** Run `op` in the Authoring Core. A refusal throws here, on the author's line. */
export function call(op: string, data: object): any {
  const answer = JSON.parse(engine.call(op, JSON.stringify(data)));
  if ("error" in answer) throw new Error(answer.error);
  return answer.ok;
}

/** A name, or a path of names: `["edge", [0, 1]]` is the shape `edge/0/1`. */
export type Key = string | number | readonly Key[];
/** A point `[x, y]`, a Slot (its centre), or a Place from `at(...)`. */
export type At = readonly [number, number] | { x: number; y: number; w: number; h: number } | Place;
/** A point `[x, y]`, or a Slot (its centre). */
export type Point = readonly [number, number] | { x: number; y: number; w: number; h: number };
/** A place to put something: its centre, its size, and the edge things placed
 * there line up on. Not a shape: nothing draws a Slot. `row` and `column` hand
 * these out. */
export class Slot {
  readonly x: number;
  readonly y: number;
  readonly w: number;
  readonly h: number;
  readonly anchor: string;

  constructor(slot: { x: number; y: number; w: number; h: number; anchor?: string }) {
    this.x = slot.x;
    this.y = slot.y;
    this.w = slot.w;
    this.h = slot.h;
    this.anchor = slot.anchor ?? "center";
  }

  get left() { return this.x - this.w / 2; }
  get right() { return this.x + this.w / 2; }
  get top() { return this.y - this.h / 2; }
  get bottom() { return this.y + this.h / 2; }

  /** The single point this Slot anchors things at. */
  point(anchor?: string): [number, number] {
    const a = anchor ?? this.anchor;
    if (a === "center") return [this.x, this.y];
    if (a === "bottom") return [this.x, this.bottom];
    if (a === "top") return [this.x, this.top];
    throw new Error(`unknown anchor '${a}' — use center, top or bottom`);
  }
}

/** The Core's Slots as Slots, each with its item, or bare for a count. */
export function paired(items: number | Iterable<unknown>, slots: any[]): any[] {
  const made = slots.map((s) => new Slot(s));
  if (typeof items === "number") return made;
  const list = [...items];
  return made.map((slot, i) => [slot, list[i]]);
}
/** A placement by edges: one horizontal anchor and one vertical. */
export interface Place { x?: number; y?: number; top?: number; bottom?: number; readonly place: true }
/** A Group as the Core needs it. */
export interface Frame { path: string[]; ox: number; oy: number; w: number }
/** One shape as the Core describes it: flat, every field present. */
export type ShapeData = Record<string, unknown> & { item: string; kind: string };

export const to = {
  key(key: Key): string[] {
    const parts: string[] = [];
    const walk = (part: Key) => (Array.isArray(part) ? part.forEach(walk) : parts.push(String(part)));
    walk(key);
    return parts;
  },
  number: (value?: number) => (value === undefined || value === null ? null : Number(value)),
  text: (value: unknown) => String(value),
  string: (value?: string) => (value === undefined || value === null ? null : String(value)),
  point(point: Point): [number, number] {
    return Array.isArray(point) ? [Number(point[0]), Number(point[1])] : [(point as any).x, (point as any).y];
  },
  points: (points: readonly Point[]) => points.map((p) => to.point(p)),
  radius: (r?: number | Point) => (r === undefined || r === null ? null : typeof r === "number" ? r : to.point(r)),
  pair: (pair: readonly [number, number]) => [Number(pair[0]), Number(pair[1])],
  bool: (value: boolean) => Boolean(value),
  int: (value: number) => Math.trunc(Number(value)),
  style: (style?: { _fields: unknown } | null) => (style === undefined || style === null ? null : style._fields),
  group: (group: any) => group._frame(),
  items: (items: number | Iterable<unknown>) => (typeof items === "number" ? items : [...items].length),
  slot: (slot?: Slot | { x: number; y: number; w: number; h: number; anchor?: string }) =>
    slot === undefined || slot === null ? null : { x: slot.x, y: slot.y, w: slot.w, h: slot.h, anchor: slot.anchor ?? "center" },
  at(at?: At) {
    if (at === undefined || at === null) return null;
    if (Array.isArray(at)) return [Number(at[0]), Number(at[1])];
    if ((at as Place).place) {
      const p = at as Place;
      return { x: p.x ?? null, y: p.y ?? null, top: p.top ?? null, bottom: p.bottom ?? null };
    }
    const slot = at as { x: number; y: number };
    return [slot.x, slot.y];
  },
};

// -- the film's settings, as Python's layout/canvas.py keeps them --------------

const settings = { width: 1280, height: 720, font: "" };

/** Set the size of the video. Defaults to 1280×720. */
export function canvas(w: number, h: number): void {
  settings.width = Number(w);
  settings.height = Number(h);
}

/** The canvas width. `width() / 2` is the horizontal centre. */
export const width = () => settings.width;
/** The canvas height. */
export const height = () => settings.height;

/** Set the font every text in the film uses, unless it names its own. */
export function font(name = ""): void {
  settings.font = String(name);
}

export const filmFont = () => settings.font;

/** A place for a shape's `at`: one horizontal anchor and one vertical. */
export function at(place: { x?: number; y?: number; top?: number; bottom?: number }): Place {
  if (place.y !== undefined && (place.top !== undefined || place.bottom !== undefined)) {
    throw new Error("give one of y=, top= or bottom=");
  }
  return { ...place, place: true };
}
