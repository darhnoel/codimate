"""Write every Skin's declarations, and the Core's dispatch table, from the
Interface Manifest (ADR 0031).

    python tools/generate.py           # write the generated files
    python tools/generate.py --check   # fail if any is out of date (CI)

One manifest, several outputs, so a name exists in every language or in none:

- crates/codimate-author/src/generated.rs   the Core's dispatch: an operation in
                                            the manifest with no Core function
                                            does not compile
- python/codimate/_surface.py               the Python Skin's methods
- skins/js/src/surface.ts                   the JavaScript Skin's methods

Python names are the manifest's; JavaScript names are the same in camelCase.
Positional arguments come first in both; the rest are keyword-only in Python
and one options object in JavaScript.
"""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "api" / "surface.toml"
HEADER = "Generated from api/surface.toml by tools/generate.py. Do not edit."


def camel(name: str) -> str:
    """The one naming rule: snake_case in, camelCase out."""
    head, *rest = name.split("_")
    return head + "".join(part.title() for part in rest)


def load() -> list[dict]:
    ops = tomllib.loads(MANIFEST.read_text())["op"]
    for op in ops:
        for arg in op["args"]:
            arg.setdefault("positional", False)
    return ops


# -- Rust: the Core's dispatch -------------------------------------------------

def rust(ops: list[dict]) -> str:
    arms = "\n".join(f'        "{op["name"]}" => run(data, surface::{op["name"]}),'
                     for op in ops)
    return f"""// {HEADER}

use crate::surface::{{self, run}};
use crate::Refusal;

/// Every operation the manifest names, to its Core function; anything else
/// is one of the Core's own operations, which Skins use but authors never see.
pub fn dispatch(op: &str, data: &str) -> Result<String, Refusal> {{
    match op {{
{arms}
        _ => surface::internal(op, data),
    }}
}}
"""


# -- Python: the Skin's methods ------------------------------------------------

PY_DEFAULT = {"none": "None", "true": "True", "false": "False"}
PY_CONTEXT = {
    "group": '"group": self._frame()',
    "shape": '"shape": self._shape()',
    "film_font": '"film_font": _film_font()',
}


def py_param(arg: dict) -> str:
    if "default" not in arg:
        return arg["name"]
    return f'{arg["name"]}={PY_DEFAULT.get(arg["default"], arg["default"])}'


def python(ops: list[dict]) -> str:
    classes: dict[str, list[str]] = {}
    for op in ops:
        positional = [py_param(a) for a in op["args"] if a["positional"]]
        keywords = [py_param(a) for a in op["args"] if not a["positional"]]
        params = ["self", *positional] + (["*", *keywords] if keywords else [])
        data = [PY_CONTEXT[c] for c in op["context"]]
        data += [f'"{a["name"]}": _{a["type"]}({a["name"]})' for a in op["args"]]
        returns = "Handle" if op["on"] == "Group" else "self"
        body = (f'        return self._build("{op["name"]}", {{{", ".join(data)}}})'
                if op["on"] == "Group" else
                f'        self._change("{op["name"]}", {{{", ".join(data)}}})\n'
                f'        return self')
        classes.setdefault(op["on"], []).append(
            f'    def {op["name"]}({", ".join(params)}) -> "{returns if returns != "self" else op["on"]}":\n'
            f'        """{op["doc"]}"""\n{body}\n')
    out = [f'"""{HEADER}\n\nThe Python Skin\'s side of the Interface Manifest: each method sends its\n'
           'arguments across the Waist and keeps what comes back."""\n',
           "from __future__ import annotations\n",
           "from typing import TYPE_CHECKING\n",
           "from ._waist import " + ", ".join(sorted(
               {"_film_font"} | {f"_{a['type']}" for op in ops for a in op["args"]})) + "\n",
           "if TYPE_CHECKING:\n    from .scene import Handle\n"]
    for owner, methods in classes.items():
        out.append(f"\nclass _{owner}Surface:\n" + "\n".join(methods))
    return "\n".join(out)


