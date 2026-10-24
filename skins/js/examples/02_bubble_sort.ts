// 02 · Bubble sort: bars that keep their identity while they swap places.
//
//   node skins/js/examples/02_bubble_sort.ts              → results/js/bubble_sort.mp4
//   node skins/js/examples/02_bubble_sort.ts sort.json    → the film's data, no video
//
// Shows: Items (two equal values are still two bars), cm.row for Slots, Groups,
// motion Rules (a lift-carry-drop swap) and per-event Timing.

import * as cm from "../src/index.ts";

type Bars = cm.Item<number>[];

// Ordinary bubble sort, emitting what it does. `items` give each value an
// identity, so a bar that moves is the same bar.
const bubbleSort = (bars: Bars, emit: cm.Emit) => {
  for (let end = bars.length - 1; end > 0; end--) {
    for (let i = 0; i < end; i++) {
      emit("compare", { items: [bars[i], bars[i + 1]] });
      if (bars[i].valueOf() > bars[i + 1].valueOf()) {
        [bars[i], bars[i + 1]] = [bars[i + 1], bars[i]];
        emit("swap", { items: [bars[i], bars[i + 1]] });
      }
    }
  }
  emit("done");
};

const view = (frame: cm.Frame<Bars>) => {
  const scene = new cm.Scene();
  const looking = frame.items() as cm.Item<number>[];
  scene.text("title", frame.is("done") ? "Sorted" : "Bubble sort", { size: 40, at: [640, 80] });
  // One Slot per bar, on a shared baseline; each bar is a Group named by its id.
  for (const [slot, item] of cm.row(frame.state, { gap: 24, size: 110 })) {
    const bar = scene.group(["bar", item.id], slot);
    const lit = looking.some((other) => other.is(item));
    bar.rect("fill", { h: item.value * 55, at: cm.at({ bottom: 0 }) })
      .fill(lit ? "#ff9f0a" : "#2997ff")
      .round(8);
    bar.text("value", item.value, { size: 28, at: cm.at({ top: 14 }) });
  }
  return scene;
};

const film = cm.explain({
  trace: cm.trace(bubbleSort, cm.items([5, 2, 4, 1, 3])),
  view,
  motion: [new cm.Rule(["bar", "*"], "lift_carry_drop", { clearance: 80 })],
  timing: new cm.Timing({ default: 0.5, events: { swap: 0.9, done: 1.0 } }),
});
const out = process.argv[2] ?? "results/js/bubble_sort.mp4";
await (out.endsWith(".json") ? film.exportWeb(out) : film.render(out, { fps: 30 }));
console.log(`wrote ${out} (${film.duration.toFixed(1)} s)`);
