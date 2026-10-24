// Markers and highlights: a stroke drawn on and turned, a pink one, and a
// highlighted word whose band rubs out while its words lighten in step.

import * as cm from "../../skins/js/src/index.ts";

const walk = (state: { reveal: number }, emit: cm.Emit) => {
  for (const reveal of [0.5, 0.0]) {
    state.reveal = reveal;
    emit("rub");
  }
};

const view = (frame: cm.Frame<{ reveal: number }>) => {
  const scene = new cm.Scene();
  const reveal = frame.state.reveal;
  scene.marker("note", { w: 260, h: 70, at: [320, 200] }).turn(-8).write(reveal);
  scene.marker("aside", { w: 180, h: 50, at: [900, 200] }).fill("pink");
  const [band, words] = scene.highlight("pivot", "Pivot", { size: 40, at: [640, 450] });
  words.fill("#ffd60a");
  band.write(reveal);
  return scene;
};

await cm.explain({ trace: cm.trace(walk, { reveal: 1.0 }), view }).exportWeb(process.argv[2]);
