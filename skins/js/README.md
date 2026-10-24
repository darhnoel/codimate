# codimate for JavaScript and TypeScript

Turn a running algorithm into an explainer video, from TypeScript. This is the
same library as the Python `codimate`: the same names, the same arguments,
and the same Rust core underneath, so one explanation makes the same film in
either language.

Not on npm yet. You run it from this repository.

## Set up (once)

You need Node 23.6 or newer (it runs `.ts` files directly), Rust with the
`wasm32-unknown-unknown` target, and the `wasm-bindgen` CLI at the version in
`Cargo.lock`. From the repository root:

```sh
rustup target add wasm32-unknown-unknown
.venv/bin/python tools/build_js.py
```

That builds the core as WebAssembly into `skins/js/pkg/` and a browser copy of
the Skin into `skins/js/dist/`. Run it again whenever the Rust code changes.

Making an MP4 also needs `ffmpeg` on your `PATH` (or set `CODIMATE_FFMPEG`).

## Your first film

```ts
import * as cm from "../src/index.ts"; // the path to skins/js/src/index.ts

type Ball = { x: number };

// The algorithm: change the state, then say what happened.
const walk = (ball: Ball, emit: cm.Emit) => {
  for (const x of [640, 1080]) {
    ball.x = x;
    emit("move");
  }
};

// The view: one picture per moment. Shapes with the same name in two moments
// are the same shape, so "ball" travels from one place to the next.
const view = (frame: cm.Frame<Ball>) => {
  const scene = new cm.Scene();
  scene.circle("ball", { r: 40, at: [frame.state.x, 360] }).fill("#2997ff");
  scene.text("label", "ball", { size: 28, at: cm.at({ x: frame.state.x, top: 420 }) });
  return scene;
};

const film = cm.explain({ trace: cm.trace(walk, { x: 200 }), view });
await film.render("ball.mp4");
```

```sh
node ball.ts
```

## Examples

Three runnable files in [examples/](examples/). Give an output path ending in
`.mp4` for a video, or in `.json` for the film data only (fast, no ffmpeg):

```sh
node skins/js/examples/01_ball.ts ball.mp4
node skins/js/examples/02_bubble_sort.ts sort.mp4
node skins/js/examples/03_plot.ts plot.json
```

| File | Shows |
|---|---|
| `01_ball.ts` | state, `trace`/`emit`, a view, `explain` |
| `02_bubble_sort.ts` | `cm.items`, `cm.row`, groups, a motion `Rule`, `Timing` |
| `03_plot.ts` | `cm.axes`, a text preset, a highlight, the camera (`focus`) |

More, each with its Python twin, are in [conformance/](../../conformance/).

## From Python to TypeScript

Everything in the [Python reference](https://darhnoel.github.io/codimate/)
works here, translated by three rules:

| Python | TypeScript |
|---|---|
| `snake_case` names | `camelCase`: `words_up` → `wordsUp`, `final_hold` → `finalHold` |
| keyword arguments | one options object at the end: `circle("ball", r=40)` → `circle("ball", { r: 40 })` |
| tuples | arrays: `at=(640, 360)` → `at: [640, 360]` |

And a few things that are about the language, not the library:

- **State** is a plain object or class instance (`type Ball = { x: number }`).
  `trace` copies it at every `emit`, keeping classes; pass
  `{ snapshot: (s) => ... }` as the third argument to copy it yourself.
- **Event data:** Python's `emit("move", x=x)` is `emit("move", { x })`.
- **Items:** `cm.items([5, 2, 4])` gives values an identity. Compare their
  values with `a.valueOf() > b.valueOf()` and their identity with `a.is(b)`.
- **Timing** takes an options object:
  `new cm.Timing({ default: 0.5, events: { swap: 0.9 } })`. Subclass it and
  override `forEvent(event)` for timing of your own.
- **Rule** paths and eases are strings, as in Python:
  `new cm.Rule("dot", "arc", { ease: "in_out" })`.

Editors show each function's documentation on hover. Its code samples are
still the Python ones for now.

## Output

| Call | Where | Gives |
|---|---|---|
| `await film.render("out.mp4", { fps: 30 })` | Node | an MP4, through ffmpeg |
| `await film.exportWeb("out.json")` | Node | the film data, to play later |
| `film.play(canvas)` | browser | the film, live on a `<canvas>`; returns a function that stops it |
| `film.timeline()`, `film.chapters()` | both | when each event plays, in seconds |
| `film.report()` | both | labels that something covers, or `""` |

## In the browser

Browsers do not run TypeScript, so a page imports the built copy in `dist/`:

```sh
cd skins/js && python3 -m http.server 8011
# open http://127.0.0.1:8011/demo/
```

[demo/index.html](demo/index.html) is the smallest page that plays a film. Write
your own the same way: `import * as cm from "../dist/index.js"`, then
`cm.explain({ ... }).play(canvas)`.

## Not here yet

- Sound (Python only for now).
- The Science Kit, the Previewer and the index stay in Python.
- In the browser, the covered-label check skips formulas.
- The built-in fonts have no emoji.

## Tests

```sh
node --test skins/js/test/skin.test.ts
.venv/bin/python tools/conformance.py   # every example, byte for byte, in both languages
```
