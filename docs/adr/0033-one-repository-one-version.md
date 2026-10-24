# ADR 0033 — One repository, one version for every Skin

**Status:** Accepted — 2026-10-24

The Interface Manifest, the Authoring Core, the Waist, every Skin and every Conformance Example live in this one repository (`api/`, `crates/`, `skins/<language>/`, `conformance/`), so a feature is one change that touches the Manifest, the Core, its Conformance Example and every regenerated Skin together. The Python package moves from `python/` to `skins/python/`; its published name, `codimate`, does not change. Every Skin is released at the same version as the Core, together, by one workflow that publishes only when every Conformance Example passes in every language: `codimate` 0.5.0 on PyPI and on npm are the same Core and the same interface. A fix in one Skin is therefore a release of all of them.

## Considered Options

- **A repository per Skin, versioned on its own.** Each could release at its own pace, which is exactly how one falls behind the other.
