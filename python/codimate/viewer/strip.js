// One picture per chapter — where `emit(..., chapter="…")` says a part of
// the film begins. Films without chapters have no strip.

import { html } from "./html.js";
import { useEffect, useState, useRef } from "preact/hooks";
import { film, time, stamp, jumpTo } from "./state.js";

// A chapter's picture is taken from its middle: its first beat is often the
// changeover, with the old caption gone and the new one not yet in.
function spans(f) {
  const cs = f.chapters;
  return cs.map(([at, name], i) => {
    const end = i + 1 < cs.length ? cs[i + 1][0] : f.duration;
    return { at, end, name, middle: (at + end) / 2 };
  });
}

export function Strip() {
  const f = film.value;
  const [thumbs, setThumbs] = useState({});
  const active = useRef(null);

  // Drawn one after another, newest build only: a reload mid-way starts over.
  useEffect(() => {
    if (!f?.chapters.length) return;
    let live = true;
    const urls = {};
    (async () => {
      for (const c of spans(f)) {
        const blob = await (await fetch(`/frame?t=${c.middle}`)).blob();
        if (!live) return;
        urls[c.at] = URL.createObjectURL(blob);
        setThumbs({ ...urls });
      }
    })();
    return () => { live = false; Object.values(urls).forEach(URL.revokeObjectURL); };
  }, [f?.built]);

  const t = time.value;
  const all = f?.chapters.length ? spans(f) : [];
  const current = all.findIndex((c, i) =>
    c.at <= t + 1e-6 && (t < c.end || i === all.length - 1));
  useEffect(() => active.current?.scrollIntoView({ block: "nearest" }), [current]);
  if (!all.length) return null;

  return html`
    <nav class="strip">
      ${all.map((c, i) => {
        const on = i === current;
        return html`
          <button class=${"chapter" + (on ? " on" : "")} ref=${on ? active : null}
                  onClick=${() => jumpTo(c.at + 1e-3)} title=${c.name}>
            <div class="thumb">${thumbs[c.at] && html`<img src=${thumbs[c.at]} alt="" />`}</div>
            <div class="caption"><span>${c.name}</span><time>${stamp(c.at)}</time></div>
          </button>`;
      })}
    </nav>`;
}
