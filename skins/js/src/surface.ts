// Generated from api/surface.toml by tools/generate.py. Do not edit.
//
// The JavaScript Skin's side of the Interface Manifest: each method sends
// its arguments across the Waist and keeps what comes back.

import { filmFont, to, type At, type Frame, type Key, type ShapeData } from "./waist.ts";
import type { Handle } from "./scene.ts";

export abstract class GroupSurface {
  protected abstract _frame(): Frame;
  protected abstract _build(op: string, data: object): Handle;

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
