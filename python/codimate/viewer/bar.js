// A plain progress bar, as a video player has: play, where you are, how long.

import { html } from "./html.js";
import { film, time, playing, chapter, stamp, toggle, seek } from "./state.js";

const PLAY = html`<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 2.5v11l9-5.5z" /></svg>`;
const PAUSE = html`<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 2.5h3v11H4zM9 2.5h3v11H9z" /></svg>`;

export function Bar() {
  const end = film.value?.duration ?? 0, t = time.value;
  const done = end ? (t / end) * 100 : 0;
  return html`
    <div class="bar">
      <button class="play" onClick=${toggle} aria-label=${playing.value ? "Pause" : "Play"}
              title="space">${playing.value ? PAUSE : PLAY}</button>
      <input class="seek" type="range" min="0" max=${end} step="0.01" value=${t}
             aria-label="time" style=${{ "--done": done + "%" }}
             onInput=${(e) => seek(+e.currentTarget.value)} />
      <span class="clock">${stamp(t)} <span class="quiet">/ ${stamp(end)}</span></span>
      ${chapter.value && html`<span class="where">${chapter.value}</span>`}
    </div>`;
}
