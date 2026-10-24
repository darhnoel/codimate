// codimate for JavaScript and TypeScript: the same Authoring Surface as the
// Python package, over the same Authoring Core (ADR 0030–0032).
//
//   import * as cm from "codimate";
//
//   const view = (frame) => {
//     const scene = new cm.Scene();
//     scene.circle("ball", { r: 40, at: [frame.state.x, 360] }).fill("#2997ff");
//     return scene;
//   };
//   cm.explain({ trace: cm.trace(walk, { x: 200 }), view }).play(canvas);

export { at, canvas, font, height, width } from "./waist.ts";
export { Group, Handle, Scene } from "./scene.ts";
export { Frame, Item, items, trace, type Emit, type Event, type Trace } from "./trace.ts";
export { Explanation, Timing, explain, type View } from "./explain.ts";