# -- TypeScript: the Skin's methods --------------------------------------------

TS_TYPE = {"key": "Key", "number": "number", "text": "unknown", "string": "string", "at": "At",
           "point": "Point", "points": "readonly Point[]", "radius": "number | Point",
           "pair": "readonly [number, number]", "bool": "boolean"}
TS_CONTEXT = {
    "group": "group: this._frame()",
    "shape": "shape: this._shape()",
    "film_font": "film_font: filmFont()",
}


def typescript(ops: list[dict]) -> str:
    classes: dict[str, list[str]] = {}
    for op in ops:
        positional = [a for a in op["args"] if a["positional"]]
        options = [a for a in op["args"] if not a["positional"]]
        params = []
        for a in positional:
            mark = "?" if "default" in a else ""
            params.append(f'{camel(a["name"])}{mark}: {TS_TYPE[a["type"]]}')
        if options:
            fields = "; ".join(f'{camel(a["name"])}{"?" if "default" in a else ""}: '
                               f'{TS_TYPE[a["type"]]}' for a in options)
            required = any("default" not in a for a in options)
            params.append(f'options{"" if required else "?"}: {{ {fields} }}')
        data = [TS_CONTEXT[c] for c in op["context"]]
        for a in op["args"]:
            source = camel(a["name"]) if a["positional"] else f'options?.{camel(a["name"])}'
            default = a.get("default")
            if default not in (None, "none"):
                source = f"({source} ?? {default})"
            data.append(f'{a["name"]}: to.{a["type"]}({source})')
        call = f'"{op["name"]}", {{ {", ".join(data)} }}'
        if op["on"] == "Group":
            body = f"    return this._build({call});"
            returns = "Handle"
        else:
            body = f"    this._change({call});\n    return this;"
            returns = "this"
        classes.setdefault(op["on"], []).append(
            f'  /** {op["doc"]} */\n'
            f'  {camel(op["name"])}({", ".join(params)}): {returns} {{\n{body}\n  }}\n')
    out = [f"// {HEADER}\n//\n// The JavaScript Skin's side of the Interface Manifest: each method sends\n"
           "// its arguments across the Waist and keeps what comes back.\n",
           'import { filmFont, to, type At, type Frame, type Key, type Point, type ShapeData } from "./waist.ts";\n'
           'import type { Handle } from "./scene.ts";\n']
    for owner, methods in classes.items():
        hooks = ("  protected abstract _frame(): Frame;\n"
                 "  protected abstract _build(op: string, data: object): Handle;\n"
                 if owner == "Group" else
                 "  protected abstract _shape(): ShapeData;\n"
                 "  protected abstract _change(op: string, data: object): void;\n")
        out.append(f"export abstract class {owner}Surface {{\n{hooks}\n" + "\n".join(methods) + "}\n")
    return "\n".join(out)


OUTPUTS = {
    ROOT / "crates/codimate-author/src/generated.rs": rust,
    ROOT / "python/codimate/_surface.py": python,
    ROOT / "skins/js/src/surface.ts": typescript,
}


def rustfmt(text: str) -> str:
    """Format generated Rust the way `cargo fmt` would, so formatting the
    workspace never makes the generated file look out of date."""
    import subprocess

    done = subprocess.run(["rustfmt", "--edition", "2021", "--emit", "stdout"],
                          input=text, capture_output=True, text=True, check=True)
    return done.stdout


def main(argv: list[str]) -> int:
    ops = load()
    stale = []
    for path, write in OUTPUTS.items():
        text = write(ops)
        if path.suffix == ".rs":
            text = rustfmt(text)
        if "--check" in argv:
            if not path.exists() or path.read_text() != text:
                stale.append(path.relative_to(ROOT))
            continue
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        print(f"wrote {path.relative_to(ROOT)}")
    if stale:
        print("out of date — run python tools/generate.py:", *stale, sep="\n  ")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
