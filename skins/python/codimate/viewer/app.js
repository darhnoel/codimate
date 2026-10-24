// The Previewer window: the filmstrip, the picture with its bar, and the
// panel — laid out as Figma lays out pages, canvas and properties.
//
// It never writes (ADR 0018). Everything it knows arrives from /index; what
// it produces is notes, which stay in this browser until copied out.

import { render } from "preact";
import { html } from "./html.js";
import { film, time, picked, tab, toggle, seek, pause, load } from "./state.js";
import { Stage } from "./stage.js";
import { Strip } from "./strip.js";
import { Bar } from "./bar.js";
import { Panel } from "./panel.js";

function App() {
  const f = film.value;
  if (!f) return html`<div class="loading">Loading…</div>`;
  return html`
    <div class=${"app" + (f.chapters.length ? "" : " no-strip")}>
      <header>
        <strong>${f.title}</strong>
        ${!f.video && html`<button class="icon" onClick=${() => load(true)}
            title="Run the script again (it also reruns on save)" aria-label="Reload">⟳</button>`}
        ${f.error && html`<span class="error" title=${f.error}>
            The script failed — showing the last good run. ${f.error.split("\n").pop()}</span>`}
      </header>
      <${Strip} />
      <main><${Stage} /><${Bar} /></main>
      <${Panel} />
    </div>`;
}

const chapterJump = (dir) => {
  const starts = film.value.chapters.map(([at]) => at), t = time.value;
  const next = dir > 0 ? starts.find((at) => at > t + 1e-3)
    : [...starts].reverse().find((at) => at < t - 0.25);
  if (next !== undefined) { pause(); seek(next + 1e-3); }
};

document.addEventListener("keydown", (e) => {
  if (!film.value || e.target.closest?.("textarea, input:not([type=range])")) return;
  const keys = {
    " ": toggle,
    ArrowLeft: () => { pause(); seek(time.value - (e.shiftKey ? 1 : 0.1)); },
    ArrowRight: () => { pause(); seek(time.value + (e.shiftKey ? 1 : 0.1)); },
    "[": () => chapterJump(-1),
    "]": () => chapterJump(1),
    Escape: () => (picked.value = null),
    n: () => { tab.value = "inspect"; requestAnimationFrame(() => document.getElementById("note")?.focus()); },
  };
  const act = keys[e.key];
  if (act) { e.preventDefault(); act(); }
});

load().then(() => {
  document.title = `${film.value.title} — Codimate Previewer`;
  if (!film.value.video) setInterval(() => load(), 1500);
});

render(html`<${App} />`, document.body);
