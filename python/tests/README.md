# Self-checks

Plain `assert` tests, no framework. One file per module of the package:

| file | covers |
|---|---|
| `test_layout.py` | the canvas, Slots, `row`, `column`, anchors |
| `test_scene.py` | Groups, names, and the shapes you can draw |
| `test_trace.py` | `emit`/`trace`, and Items that survive a snapshot |
| `test_explain.py` | motion rules, timing, `cm.ease` |

Run the lot:

```bash
python python/tests/run.py
```

or one file on its own while you work on it:

```bash
python python/tests/test_layout.py
```

They are also valid pytest modules if you prefer that. Everything except
`cm.ease` is pure Python and runs without the Rust extension built — the Engine
has its own tests in `crates/codimate-py/src/lib.rs`, run with `cargo test`.

A test's name should say what must be true, so a failure reads as a sentence:
`test_a_group_moves_as_one_thing`, not `test_group_2`.
