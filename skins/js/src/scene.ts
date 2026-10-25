// Scene, Group and Handle: where an author draws. The drawing itself happens
// in the Authoring Core (the generated methods in surface.ts send it there);
// what is left here is keeping the shapes and naming their places.

import { GroupSurface, HandleSurface, SceneSurface } from "./surface.ts";
import { call, to, width, type Frame, type Key, type ShapeData } from "./waist.ts";

/** Somewhere to draw, with its own origin and its own name. */
export class Group extends GroupSurface {
  protected readonly scene: Scene;
  private readonly path: Key[];
  private readonly ox: number;
  private readonly oy: number;
  private readonly w: number;

  constructor(scene: Scene | null, path: Key[], ox: number, oy: number, w: number) {
    super();
    this.scene = scene ?? (this as unknown as Scene);
    this.path = path;
    this.ox = ox;
    this.oy = oy;
    this.w = w;
  }

  private frameData: Frame | null = null;

  protected _frame(): Frame {
    return (this.frameData ??= { path: this.path.map((p) => to.key(p).join("/")), ox: this.ox, oy: this.oy, w: this.w });
  }

  protected _nest(frame: Frame): Group {
    const group = new Group(this.scene, frame.path, frame.ox, frame.oy, frame.w);
    group.frameData = frame;
    return group;
  }

  protected _build(op: string, data: object): Handle {
    return this.keep(call(op, data));
  }

  protected _buildCharacter(op: string, data: object): Character {
    const shape = call(op, data) as ShapeData;
    this.keep(shape);
    return new Character(this.scene.shapes, shape.item, this.scene.marks);
  }

  /** @internal Keep a marker (one shape and its stroke) or a highlight (band
   * and words, and their marks) the Core drew. */
  protected _keepMarked(answer: any): any {
    for (const [name, mark] of Object.entries(answer.marks ?? {})) this.scene.marks.set(name, mark);
    if ("shape" in answer) {
      const handle = this.keep(answer.shape);
      this.scene.marks.set(handle.name, answer.mark);
      return handle;
    }
    return this._adopt(answer.shapes);
  }

  /** @internal Keep several shapes the Authoring Core drew at once, in order. */
  _adopt(shapes: ShapeData[]): Handle[] {
    return shapes.map((shape) => this.keep(shape));
  }

  private keep(shape: ShapeData): Handle {
    if (this.scene.shapes.has(shape.item)) {
      throw new Error(`two shapes share the name '${shape.item}' — a name means exactly one thing`);
    }
    this.scene.shapes.set(shape.item, shape);
    return new Handle(this.scene.shapes, shape.item, this.scene.marks);
  }
}

/** The picture at one moment. No animation, no timing, no memory of the
 * frame before. Shapes are paired between moments by name. */
export class Scene extends SceneSurface(Group) {
  readonly shapes = new Map<string, ShapeData>();
  /** Marks the Core drew: a marker's stroke, a highlight's link to its band. */
  readonly marks = new Map<string, any>();
  private aim: Record<string, unknown> | null = null;
  private readonly overlays = new Set<string>();

  constructor() {
    super(null, [], 0, 0, width());
  }

  protected _aim(aim: Record<string, unknown>): void {
    this.aim = aim;
  }

  protected _openOverlay(frame: Frame): Group {
    this.overlays.add(frame.path[frame.path.length - 1]);
    return this._nest(frame);
  }

  /** @internal The scene as the Engine takes it. */
  _payload(): ShapeData[] {
    return [...this.shapes.values()];
  }

  /** @internal Where the camera looks, and what it leaves where it is. */
  _camera(): Record<string, unknown> | null {
    return this.aim === null ? null : { ...this.aim, fixed: [...this.overlays].sort() };
  }
}

/** What a shape call gives back: the chained way to say more about a shape. */
export class Handle extends HandleSurface {
  protected readonly shapes: Map<string, ShapeData>;
  protected readonly marks: Map<string, any>;
  readonly name: string;

