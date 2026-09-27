// The picture, and pointing at it. Hover outlines what is under the cursor;
// click selects it. Boxes are in canvas coordinates, so everything drawn
// here is scaled by how large the picture happens to be on screen.

import { html } from "./html.js";
import { useEffect, useRef, useState } from "preact/hooks";
import { film, time, playing, frame, hover, picked, attachVideo, hitsAt,
         shapeNamed, pickAt, shapesReady } from "./state.js";

const BLUE = "#0d99ff", RED = "#f24822";

export function Stage() {
  const box = useRef(null), canvas = useRef(null), video = useRef(null);
  const [size, setSize] = useState([0, 0]);
  const f = film.value;
  const [W, H] = f?.canvas ?? [16, 9];

  // Fit the picture inside the stage, and the overlay exactly over it.
  useEffect(() => {
    // The content box, not clientWidth: the stage's padding is not room.
    const fit = ([{ contentRect: { width: cw, height: ch } }]) => {
      const k = Math.min(cw / W, ch / H);
      setSize([Math.floor(W * k), Math.floor(H * k)]);
    };
    const watch = new ResizeObserver(fit);
    watch.observe(box.current);
    return () => watch.disconnect();
  }, [W, H]);

  useEffect(() => {
    attachVideo(f?.video ? video.current : null);
    if (!f?.video) return;
    const el = video.current;
    const settle = () => { if (!playing.value) time.value = el.currentTime; };
    el.addEventListener("seeked", settle);
    el.addEventListener("ended", () => { playing.value = false; });
    return () => el.removeEventListener("seeked", settle);
  }, [f?.video]);

  // Redraw the outlines whenever anything they depend on changes.
  useEffect(() => draw(canvas.current, size, W), [
    size, time.value, hover.value, picked.value, playing.value, f, shapesReady.value]);

  const toCanvas = (e) => {
    const r = canvas.current.getBoundingClientRect();
    return [(e.clientX - r.left) * W / r.width, (e.clientY - r.top) * W / r.width];
  };
  const onMove = (e) => {
    if (playing.value) return;
    const [x, y] = toCanvas(e);
    hover.value = hitsAt(time.value, x, y)[0]?.name ?? null;
  };
  const onDown = (e) => { if (!playing.value) pickAt(...toCanvas(e)); };

  const [w, h] = size;
  return html`
    <div class="stage" ref=${box}>
      <div class="picture" style=${{ width: w + "px", height: h + "px" }}>
        ${f?.video
          ? html`<video ref=${video} src="/video" preload="auto" playsinline></video>`
          : frame.value && html`<img src=${frame.value} alt="the film at this moment" />`}
        <canvas ref=${canvas} onMouseMove=${onMove} onMouseLeave=${() => (hover.value = null)}
                onMouseDown=${onDown} />
      </div>
    </div>`;
}

function draw(canvas, [w, h], W) {
  if (!canvas || !w) return;
  const dpr = window.devicePixelRatio || 1;
  canvas.width = w * dpr; canvas.height = h * dpr;
  canvas.style.width = w + "px"; canvas.style.height = h + "px";
  const g = canvas.getContext("2d");
  g.setTransform(dpr, 0, 0, dpr, 0, 0);
  g.clearRect(0, 0, w, h);
  if (playing.value) return;
  const k = w / W;

  const outline = (name, color, width, dash = []) => {
    const s = shapeNamed(name);
    if (!s?.box) return null;
    const [x, y, bw, bh] = s.box.map((v) => v * k);
    g.setLineDash(dash); g.strokeStyle = color; g.lineWidth = width;
    g.strokeRect(x - 0.5, y - 0.5, Math.max(bw, 1) + 1, Math.max(bh, 1) + 1);
    g.setLineDash([]);
    return [x, y];
  };
  const tag = (name, [x, y], color) => {
    g.font = "11px Inter, ui-sans-serif, system-ui, sans-serif";
    const text = name.length > 48 ? name.slice(0, 47) + "…" : name;
    const tw = g.measureText(text).width + 8;
    const ty = y > 20 ? y - 19 : y + 2;
    g.fillStyle = color; g.fillRect(x, ty, tw, 17);
    g.fillStyle = "#fff"; g.fillText(text, x + 4, ty + 12.5);
  };

  const p = picked.value;
  if (p?.pair) p.pair.forEach((n) => outline(n, RED, 1.5, [4, 3]));
  if (p) { const at = outline(p.name, BLUE, 2); if (at) tag(p.name, at, BLUE); }
  if (hover.value && hover.value !== p?.name) {
    const at = outline(hover.value, BLUE, 1);
    if (at) tag(hover.value, at, "#2c2c2c");
  }
}
