# Self-checks

Plain `assert` tests, no framework. One file per module of the package:

| file | covers |
|---|---|
| `test_layout.py` | the canvas, Slots, `row`, `column`, anchors |
| `test_scene.py` | Groups, names, and the shapes you can draw |
| `test_trace.py` | `emit`/`trace`, and Items that survive a snapshot |
| `test_explain.py` | motion rules, timing, `cm.ease` |
| `test_science.py` | the science kit: camera, spheres, orbits, and the marks |
| `test_index.py` | the compact index: written, read back, any beat in any order |
| `test_preview.py` | the Previewer: running a script without rendering it, and asking what is on screen |
| `test_docs.py` | every code block in the guide still runs |
| `test_doc_programs.py` | guide programs build scenes and snippets compile without exporting media |

Run the lot:

```bash
python skins/python/tests/run.py --all     # also runs the guide's programs, which render real video
python skins/python/tests/run.py           # skip that while working on the library
```

or one file on its own while you work on it:

```bash
python skins/python/tests/test_layout.py
```

They are also valid pytest modules if you prefer that. Everything except
`cm.ease` is pure Python and runs without the Rust extension built — the Engine
has its own tests in `crates/codimate-py/src/lib.rs`, run with `cargo test`.

The examples have their own checks in
[codimate-examples](https://github.com/darhnoel/codimate-examples): every example
builds, every formula typesets, and an opt-in render checks each is neither black nor
frozen. That repository is where the end-to-end rendering of every example lives. Here,
`test_docs.py` still renders the guide's complete programs, so the path from view to
video is covered.

The documentation checks discover every `python` block in the README and
top-level Markdown guides. The fast suite captures complete programs before
export and constructs their scenes; it also compiles usage fragments.
With `--all`, `test_docs.py` additionally exports complete programs in temporary
folders to check the rendering pipeline without overwriting existing videos.

A test's name should say what must be true, so a failure reads as a sentence:
`test_a_group_moves_as_one_thing`, not `test_group_2`.
