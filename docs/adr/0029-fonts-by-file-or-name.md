# ADR 0029 — Fonts by file or installed name, without touching the Engine

**Status:** Accepted — 2026-10-18

## Context

Every font Codimate could draw was compiled into the Engine: `codimate-fonts`
embeds each `.ttf` in its `fonts/` folder at build time. Text had no font of
its own, so an author who wanted anything but DejaVu Sans Mono had to add a file
to the crate and rebuild Rust.

## Decision

A text names its font, and the Engine loads it at render time.

- **By file or by name.** A name with a `/` or `\`, or ending `.ttf`, `.otf`,
  `.ttc` or `.otc`, is a file. Anything else is an installed font, looked up
  with `fontdb` by family, or by face name with case, spaces and hyphens
  ignored, so `"Inter Bold"` finds `Inter-Bold`. The system is scanned once,
  and only when a name is asked for.
- **A film default and a per-text override.** `cm.font(...)` sets the font for
  every text built after it; `font=` on `scene.text` and `scene.highlight`
  overrides one. `""` is the built-in font, and the payload carries the name.
- **One file per face.** Bold and italic are their own file or name; there is
  no `weight=` to match against a family.
- **Fallback stays.** Characters the named font lacks are drawn in the embedded
  fonts, so Khmer and Chinese still show. A loaded font is used only by the
  text that asked for it; the built-in fallback order never changes.
- **Errors where they are made.** `cm.font` loads at once and raises
  `ValueError`; a bad `font=` fails the build naming the shape. Never a quiet
  fallback to the wrong font.
- **Measured in the same font.** `measure`, a highlight's band and the camera's
  framing all measure in the text's font.

Loaded fonts are kept for the life of the process, like formulas and pictures.
Shaping reads TrueType and CFF outlines alike, so `.otf` draws, and an installed
font inside a collection uses its own face index.

## Consequences

- An author changes fonts from Python alone.
- A film that names an installed font can look different on a machine without
  it, so the guide recommends a file kept with the project.
- `fontdb` is a new dependency of `codimate-fonts`.
- The default stays DejaVu Sans Mono, so every existing film renders as before.
