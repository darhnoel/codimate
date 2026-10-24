// Scene, Group and Handle: where an author draws. The drawing itself happens
// in the Authoring Core (the generated methods in surface.ts send it there);
// what is left here is keeping the shapes and naming their places.

import { GroupSurface, HandleSurface } from "./surface.ts";
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

  protected _frame(): Frame {
    return { path: this.path.map((p) => to.key(p).join("/")), ox: this.ox, oy: this.oy, w: this.w };
  }

  protected _build(op: string, data: object): Handle {
    const shape: ShapeData = call(op, data);
    if (this.scene.shapes.has(shape.item)) {
      throw new Error(`two shapes share the name '${shape.item}' — a name means exactly one thing`);
    }
    this.scene.shapes.set(shape.item, shape);
    return new Handle(this.scene.shapes, shape.item);
  }
}

/** The picture at one moment. No animation, no timing, no memory of the
 * frame before. Shapes are paired between moments by name. */
export class Scene extends Group {
  readonly shapes = new Map<string, ShapeData>();

  constructor() {
    super(null, [], 0, 0, width());
  }

  /** @internal The scene as the Engine takes it. */
  _payload(): ShapeData[] {
    return [...this.shapes.values()];
  }

  /** @internal Where the camera looks; none until focus() arrives. */
  _camera(): null {
    return null;
  }
}

/** What a shape call gives back: the chained way to say more about a shape. */
export class Handle extends HandleSurface {
  private readonly shapes: Map<string, ShapeData>;
  readonly name: string;

  constructor(shapes: Map<string, ShapeData>, name: string) {
    super();
    this.shapes = shapes;
    this.name = name;
  }

  protected _shape(): ShapeData {
    return this.shapes.get(this.name)!;
  }

  protected _change(op: string, data: object): void {
    this.shapes.set(this.name, call(op, data));
  }
}
