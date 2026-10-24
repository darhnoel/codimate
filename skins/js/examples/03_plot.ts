// 03 · A plot that explains itself: axes, a curve from your own function, a
// dot travelling along it, a highlighted word and a title that rises in.
//
//   node skins/js/examples/03_plot.ts              → results/js/plot.mp4
//   node skins/js/examples/03_plot.ts plot.json    → the film's data, no video
//
// Shows: cm.axes (draw, line, at), a text preset with its rules, a highlight
// that rubs out, and a camera that moves in to frame the dot.

import * as cm from "../src/index.ts";

const f = (x: number) => (x * x) / 2;
const plot = cm.axes({ x: [-4, 4], y: [-1, 8], size: [780, 430], at: [640, 400] })
  .looks("#5b6478", 16);
const title = cm.wordsUp("title", "Watch the slope", {
  style: new cm.TextPresetStyle({ size: 44 }),
  at: [640, 90],
});

type State = { t: number; step: number };

const walk = (state: State, emit: cm.Emit) => {
  state.step = 1;
  emit("title");
  for (const t of [-1, 1, 3]) {
    state.t = t;
    emit("move");
  }
  state.step = 2;
  emit("zoom");
};

const view = (frame: cm.Frame<State>) => {
  const { t, step } = frame.state;
  const scene = new cm.Scene();
  title.draw(scene, { state: step >= 1 ? "shown" : "hidden" });
  plot.draw(scene, undefined, { grid: true });
  scene.curve("f", plot.line(f, 80), { w: 4 }).fill("#ff9f0a");
  scene.circle("dot", { r: 10, at: plot.at(t, f(t)) }).fill("white");
  const [band] = scene.highlight("note", "y = x² / 2", { size: 26, at: [1030, 640] });
  band.write(step >= 2 ? 0 : 1);
  if (step >= 2) scene.focus(["dot"], { pad: 140 });
  return scene;
};

const film = cm.explain({
  trace: cm.trace(walk, { t: -3, step: 0 }),
  view,
  motion: [...title.rules(), new cm.Rule("dot", "arc", { bend: 0.25, ease: "in_out" })],
  timing: new cm.Timing({ default: 0.9, events: { zoom: 1.4 } }),
});
const out = process.argv[2] ?? "results/js/plot.mp4";
await (out.endsWith(".json") ? film.exportWeb(out) : film.render(out, { fps: 30 }));
console.log(`wrote ${out} (${film.duration.toFixed(1)} s)`);
console.log(film.report() || "no labels covered");
