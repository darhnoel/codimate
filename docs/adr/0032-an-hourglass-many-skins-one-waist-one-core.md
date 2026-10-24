# ADR 0032 — An hourglass: many Skins, one Waist, one Core

**Status:** Accepted — 2026-10-24. Refines ADR 0030: Skins reach the Authoring Core through one narrow boundary, not by binding each Core function.

**Revised 2026-10-24, before any code:** a Skin does not batch a Scene's calls. Authors use results at once (a Slot's edges, a measurement, a handle's name), and batching would mean carrying each call's file and line just to report an error. The Waist is one function, `call(op, data) → data`: every Builder call and Query crosses it as it happens, so an error is raised on the author's own line with nothing extra. Batching returns only if the recording benchmark asks for it. What follows describes the original batching design; read "records … and sends each Scene's list" as "sends each call".

Codimate will have more Skins than Python and JavaScript (C++ and others later). If each language bound each Core function, every new language would cost a binding per public function and every new feature a binding per language: functions × languages of glue. Instead the Core sits behind a **Waist** of about three functions that take and return plain data: build a Scene from a list of recorded calls, answer a query, build a film. A Skin, generated from the Interface Manifest (ADR 0031), records the author's builder calls as data (`["rect", "box", {...}]`, `["fill", "#fff"]`) and sends each Scene's list across the Waist in one call. The few calls an author needs answered at once (`measure`, Slots, `ease`) are marked as **queries** in the Manifest and cross individually. JavaScript reaches the Waist through WebAssembly; every native language, Python included in time, through one C interface.

Adding a language is then a generator template, a transport and a Conformance runner, with no change to the Core; adding a feature is a Manifest entry and a Core implementation, with no change to any Skin.

## Consequences

- Every recorded call carries where the author made it (file and line), so a Core error names the author's own line, not the Waist.
- A benchmark on a large film guards the cost of crossing the Waist. Measured when slice 1 landed (C lesson 06, 729 Scenes, about 41,000 calls): building Scenes went from about 1.0 s to 1.2 s, 17–23% slower, almost all of it Python turning shapes into JSON text and back; the Rust side is 68 ms. Accepted with a budget of 25%. The old Surface is deleted as slices move, so CI cannot compare against it: it guards one crossing instead (`tools/bench_waist.py`, about 10 µs a crossing on a laptop; it fails above 10 times a plain-Python JSON round trip of the same call, about 6 on that laptop, because shared CI machines differed by half from one run to the next in absolute microseconds), which catches what regresses this (a second parse, an import per call, a deep copy). The overhead shrinks as slices move the features that still read shapes in Python (markers, the covered-label check, the index, the Previewer) into the Core; when the last has moved, Scenes stay in the Core from building to rendering and the guard tightens to "no slower than before".
- Queries stay few and visible in the Manifest; each one is a round trip.

## Same bytes everywhere

Two rules found by the Conformance Examples, both needed for one film to be the same bytes from every language on every platform:

- **Numbers cross exactly.** The Core parses JSON with `serde_json`'s `float_roundtrip`; its default parser can be off by one bit for some decimals, which nudged values crossing the Waist (a marker's points did).
- **One portable libm.** The Core's `sin`, `cos` and `hypot` come from the `libm` crate on every target, not the operating system's (native) or musl's (WebAssembly), which disagree in the last bit for some inputs. Values recorded from the old Python Surface, which used the platform's, are compared to within a few units in the last place.

## Considered Options

- **Binding each Core function per language (PyO3, wasm-bindgen, …).** Simplest for two languages; the glue grows with every function and every language.
