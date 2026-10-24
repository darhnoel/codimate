// Axes: a plot with a grid and its own ink, a curve from the author's own
// function, and a dot that moves along it, mapped by `at`.

import * as cm from "../../skins/js/src/index.ts";

const plot = cm.axes({ x: [-4, 4], y: [-2, 6], size: [760, 420] }).looks("#5b6478", 16);

const walk = (state: { t: number }, emit: cm.Emit) => {
  for (const t of [-1.0, 2.0]) {
    state.t = t;
    emit("move");
  }
};

const view = (frame: cm.Frame<{ t: number }>) => {
  const t = frame.state.t;
  const scene = new cm.Scene();
  plot.draw(scene, undefined, { grid: true });
  scene.curve("f", plot.line((x) => (x * x) / 2, 80), { w: 4 }).fill("#ff9f0a");
  scene.circle("dot", { r: 9, at: plot.at(t, (t * t) / 2) }).fill("white");
  return scene;
};

await cm.explain({ trace: cm.trace(walk, { t: -3.0 }), view }).exportWeb(process.argv[2]);
