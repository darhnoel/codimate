// Text presets: words that rise into place, letters (Khmer and emoji kept
// whole), and a blurred title that resolves, each with its own rules.

import * as cm from "../../skins/js/src/index.ts";

const title = cm.wordsUpBlur("title", "Explain anything", { style: new cm.TextPresetStyle({ size: 56 }), at: [640, 160] });
const words = cm.wordsUp("words", "One idea per moment", { at: [640, 360] });
const letters = cm.lettersDown("letters", "ពន្យល់ 👩‍💻 ok", {
  style: new cm.TextPresetStyle({ size: 40, fill: "#ffd60a", distance: 30 }),
  at: [640, 540],
});
const STATES: Record<string, string[]> = {
  title: ["hidden", "blurred", "shown"],
  words: ["hidden", "shown", "shown"],
  letters: ["hidden", "hidden", "shown"],
};

const walk = (state: { step: number }, emit: cm.Emit) => {
  for (const step of [1, 2]) {
    state.step = step;
    emit("reveal");
  }
};

const view = (frame: cm.Frame<{ step: number }>) => {
  const scene = new cm.Scene();
  for (const preset of [title, words, letters]) {
    preset.draw(scene, { state: STATES[preset.name][frame.state.step] });
  }
  return scene;
};

const motion = [...title.rules(), ...words.rules({ stagger: 0.2 }), ...letters.rules({ ease: "spring" })];
await cm.explain({ trace: cm.trace(walk, { step: 0 }), view, motion }).exportWeb(process.argv[2]);
