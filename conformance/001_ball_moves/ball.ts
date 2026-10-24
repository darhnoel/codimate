// A ball moves left to right twice, with its label following it.

import * as cm from "../../skins/js/src/index.ts";

type Ball = { x: number };

const walk = (ball: Ball, emit: cm.Emit) => {
  for (const x of [640, 1080]) {
    ball.x = x;
    emit("move");
  }
};

const view = (frame: cm.Frame<Ball>) => {
  const scene = new cm.Scene();
  scene.rect("floor", { h: 4, w: 1100, at: [640, 404] }).fill("#48484a");
  scene.circle("ball", { r: 40, at: [frame.state.x, 360] }).fill("#2997ff", { edge: "#ffffff" });
  scene.text("label", "ball", { size: 28, at: cm.at({ x: frame.state.x, top: 420 }) });
  return scene;
};

cm.canvas(1280, 720);
await cm.explain({ trace: cm.trace(walk, { x: 200 } satisfies Ball), view }).exportWeb(process.argv[2]);
