// Saying more about a shape: turned about different pivots, grown, rounded,
// layered and faded, each changing between two moments.

import * as cm from "../../skins/js/src/index.ts";

const walk = (state: { t: number }, emit: cm.Emit) => {
  state.t = 1;
  emit("change");
};

const view = (frame: cm.Frame<{ t: number }>) => {
  const t = frame.state.t;
  const scene = new cm.Scene();
  scene.rect("door", { w: 80, h: 160, at: [250, 360] }).fill("#8e5a2b").turn(70 * t, "left");
  scene.rect("card", { w: 180, h: 110, at: [560, 360] }).fill("#2997ff").round(4 + 20 * t).turn(-12 * t);
  scene.circle("pulse", { r: 40, at: [860, 360] }).fill("#ff453a").grow(1 + t);
  scene.rect("wide", { w: 60, h: 60, at: [1100, 250] }).fill("#30d158").grow([1 + 2 * t, 1]);
  scene.rect("under", { w: 200, h: 120, at: [1100, 470] }).fill("#3a3a3c").on(2);
  scene.text("over", "on top", { size: 30, at: [1100, 470] }).on(5, 0.4 + 0.6 * t);
  return scene;
};

await cm.explain({ trace: cm.trace(walk, { t: 0 }), view }).exportWeb(process.argv[2]);
