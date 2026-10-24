// What happened: Items with identity, and the Trace an algorithm records.
// The trace runs the author's own function, so it lives in the Skin; its
// behaviour is Python's (trace.py), pinned by the Conformance Examples.

let nextId = 0;

/** A value with an identity of its own: two 3s in a list are two bars. */
export class Item<T = unknown> {
  readonly id: number | string;
  value: T;

  constructor(value: T, id?: number | string) {
    this.value = value;
    this.id = id ?? nextId++;
  }

  /** Compare by value, so an algorithm sorts normally: `a.valueOf() > b.valueOf()`. */
  valueOf(): T {
    return this.value;
  }

  /** The same thing, after any number of snapshots, if the ids match. */
  is(other: unknown): boolean {
    return other instanceof Item && other.id === this.id;
  }
}

/** Give each value an identity of its own, so equal values are still two things. */
export const items = <T>(values: Iterable<T>): Item<T>[] => [...values].map((v) => new Item(v));

export interface Event<S = unknown> {
  readonly name: string;
  readonly state: S;
  readonly data: Record<string, unknown>;
}

export interface Trace<S = unknown> {
  readonly initial: S;
  readonly events: readonly Event<S>[];
}

/** Record that something just happened: call it *after* changing the state. */
export type Emit = (name: string, data?: Record<string, unknown>) => void;

/** What a view function receives: the state, and what just happened (`null`
 * for the opening moment). */
export class Frame<S = unknown> {
  readonly state: S;
  readonly event: Event<S> | null;

  constructor(state: S, event: Event<S> | null) {
    this.state = state;
    this.event = event;
  }

  /** Did this moment come from an event called `name`? */
  is(name: string): boolean {
    return this.event !== null && this.event.name === name;
  }

  /** The things this event named, e.g. `frame.items()` after `emit("compare", { items: [a, b] })`. */
  items(key = "items"): unknown[] {
    return this.event === null ? [] : [...((this.event.data[key] as unknown[]) ?? [])];
  }
}

/** A deep copy that keeps every object's class, so a state's methods still
 * work on its snapshots, and an Item keeps its id. */
export function snapshotOf<T>(value: T, seen = new Map<unknown, unknown>()): T {
  if (value === null || typeof value !== "object") return value;
  if (seen.has(value)) return seen.get(value) as T;
  if (value instanceof Date) return new Date(value) as T;
  if (Array.isArray(value)) {
    const copy: unknown[] = [];
    seen.set(value, copy);
    value.forEach((v) => copy.push(snapshotOf(v, seen)));
    return copy as T;
  }
  if (value instanceof Map) {
    const copy = new Map();
    seen.set(value, copy);
    value.forEach((v, k) => copy.set(snapshotOf(k, seen), snapshotOf(v, seen)));
    return copy as T;
  }
  if (value instanceof Set) {
    const copy = new Set();
    seen.set(value, copy);
    value.forEach((v) => copy.add(snapshotOf(v, seen)));
    return copy as T;
  }
  const copy = Object.create(Object.getPrototypeOf(value));
  seen.set(value, copy);
  for (const key of Reflect.ownKeys(value)) {
    copy[key] = snapshotOf((value as any)[key], seen);
  }
  return copy;
}

/** Run `fn` over `state` and record what happened. `fn` gets the state and an
 * `emit`; call `emit` after changing the state, and a snapshot is kept.
 * `snapshot(state)` returns the part worth showing, if a deep copy of the
 * whole state is not what you want. */
export function trace<S>(
  fn: (state: S, emit: Emit) => void,
  state: S,
  options?: { snapshot?: (state: S) => S },
): Trace<S> {
  const capture = () => (options?.snapshot ? options.snapshot(state) : snapshotOf(state));
  const events: Event<S>[] = [];
  const initial = capture();
  fn(state, (name, data = {}) => {
    events.push({ name, state: capture(), data });
  });
  return { initial, events };
}
