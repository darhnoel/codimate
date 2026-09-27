// The Previewer's model: what is loaded, where the playhead is, what is
// pointed at, and the notes. No markup here — every panel reads these signals
// and redraws itself when one it read changes.

import { signal, computed, effect, batch } from "@preact/signals";

export const film = signal(null);       // the /index answer: the whole index
export const time = signal(0);
export const playing = signal(false);
export const frame = signal(null);      // object URL of the drawn picture (script mode)
export const hover = signal(null);      // name under the cursor, while paused
export const picked = signal(null);     // { name, stack, point, pair? }
export const tab = signal("inspect");
export const notes = signal([]);

export const stamp = (s) =>
  `${Math.floor(s / 60)}:${(s % 60).toFixed(2).padStart(5, "0")}`;

// ------------------------------------------------------------ the index

// Which beat `t` is in: the last one to have started.
export function beatAt(t) {
  const beats = film.value?.beats ?? [];
  let lo = 0, hi = beats.length - 1;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (beats[mid].at <= t) lo = mid; else hi = mid - 1;
  }
  return lo;
}

export const beat = computed(() => film.value?.beats[beatAt(time.value)] ?? null);

export const chapter = computed(() => {
  const found = (film.value?.chapters ?? []).filter(([at]) => at <= time.value + 1e-6);
  return found.length ? found[found.length - 1][1] : null;
});

// Shapes are stored as differences (see Explanation.index), so what is on
// screen after beat i is every beat up to i applied in order. Checkpoints
// every 32 beats keep a lookup from replaying a thousand of them.
let checkpoints = [];
effect(() => { film.value; checkpoints = []; });

function after(i) {
  const beats = film.value.beats;
  let from = Math.min(Math.floor(i / 32), checkpoints.length) - 1;
  while (from >= 0 && !checkpoints[from]) from--;
  const now = new Map(from >= 0 ? checkpoints[from] : []);
  for (let k = from >= 0 ? from * 32 + 32 : 0; k <= i; k++) {
    for (const s of beats[k].shapes.set) now.set(s.name, s);
    for (const n of beats[k].shapes.gone) now.delete(n);
    if (k % 32 === 31 && !checkpoints[(k - 31) / 32]) checkpoints[(k - 31) / 32] = new Map(now);
  }
  return now;
}

// Every shape on screen at `t`, topmost first — the Engine draws in
// (layer, name) order. Mid-beat, the nearer end of the beat is reported,
// as `preview.at` does in Python.
let memo = { key: null, shapes: [] };
export function shapesAt(t) {
  if (!film.value) return [];
  const i = beatAt(t), b = film.value.beats[i];
  const early = i > 0 && t < b.at + b.secs / 2;
  const key = `${film.value.built}:${early ? i - 1 : i}`;
  if (memo.key !== key) {
    const shapes = [...after(early ? i - 1 : i).values()];
    shapes.sort((a, b) => b.layer - a.layer || (a.name < b.name ? 1 : a.name > b.name ? -1 : 0));
    memo = { key, shapes };
  }
  return memo.shapes;
}

// The shapes under a point, topmost first. A line is given a few pixels so
// it can be pointed at at all.
export function hitsAt(t, x, y) {
  return shapesAt(t).filter(({ box }) => {
    if (!box) return false;
    const [l, top, w, h] = box, pad = Math.min(w, h) < 4 ? 4 : 0;
    return l - pad <= x && x <= l + w + pad && top - pad <= y && y <= top + h + pad;
  });
}

export const shapeNamed = (name, t = time.value) => shapesAt(t).find((s) => s.name === name);

// ------------------------------------------------------------ the clock

// Two sources behind one playhead: a rendered mp4 plays itself; a script is
// drawn a frame at a time, with its mixed sound (if any) keeping the time.
let video = null;
export const attachVideo = (el) => { video = el; };
const sound = new Audio();
let from = null;

export function seek(t) {
  const end = film.value?.duration ?? 0;
  t = Math.max(0, Math.min(end, t));
  time.value = t;
  if (video) video.currentTime = t;
  if (playing.value && film.value?.sound) sound.currentTime = t;
}

export function play() {
  if (!film.value || playing.value) return;
  if (time.value >= film.value.duration - 0.01) seek(0);
  batch(() => { playing.value = true; hover.value = null; });
  from = performance.now() / 1000;
  if (video) video.play();
  else if (film.value.sound) { sound.currentTime = time.value; sound.play(); }
  requestAnimationFrame(run);
}

export function pause() {
  playing.value = false;
  if (video) video.pause();
  sound.pause();
}

export const toggle = () => (playing.value ? pause() : play());

