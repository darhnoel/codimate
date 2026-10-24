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
