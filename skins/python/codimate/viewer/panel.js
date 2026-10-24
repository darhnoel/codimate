// The right-hand panel: what is selected, the notes, and the covered labels.

import { html } from "./html.js";
import { useState } from "preact/hooks";
import { film, time, beat, chapter, picked, tab, notes, stamp, shapeNamed,
         addNote, noteText, jumpTo, shapesReady, playing } from "./state.js";

export function Panel() {
  const issues = film.value?.covered ?? [];
  const tabs = [["inspect", "Inspect"], ["notes", `Notes ${notes.value.length}`],
                ["issues", `Issues ${issues.length}`]];
  return html`
    <aside class="panel">
      <div class="tabs" role="tablist">
        ${tabs.map(([id, label]) => html`
          <button role="tab" aria-selected=${tab.value === id}
                  class=${id === "issues" && !issues.length ? "empty" : ""}
                  onClick=${() => (tab.value = id)}>${label}</button>`)}
      </div>
      <div class="body">
        ${tab.value === "inspect" ? html`<${Inspect} />`
          : tab.value === "notes" ? html`<${Notes} />` : html`<${Issues} />`}
      </div>
    </aside>`;
}

function Inspect() {
  const p = picked.value, b = beat.value;
  shapesReady.value;               // redraw once the shapes at this beat arrive
  // Playing, the shapes are not asked for: that would be every beat, and
  // on a big film hundreds of kilobytes each. Pause, and they are back.
  const moving = playing.value;
  const shape = p && !moving && shapeNamed(p.name);
  return html`
    ${p ? html`
      <section>
        <h3 class="name">${p.name}</h3>
        ${shape ? html`
          <p class="quiet">${shape.kind} · layer ${shape.layer}</p>
          ${shape.box && html`
            <dl class="fields">
              ${["x", "y", "w", "h"].map((k, i) => html`
                <div><dt>${k.toUpperCase()}</dt><dd>${round(shape.box[i])}</dd></div>`)}
            </dl>`}`
        : html`<p class="quiet">${moving ? "playing — pause to inspect"
                                          : "not on screen at this moment"}</p>`}
        ${!moving && p.stack.length > 1 && html`
          <h4>Stacked here</h4>
          <ul class="stack">
            ${p.stack.map((n) => html`
              <li><button class=${n === p.name ? "on" : ""}
                    onClick=${() => (picked.value = { ...p, name: n })}>${n}</button></li>`)}
          </ul>`}
      </section>`
    : html`<p class="hint">Click anything in the frame. Click again to reach what is beneath it.</p>`}
    <section>
      <h4>Moment</h4>
      <dl class="rows">
        <dt>time</dt><dd>${stamp(time.value)}</dd>
        <dt>event</dt><dd>${b?.event ?? "—"}</dd>
        ${chapter.value && html`<dt>chapter</dt><dd>${chapter.value}</dd>`}
      </dl>
    </section>
    <section>
      <h4>State</h4>
      <div class="tree"><${Tree} value=${b?.state} top /></div>
    </section>
    <${Note} />`;
}

const round = (v) => Math.round(v * 10) / 10;

function Note() {
  const [words, setWords] = useState("");
  const add = () => { addNote(words); setWords(""); };
  return html`
    <section class="note">
      <textarea id="note" rows="2" value=${words} placeholder="What is wrong here?  (Enter adds)"
                onInput=${(e) => setWords(e.currentTarget.value)}
                onKeyDown=${(e) => {
                  if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); add(); }
                  if (e.key === "Escape") e.currentTarget.blur();
                }} />
      <button class="primary" onClick=${add}>Add note</button>
    </section>`;
}

function Notes() {
  const all = notes.value;
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    await navigator.clipboard.writeText(all.map(noteText).join("\n\n") + "\n");
    setCopied(true); setTimeout(() => setCopied(false), 1200);
  };
  const clear = () => { if (confirm(`Delete all ${all.length} notes?`)) notes.value = []; };
  if (!all.length) return html`<p class="hint">No notes yet. Select something, write what is wrong, press Enter.</p>`;
  return html`
    <div class="actions">
      <button class="primary" onClick=${copy}>${copied ? "Copied" : "Copy all"}</button>
      <button onClick=${clear}>Clear</button>
    </div>
    <ol class="list">
      ${all.map((n, i) => html`
        <li>
          <button class="item" onClick=${() => jumpTo(n.t, n.name)}>
            <span class="row"><time>${stamp(n.t)}</time><code>${n.name ?? "frame"}</code></span>
            ${n.words && html`<span class="words">${n.words}</span>`}
          </button>
          <button class="remove" aria-label="Delete note" title="Delete"
                  onClick=${() => (notes.value = all.filter((_, j) => j !== i))}>×</button>
        </li>`)}
    </ol>`;
}

function Issues() {
  const all = film.value?.covered ?? [];
  if (!all.length) return html`<p class="hint">Nothing is drawn over a label.</p>`;
  return html`
    <p class="hint">Something is drawn over these labels. Each is listed where it starts.</p>
    <ol class="list">
      ${all.map(([t, above, below]) => html`
        <li><button class="item" onClick=${() => jumpTo(t, below, [below, above])}>
          <span class="row"><time>${stamp(t)}</time></span>
          <span class="words"><code>${above}</code> over <code>${below}</code></span>
        </button></li>`)}
    </ol>`;
}

// The State, folded: objects and lists open one level at a time. An Item
// ({value, id}) is shown as its value, since that is what it stands for.
function Tree({ value, name, top = false, depth = 0 }) {
  const label = name !== undefined && html`<span class="key">${name}</span>`;
  if (value === null || typeof value !== "object") {
    return top ? html`<span class="quiet">—</span>`
      : html`<div class="leaf">${label}<span class=${"v " + typeof value}>${show(value)}</span></div>`;
  }
  if (isItem(value)) {
    return html`<div class="leaf">${label}<span class="v">${show(value.value)}</span><span class="quiet"> #${value.id}</span></div>`;
  }
  const entries = Array.isArray(value) ? value.map((v, i) => [i, v]) : Object.entries(value);
  const rows = entries.slice(0, 200).map(([k, v]) =>
    html`<${Tree} key=${k} name=${k} value=${v} depth=${depth + 1} />`);
  if (entries.length > 200) rows.push(html`<div class="quiet">… ${entries.length - 200} more</div>`);
  if (top) return rows;
  return html`
    <details open=${depth < 1 && entries.length <= 12}>
      <summary>${label}<span class="quiet">${Array.isArray(value) ? `[${entries.length}]` : `{${entries.length}}`}</span></summary>
      <div class="kids">${rows}</div>
    </details>`;
}

const isItem = (v) => v && !Array.isArray(v) && Object.keys(v).length === 2 && "value" in v && "id" in v;
const show = (v) => typeof v === "string" ? JSON.stringify(v)
  : typeof v === "number" && !Number.isInteger(v) ? String(+v.toPrecision(6)) : String(v);
