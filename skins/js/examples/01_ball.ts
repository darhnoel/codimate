// 01 · A ball and its label: the smallest film.
//
//   node skins/js/examples/01_ball.ts                 → results/js/ball.mp4
//   node skins/js/examples/01_ball.ts ball.json       → the film's data, no video

import * as cm from "../src/index.ts";

type Ball = { x: number };

// The algorithm: change the state, then say what happened.
const walk = (ball: Ball, emit: cm.Emit) => {
  for (const x of [640, 1080]) {
    ball.x = x;
    emit("move");
  }
};

// The view: one picture per moment. Codimate animates between them, pairing
// shapes by name: "ball" here is the same ball in every moment, so it travels.
const view = (frame: cm.Frame<Ball>) => {
  const scene = new cm.Scene();
  scene.rect("floor", { w: 1100, h: 4, at: [640, 404] }).fill("#48484a");
  scene.circle("ball", { r: 40, at: [frame.state.x, 360] }).fill("#2997ff");
  scene.text("label", "ball", { size: 28, at: cm.at({ x: frame.state.x, top: 420 }) });
  return scene;
};

const film = cm.explain({ trace: cm.trace(walk, { x: 200 }), view });
const out = process.argv[2] ?? "results/js/ball.mp4";
await (out.endsWith(".json") ? film.exportWeb(out) : film.render(out, { fps: 30 }));
console.log(`wrote ${out} (${film.duration.toFixed(1)} s)`);
