// A formula drawn on, a picture, vector art, an overlay title the camera
// leaves where it is, and a camera that moves to frame two things.

import * as cm from "../../skins/js/src/index.ts";

const walk = (state: { look: boolean }, emit: cm.Emit) => {
  state.look = true;
  emit("zoom");
};

const view = (frame: cm.Frame<{ look: boolean }>) => {
  const scene = new cm.Scene();
  scene.formula("eq", String.raw`\frac{QK^{T}}{\sqrt{d_k}}`, { size: 40, at: [400, 300] })
    .write(frame.state.look ? 1.0 : 0.3, 2);
  scene.image("photo", "figures/attention.png", { size: [320, 180], at: [900, 300] });
  scene.svg("logo", "brand.svg", { size: 90, at: cm.at({ x: 1180, top: 24 }) });
  const hud = scene.overlay();
  hud.text("title", "Scaled dot-product attention", { size: 32, at: [640, 52] });
  if (frame.state.look) scene.focus(["eq", "photo"], { pad: 60 });
  return scene;
};

await cm.explain({ trace: cm.trace(walk, { look: false }), view }).exportWeb(process.argv[2]);
