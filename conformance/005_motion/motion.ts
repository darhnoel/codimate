// Motion and timing: bars swapped by a lift-carry-drop path, a card on an
// arc, labels that spring in turn, and events with their own durations.

import * as cm from "../../skins/js/src/index.ts";

const sort = (items: cm.Item<number>[], emit: cm.Emit) => {
  [items[0], items[2]] = [items[2], items[0]];
  emit("swap", { items: [items[0], items[2]] });
  items.reverse();
  emit("flip");
};

const view = (frame: cm.Frame<cm.Item<number>[]>) => {
  const scene = new cm.Scene();
  for (const [slot, item] of cm.row(frame.state, { gap: 40, size: 120 })) {
    const bar = scene.group(["bar", item.id], slot);
    bar.rect("fill", { h: item.value * 40, at: cm.at({ bottom: 0 }) }).fill("#2997ff");
    bar.text("label", item.value, { size: 24, at: cm.at({ top: 12 }) });
  }
  const swapped = frame.is("swap");
  scene.rect("card", { w: 160, h: 90, at: [swapped ? 1100 : 180, 120] }).fill("#ff9f0a");
  return scene;
};

const motion = [
  new cm.Rule(["bar", "*", "fill"], "lift_carry_drop", { clearance: 90 }),
  new cm.Rule("card", "arc", { bend: -0.4, ease: "out" }),
  new cm.Rule(["bar", "*", "label"], undefined, { ease: "spring", stagger: 0.1, order: "centre", bounciness: 0.5 }),
];
const timing = new cm.Timing({ default: 0.8, events: { swap: 1.5 }, opening: 0.4, finalHold: 1.0 });
await cm.explain({ trace: cm.trace(sort, cm.items([1, 4, 2, 5])), view, motion, timing }).exportWeb(process.argv[2]);