  constructor(shapes: Map<string, ShapeData>, name: string, marks: Map<string, any> = new Map()) {
    super();
    this.shapes = shapes;
    this.marks = marks;
    this.name = name;
  }

  protected _shape(): ShapeData {
    return this.shapes.get(this.name)!;
  }

  /** Change this shape with the Authoring Core and keep the result. A marked
   * shape goes with its mark and its partner, and every shape and mark the
   * Core changes is kept. */
  protected _change(op: string, data: object): void {
    const mark = this.marks.get(this.name);
    if (mark === undefined) {
      this.shapes.set(this.name, call(op, data));
      return;
    }
    const partner = mark.band ?? mark.words;
    const names = [this.name, ...(partner && this.shapes.has(partner) ? [partner] : [])];
    const answer = call("marked", {
      op, name: this.name, data,
      shapes: Object.fromEntries(names.map((n) => [n, this.shapes.get(n)])),
      marks: Object.fromEntries(names.filter((n) => this.marks.has(n)).map((n) => [n, this.marks.get(n)])),
    });
    for (const [name, shape] of Object.entries(answer.shapes)) this.shapes.set(name, shape as ShapeData);
    for (const [name, value] of Object.entries(answer.marks)) this.marks.set(name, value);
  }
}

/** A connected, SVG-backed figure whose pose belongs to one Scene. */
export class Character extends Handle {
  /** Select a joint by the `id` of its marked SVG group. */
  part(name: string): CharacterPart {
    if (!name) throw new Error("a character part needs a non-empty SVG group id");
    return new CharacterPart(this.shapes, this.name, name);
  }

  /** Choose a named SVG expression variant for this Scene. */
  expression(name: string, options: { transition?: "crossfade" | "cut" } = {}): this {
    if (!name) throw new Error("a character expression needs a non-empty name");
    const transition = options.transition ?? "crossfade";
    if (transition !== "crossfade" && transition !== "cut") {
      throw new Error("expression transition must be 'crossfade' or 'cut'");
    }
    const pose = this.readPose();
    pose.expression = name;
    if (transition === "cut") pose.expression_transition = "cut";
    else delete pose.expression_transition;
    this.writePose(pose);
    return this;
  }

  private readPose(): CharacterPose {
    const encoded = this._shape().pose;
    if (typeof encoded !== "string" || encoded === "") return { expression: null, joints: {} };
    return JSON.parse(encoded) as CharacterPose;
  }

  private writePose(pose: CharacterPose): void {
    const joints = Object.fromEntries(Object.entries(pose.joints).sort(([a], [b]) => a.localeCompare(b)));
    this._shape().pose = JSON.stringify({
      expression: pose.expression,
      expression_transition: pose.expression_transition,
      joints,
    });
  }
}

/** A joint handle. Its descendants follow when its angle changes. */
export class CharacterPart {
  private readonly shapes: Map<string, ShapeData>;
  private readonly item: string;
  private readonly partName: string;

  constructor(shapes: Map<string, ShapeData>, item: string, partName: string) {
    this.shapes = shapes;
    this.item = item;
    this.partName = partName;
  }

  /** Set this joint's angle in degrees for the current Scene. */
  turn(degrees: number): this {
    if (!Number.isFinite(degrees)) throw new Error("a character joint angle must be finite");
    const shape = this.shapes.get(this.item)!;
    const encoded = typeof shape.pose === "string" && shape.pose !== ""
      ? JSON.parse(shape.pose) as CharacterPose
      : { expression: null, joints: {} };
    encoded.joints[this.partName] = degrees;
    const joints = Object.fromEntries(Object.entries(encoded.joints).sort(([a], [b]) => a.localeCompare(b)));
    shape.pose = JSON.stringify({
      expression: encoded.expression,
      expression_transition: encoded.expression_transition,
      joints,
    });
    return this;
  }
}

type CharacterPose = {
  expression: string | null;
  expression_transition?: "crossfade" | "cut";
  joints: Record<string, number>;
};
