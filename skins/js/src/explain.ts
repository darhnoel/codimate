// The Explanation: the moments, put together. As in Python (explanation.py),
// the view runs once per event, durations come from Timing, and a held opening
// and ending are segments that go nowhere. Everything per frame happens in the
// Engine.

import { Frame, type Event, type Trace } from "./trace.ts";
import type { Scene } from "./scene.ts";
import type { Rule } from "./surface.ts";
import { call, engine, filmFont, height, memory, width } from "./waist.ts";
import { ExplanationSurface } from "./surface.ts";

/** How long each event lasts, in seconds. */
export class Timing {
  readonly default: number;
  readonly events: Record<string, number>;
  readonly opening: number;
  readonly finalHold: number;

  constructor(options?: { default?: number; events?: Record<string, number>; opening?: number; finalHold?: number }) {
    this.default = options?.default ?? 0.6;
    this.events = options?.events ?? {};
    this.opening = options?.opening ?? 0.8;
    this.finalHold = options?.finalHold ?? 1.2;
  }

  /** How long `event` lasts: its own entry, or `default`. */
  forEvent(event: Event): number {
    return this.events[event.name] ?? this.default;
  }
}

export type View<S> = (frame: Frame<S>) => Scene;

/** A trace, a view and a timing, ready to play or render. Built by `explain`. */
export class Explanation<S = unknown> extends ExplanationSurface {
  readonly trace: Trace<S>;
  readonly timing: Timing;
  readonly motion: Rule[];
  readonly scenes: Scene[];
  readonly durations: number[];

  constructor(options: { trace: Trace<S>; view: View<S>; motion?: Rule[]; timing?: Timing }) {
    super();
    this.trace = options.trace;
    this.timing = options.timing ?? new Timing();
    this.motion = options.motion ?? [];
    const view = options.view;
    this.scenes = [view(new Frame(this.trace.initial, null))];
    for (const event of this.trace.events) this.scenes.push(view(new Frame(event.state, event)));
    this.durations = this.trace.events.map((e) => this.timing.forEvent(e));
    this.scenes.unshift(this.scenes[0]);
    this.durations.unshift(this.timing.opening);
    this.scenes.push(this.scenes[this.scenes.length - 1]);
    this.durations.push(this.timing.finalHold);
  }

  /** @internal This film as the Authoring Core reads it (timeline, chapters,
   * covered labels). */
  protected _film(): object {
    return {
      durations: this.durations,
      events: this.trace.events.map((e) => ({ name: e.name, chapter: e.data.chapter ?? null })),
      scenes: this.scenes.map((s) => s._payload()),
      cameras: this.scenes.map((s) => s._camera()),
      film_font: filmFont(),
    };
  }

  /** The film's length, in seconds. */
  get duration(): number {
    return this.durations.reduce((a, b) => a + b, 0);
  }

  /** The film's data: what a browser plays, written by the Authoring Core so
   * the same film from any language is the same bytes. */
  filmData(): string {
    return call("film", {
      width: width(),
      height: height(),
      durations: this.durations,
      scenes: this.scenes.map((s) => s._payload()),
      cameras: this.scenes.map((s) => s._camera()),
      rules: this.motion.map((rule) => rule._payload()),
    });
  }

  /** Save the film's data to `output` (Node). */
  async exportWeb(output: string): Promise<string> {
    const { mkdirSync, writeFileSync } = await import("node:fs");
    const { dirname, resolve } = await import("node:path");
    const path = resolve(output);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, this.filmData());
    return output;
  }

  /** The film, built once, to draw any moment of. */
  private built(): any {
    return new engine.Film(this.filmData());
  }

  /** Play the film on a `<canvas>`, looping. Returns a function that stops it. */
  play(canvas: HTMLCanvasElement): () => void {
    const film = this.built();
    canvas.width = film.width;
    canvas.height = film.height;
    const context = canvas.getContext("2d")!;
    let start = performance.now();
    let frame = 0;
    const tick = (now: number) => {
      const seconds = ((now - start) / 1000) % film.duration;
      const at = film.draw(seconds, 1.0);
      const pixels = new Uint8ClampedArray(memory().buffer, at, film.pixelsLength);
      context.putImageData(new ImageData(pixels, film.width, film.height), 0, 0);
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }

  /** Render the film to an MP4 at `output` (Node, with ffmpeg on PATH). */
  async render(output: string, options?: { fps?: number }): Promise<string> {
    const { spawn } = await import("node:child_process");
    const { mkdirSync } = await import("node:fs");
    const { dirname, resolve } = await import("node:path");
    const fps = options?.fps ?? 30;
    const film = this.built();
    const path = resolve(output);
    mkdirSync(dirname(path), { recursive: true });
    const ffmpeg = spawn("ffmpeg", [
      "-y", "-v", "error", "-f", "rawvideo", "-pix_fmt", "rgba",
      "-s", `${film.width}x${film.height}`, "-r", String(fps), "-i", "-",
      "-c:v", "libx264", "-pix_fmt", "yuv420p", path,
    ], { stdio: ["pipe", "inherit", "inherit"] });
    const done = new Promise<void>((ok, fail) =>
      ffmpeg.on("close", (code) => (code === 0 ? ok() : fail(new Error(`ffmpeg exited with ${code}`)))));
    const frames = Math.round(film.duration * fps);
    for (let i = 0; i < frames; i++) {
      const at = film.draw(i / fps, 1.0);
      const pixels = Buffer.from(new Uint8Array(memory().buffer, at, film.pixelsLength));
      if (!ffmpeg.stdin.write(pixels)) await new Promise((ok) => ffmpeg.stdin.once("drain", ok));
    }
    ffmpeg.stdin.end();
    await done;
    return output;
  }
}

/** Put a trace, a view, motion rules and timing together into a film. */
export function explain<S>(options: { trace: Trace<S>; view: View<S>; motion?: Rule[]; timing?: Timing }): Explanation<S> {
  return new Explanation(options);
}
