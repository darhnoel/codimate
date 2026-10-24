// Bars standing on Slots, each a Group of a bar and its label, and a nested
// Group (a card with a badge) that moves as one thing between moments.

import * as cm from "../../skins/js/src/index.ts";

type State = { values: number[]; card: number };

const walk = (state: State, emit: cm.Emit) => {
  state.values = [3, 5, 2];
  state.card = 900;
  emit("grow");
};

const view = (frame: cm.Frame<State>) => {
  const scene = new cm.Scene();
  frame.state.values.forEach((value, i) => {
    const slot: cm.Slot = { x: 200 + i * 160, y: 560, w: 120, h: 120, anchor: "bottom" };
    const bar = scene.group(["bar", i], slot);
    bar.rect("fill", { h: value * 60, at: cm.at({ bottom: 0 }) }).fill("#2997ff");
    bar.text("value", value, { size: 24, at: cm.at({ top: 12 }) });
  });
  const card = scene.group("card", undefined, { at: [frame.state.card, 300], w: 240 });
  card.rect("back", { h: 140 }).fill("#1c1c1e", { edge: "#3a3a3c", edgeW: 2 });
  const badge = card.group("badge", undefined, { at: [80, -50] });
  badge.circle("dot", { r: 18 }).fill("#ff453a");
  badge.text("n", "!", { size: 20 });
  return scene;
};

await cm.explain({ trace: cm.trace(walk, { values: [1, 2, 3], card: 700 }), view }).exportWeb(process.argv[2]);
