# ADR 0034 — Fonts by family, with an open set built in

**Status:** Accepted — 2026-10-24. Extends ADR 0029 to Skins without a file system.

A browser has no disk and cannot read installed fonts, so a film cannot rely on a path or on a family the viewer's machine happens to have. Films name fonts by family, and each Skin resolves the name where it runs: a path or an installed family in Python and in Node, as today; fonts a page provides (`cm.fonts.add(family, url)`) in the browser. Codimate ships a small open set that every Skin can always resolve: DejaVu Sans Mono, still the default so existing films do not change, plus Inter for text, JetBrains Mono for code (the open replacement for Menlo, which is Apple's and cannot be redistributed) and Noto Sans JP for Japanese, all under open licences that allow redistribution. Fonts are no longer embedded in the WebAssembly module (most of its 6 MB); each loads only when a film uses it.

## Consequences

- A film made with the built-in set looks the same in every Skin and on every machine.
- Films that name Menlo or another installed family still render in Python and Node, but need the page to provide that font in the browser.
