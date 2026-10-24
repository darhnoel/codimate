# Vendored libraries

The Previewer's page has no build step, so its libraries live here as the
plain ES modules npm publishes, and `../index.html` maps their names to these
files. All MIT licensed.

| file | package | from |
| --- | --- | --- |
| `preact.js` | preact 10.29.8 | `dist/preact.module.js` |
| `hooks.js` | preact 10.29.8 | `hooks/dist/hooks.module.js` |
| `htm.js` | htm 3.1.1 | `dist/htm.module.js` |
| `signals-core.js` | @preact/signals-core 1.14.4 | `dist/signals-core.module.js` |
| `signals.js` | @preact/signals 2.11.2 | `dist/signals.module.js` |

Each is unmodified except for a first-line comment naming it, and the removal
of its `sourceMappingURL` comment (the maps are not vendored, so a browser
would only log a 404 for each).

To update one, fetch the same path at the new version, for example

    curl -o preact.js https://cdn.jsdelivr.net/npm/preact@<version>/dist/preact.module.js

then repeat those two edits and change the version here. Keep preact and
hooks on the same version.
