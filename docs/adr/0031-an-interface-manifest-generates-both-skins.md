# ADR 0031 — An Interface Manifest generates both Skins

**Status:** Accepted — 2026-10-24

With one Authoring Core (ADR 0030), drift can still happen at the Skins: each declares the public names, arguments and defaults, and one could gain an argument the other lacks. So the Skins' declarations are not written by hand. One **Interface Manifest** lists every public function, argument, type and default; a generator writes the Python signatures and the TypeScript types from it, converting snake_case to camelCase by one fixed rule. Behaviour is pinned separately by **Conformance Examples**: the same small explanation in both languages must produce identical film data, and a new feature begins as one that fails in both.

The Manifest is its own language-neutral file (`api/surface.toml`), belonging to neither Skin, so neither language is the original and a change can start from either side. From it the generator writes the Python signatures and stubs, the TypeScript functions and `.d.ts` types, a Rust trait the Core must implement (an entry without a Core implementation does not compile), and the API reference page.

## Considered Options

- **Attributes on the Core's Rust signatures.** No separate file, but Python defaults, JavaScript option shapes and docs end up in Rust macros, and only Rust developers could change the interface.
- **Python as the source, TypeScript generated from it.** Python becomes the real one and JavaScript the copy: the opposite of changes flowing both ways.
- **Hand-written Skins checked by a parity test.** Catches drift after the fact instead of making it impossible; kept as the fallback if generating Python's keyword arguments proves awkward.
- **Conformance Examples alone.** Catch behaviour drift, but not an argument no example happens to use.