function run() {
  if (!playing.value) return;
  const now = performance.now() / 1000;
  // Past the last clip the film still has its final hold: back to the wall clock.
  const t = video ? video.currentTime
    : film.value.sound && !sound.ended ? sound.currentTime
    : time.value + (now - from);
  from = now;
  time.value = Math.min(t, film.value.duration);
  if (time.value >= film.value.duration) pause();
  else requestAnimationFrame(run);
}

// Frames are drawn on request. One in flight; a fast drag skips to wherever
// it ends up, and playing drops frames rather than falling behind.
let busy = false, wanted = null;
effect(() => {
  const t = time.value, f = film.value;
  if (!f || f.video) return;
  wanted = [t, f.built];
  if (!busy) fetchFrame();
});
async function fetchFrame() {
  busy = true;
  while (wanted) {
    const [t] = wanted; wanted = null;
    try {
      const blob = await (await fetch(`/frame?t=${t}`)).blob();
      const old = frame.value;
      frame.value = URL.createObjectURL(blob);
      if (old) URL.revokeObjectURL(old);
    } catch { /* the next request will do */ }
  }
  busy = false;
}

// ------------------------------------------------------------ loading

// Polled: this is also how a saved edit to the script arrives.
export async function load(force = false) {
  const got = await (await fetch("/index" + (force ? "?force" : ""))).json();
  const changed = !film.value || got.built !== film.value.built || got.error !== film.value.error;
  if (!changed) return;
  batch(() => {
    const first = !film.value;
    film.value = got;
    if (first) notes.value = readNotes();
    if (got.sound) sound.src = `/sound?built=${got.built}`;
    if (time.value > got.duration) time.value = got.duration;
  });
}

// ------------------------------------------------------------ notes

// Kept in this browser, per film, so a refresh does not lose a review. The
// Previewer never writes a file (ADR 0018); this never leaves the browser.
const noteKey = () => `codimate-notes:${film.value.key}`;
function readNotes() {
  try { return JSON.parse(localStorage.getItem(noteKey())) ?? []; } catch { return []; }
}
effect(() => {
  const all = notes.value;
  if (!film.value) return;
  try { localStorage.setItem(noteKey(), JSON.stringify(all)); } catch { /* private mode */ }
});

export function addNote(words) {
  const t = time.value, name = picked.value?.name ?? null;
  const shape = name && shapeNamed(name, t);
  const under = picked.value?.stack.filter((n) => n !== name) ?? [];
  notes.value = [...notes.value, {
    t, name, words: words.trim(), under: under.slice(0, 5),
    event: beat.value?.event, chapter: chapter.value,
    state: brief(beat.value?.state), box: shape?.box ?? null,
  }];
}

// The same shape as `preview.note` in Python: plain text a person or an
// agent can act on.
export function noteText(n) {
  const lines = [`${stamp(n.t)}  ${n.name ?? "(the whole frame)"}`, `  event    ${n.event}`];
  if (n.chapter) lines.push(`  chapter  ${n.chapter}`);
  if (n.under?.length) lines.push(`  under    ${n.under.join(", ")}`);
  lines.push(`  state    ${n.state}`);
  if (n.words) lines.push(`> ${n.words}`);
  return lines.join("\n");
}

function short(v, room = 24) {
  const text = typeof v === "string" ? v
    : typeof v === "number" && !Number.isInteger(v) ? String(+v.toPrecision(4))
    : JSON.stringify(v);
  return text.length <= room ? text : text.slice(0, room - 1) + "…";
}
export function brief(state, room = 96) {
  const text = state && typeof state === "object" && !Array.isArray(state)
    ? Object.entries(state).map(([k, v]) => `${k}=${short(v)}`).join("  ")
    : short(state, 200);
  return text.length <= room ? text : text.slice(0, room - 1) + "…";
}

// ------------------------------------------------------------ picking

// Click once: the topmost shape. Click the same spot again: the one beneath
// it, and so on round the stack — as in Figma.
export function pickAt(x, y) {
  const stack = hitsAt(time.value, x, y).map((s) => s.name);
  const last = picked.value;
  const again = last && last.point && Math.hypot(last.point[0] - x, last.point[1] - y) < 3
    && last.stack.join() === stack.join();
  if (!stack.length) { picked.value = null; return; }
  const name = again ? stack[(stack.indexOf(last.name) + 1) % stack.length] : stack[0];
  picked.value = { name, stack, point: [x, y] };
}

export function jumpTo(t, name = null, pair = null) {
  pause();
  seek(t);
  picked.value = name ? { name, stack: pair ?? [name], point: null, pair } : null;
}
