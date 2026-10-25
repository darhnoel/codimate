// What the JavaScript Skin does by hand: the trace, snapshots, and keeping
// shapes. Everything it draws is the Authoring Core's, tested there and pinned
// across languages by the Conformance Examples.

import { test } from "node:test";
import assert from "node:assert/strict";

import * as cm from "../src/index.ts";

test("a snapshot keeps the state's class, so its methods still work", () => {
  class Counter {
    n = 0;
    double() {
      return this.n * 2;
    }
  }
  const trace = cm.trace((c: Counter, emit) => {
    c.n = 3;
    emit("count");
  }, new Counter());
  assert.equal(trace.initial.n, 0);
  assert.equal(trace.events[0].state.double(), 6);
});

test("each emit keeps its own snapshot, taken when it was called", () => {
  const trace = cm.trace((s: { x: number }, emit) => {
    for (const x of [1, 2, 3]) {
      s.x = x;
      emit("move", { x });
    }
  }, { x: 0 });
  assert.deepEqual(trace.events.map((e) => e.state.x), [1, 2, 3]);
  assert.deepEqual(trace.events.map((e) => e.data.x), [1, 2, 3]);
});

test("an Item is still itself after a snapshot, and compares by value", () => {
  const [a, b] = cm.items([3, 1]);
  const trace = cm.trace((list: cm.Item<number>[], emit) => {
    list.reverse();
    emit("swap");
  }, [a, b]);
  const [first, second] = trace.events[0].state;
  assert.ok(first.is(b) && second.is(a));
  assert.ok(a.valueOf() > b.valueOf());
});

test("a frame says which event made it", () => {
  const frame = new cm.Frame({}, { name: "swap", state: {}, data: { items: [1, 2] } });
  assert.ok(frame.is("swap") && !frame.is("compare"));
  assert.deepEqual(frame.items(), [1, 2]);
  assert.deepEqual(new cm.Frame({}, null).items(), []);
});

test("a refusal from the Core is thrown at the author's call", () => {
  const scene = new cm.Scene();
  assert.throws(() => scene.rect("r", { h: 4, at: [0, 0] }), /give one of w=/);
});

test("two shapes may not share a name", () => {
  const scene = new cm.Scene();
  scene.circle("dot", { r: 2, at: [0, 0] });
  assert.throws(() => scene.circle("dot", { r: 2, at: [9, 9] }), /share the name 'dot'/);
});

test("options are camelCase and reach the Core under its names", () => {
  const scene = new cm.Scene();
  scene.circle("c", { r: 2, at: [0, 0] }).fill("red", { edge: "#000", edgeW: 3 });
  const [shape] = scene._payload();
  assert.equal(shape.edge_w, 3);
  assert.equal(shape.color, "red");
});

test("a character keeps its connected joint pose and expression in one Scene Item", () => {
  const scene = new cm.Scene();
  const guide = scene.character("guide", cm.characters.codimate, { size: 360, at: [640, 360] });
  guide.part("right-arm").turn(-25);
  guide.expression("thinking");
  guide.theme("dark");
  const [shape] = scene._payload();
  assert.equal(shape.kind, "character");
  assert.equal(shape.text, "codimate:codimate-mascot");
  assert.deepEqual(JSON.parse(String(shape.pose)), {
    expression: "thinking",
    joints: { "right-arm": -25 },
    theme: "dark",
  });
});

test("a character can cut between expressions without fading its mouth", () => {
  const scene = new cm.Scene();
  const guide = scene.character("guide", cm.characters.codimate, { at: [640, 360] });
  guide.expression("talk-open", { transition: "cut" });
  guide.part("right-hand").turn(28);
  guide.theme("dark");
  const [shape] = scene._payload();
  assert.deepEqual(JSON.parse(String(shape.pose)), {
    expression: "talk-open",
    expression_transition: "cut",
    theme: "dark",
    joints: { "right-hand": 28 },
  });
});

test("a character rejects an unknown color theme", () => {
  const guide = new cm.Scene().character("guide", cm.characters.codimate, { at: [640, 360] });
  assert.throws(() => guide.theme("sepia"), /theme must be 'light' or 'dark'/);
});

test("a film opens and closes on held moments", () => {
  const film = cm.explain({
    trace: cm.trace((s: { x: number }, emit) => emit("move"), { x: 0 }),
    view: () => new cm.Scene(),
    timing: new cm.Timing({ default: 1, opening: 0.5, finalHold: 2 }),
  });
  assert.deepEqual(film.durations, [0.5, 1, 2]);
  assert.equal(film.scenes.length, 4);
  assert.equal(film.duration, 3.5);
});

test("Timing is an extension point: a subclass decides each event's length", () => {
  class Paced extends cm.Timing {
    forEvent(event: cm.Event) {
      return Number(event.data.seconds ?? 0);
    }
  }
  const film = cm.explain({
    trace: cm.trace((s: object, emit) => {
      emit("a", { seconds: 2 });
      emit("b", { seconds: 0.5 });
    }, {}),
    view: () => new cm.Scene(),
    timing: new Paced({ opening: 0, finalHold: 0 }),
  });
  assert.deepEqual(film.durations, [0, 2, 0.5, 0]);
});

test("a Rule is checked by the Core, with the words Python uses", () => {
  assert.throws(() => new cm.Rule("*", "teleport"), /unknown path 'teleport'/);
  const rule = new cm.Rule(["bar", "*"], "arc", { bend: -0.4, ease: "out" });
  assert.equal(rule.pattern, "bar/*");
  assert.deepEqual(rule._payload(), ["bar/*", "arc", { bend: -0.4 }, "out", "screen"]);
});

test("timeline, chapters and covered labels are Python's, from the same film", async () => {
  const { readFileSync } = await import("node:fs");
  const recorded = JSON.parse(
    readFileSync(new URL("../../../crates/codimate-author/tests/fixtures/slice11.json", import.meta.url), "utf8"));
  const answer = (op: string) => recorded.find((c: any) => c.op === op).expected.ok;
  const walk = (state: { x: number }, emit: cm.Emit) => {
    state.x = 300;
    emit("slide", { chapter: "Start" });
    state.x = 640;
    emit("cover");
    state.x = 900;
    emit("clear", { chapter: "End" });
  };
  const view = (frame: cm.Frame<{ x: number }>) => {
    const scene = new cm.Scene();
    const x = frame.state.x;
    scene.rect("plate", { w: 240, h: 60, at: [640, 300] }).fill("#1c1c1e");
    scene.text("label", "A label", { size: 30, at: [640, 300] });
    scene.circle("ball", { r: 40, at: [x, 300] }).fill("#2997ff").on(11);
    scene.line("edge", { start: [150, 180], end: [300, 60], w: 2 }).on(11);
    scene.text("near", "near", { size: 20, at: [220, 130] });
    scene.text("a", "abc", { size: 24, at: [500, 600] });
    scene.text("b", "abc", { size: 24, at: [520, 600] });
    if (x === 900) {
      scene.overlay().text("title", "Title", { size: 30, at: [640, 40] });
      scene.rect("box", { w: 100, h: 100, at: [640, 40] }).on(12);
      scene.focus(["plate"]);
    }
    return scene;
  };
  const film = cm.explain({
    trace: cm.trace(walk, { x: 100 }), view,
    timing: new cm.Timing({ default: 0.7, events: { cover: 1.25 } }),
  });
  assert.deepEqual(film.timeline(), answer("timeline"));
  assert.deepEqual(film.chapters(), answer("chapters"));
  assert.deepEqual(film.covered(), answer("film_covered"));
  assert.equal(film.report(), answer("report"));
});
