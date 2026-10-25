"""Write every Skin's declarations, and the Core's dispatch table, from the
Interface Manifest (ADR 0031).

    python tools/generate.py           # write the generated files
    python tools/generate.py --check   # fail if any is out of date (CI)

One manifest, several outputs, so a name exists in every language or in none:

- crates/codimate-author/src/generated.rs   the Core's dispatch: an operation in
                                            the manifest with no Core function
                                            does not compile
- skins/python/codimate/_surface.py         the Python Skin's methods
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
            arg.setdefault("variadic", False)
    return ops


def py_doc(doc: str, indent: str) -> str:
    """A docstring at `indent`: raw if it holds a backslash (LaTeX), lines
    indented under the first."""
    lines = doc.strip("\n").splitlines()
    body = "\n".join([lines[0]] + [(indent + line) if line else "" for line in lines[1:]])
    raw = "r" if "\\" in doc else ""
    closing = f"\n{indent}" if len(lines) > 1 else ""
    return f'{indent}{raw}"""{body}{closing}"""'


def ts_doc(doc: str, indent: str) -> str:
    """A JSDoc comment at `indent`."""
    lines = doc.strip("\n").splitlines()
    if len(lines) == 1:
        return f"{indent}/** {lines[0]} */"
    inner = "\n".join(f"{indent} *" + (f" {line}" if line else "") for line in lines)
    return f"{indent}/**\n{inner}\n{indent} */"


# -- Rust: the Core's dispatch -------------------------------------------------

def rust(ops: list[dict]) -> str:
    # Each operation, and each second step a sampled method crosses (`then`).
    names = [(op["name"], op.get("module", "surface")) for op in ops if op["kind"] != "shortcut"]
    names += [(op["then"], op.get("module", "surface")) for op in ops if "then" in op]
    arms = "\n".join(f'        "{name}" => run(data, {module}::{name}),' for name, module in names)
    modules = ", ".join(sorted({op["module"] for op in ops if "module" in op}))
    return f"""// {HEADER}

use crate::surface::{{self, run}};
use crate::{{{modules}}};
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

PY_DEFAULT = {"none": "None", "true": "True", "false": "False", "origin": "(0.0, 0.0)"}
PY_CONTEXT = {
    "group": '"group": self._frame()',
    "shape": '"shape": self._shape()',
    "film_font": '"film_font": _film_font()',
    "canvas": '"canvas": _canvas()',
    "preset": '"preset": self._fields',
    "axes": '"axes": self._fields',
    "film": '**self._film()',
}
PY_RETURNS = {"points": "_as_points({})", "size": "_as_size({})", "number": "{}",
              "slots": "_paired(items, {})", "handles": "group._adopt({})",
              "rules": "tuple(Rule._of(made) for made in {})",
              "point": "_as_size({})", "ticks": "[tuple(tick) for tick in {}]",
              "rows": "[tuple(row) for row in {}]", "text": "{}"}


def py_param(arg: dict) -> str:
    if arg["variadic"]:
        return f'*{arg["name"]}'
    if "default" not in arg:
        return arg["name"]
    return f'{arg["name"]}={PY_DEFAULT.get(arg["default"], arg["default"])}'


def py_signature(args: list[dict], first: list[str]) -> str:
    named = [a for a in args if a["type"] != "options"]
    positional = [py_param(a) for a in named if a["positional"]]
    keywords = [py_param(a) for a in named if not a["positional"]]
    params = first + positional + (["*", *keywords] if keywords else [])
    if any(a["type"] == "options" for a in args):
        params.append("**options")
    return ", ".join(params)


def py_data(op: dict) -> str:
    data = [PY_CONTEXT[c] for c in op["context"]]
    data += [f'"{a["name"]}": ' + ("options" if a["type"] == "options" else
                                    f'_{a["type"]}({a["name"]})') for a in op["args"]
             if a["type"] != "function"]
    return "{" + ", ".join(data) + "}"


def py_value(op: dict, ops: list[dict]) -> str:
    """A value class: made by the Core, its fields set from the answer, with
    the methods the manifest gives it."""
    cls, types = op["class"], op.get("field_types", {})
    fields = []
    for f in op["fields"]:
        if f in types:
            fields.append(f'        self.{f} = None if made["{f}"] is None else '
                          f'{types[f]}._of(made["{f}"])')
        else:
            fields.append(f'        self.{f} = made["{f}"]')
    payload = ('        self._made = tuple(made["payload"])\n' if op.get("payload") else "")
    shown = " + \", \" + ".join(f'f"{f}={{self.{f}!r}}"' for f in op["fields"])
    methods = []
    for m in (o for o in ops if o["kind"] == "method" and o["on"] == cls):
        crossing = f'call("{m["name"]}", {py_data(m)})'
        if m["returns"] == "sampled":
            # The Core gives the samples, the author's function their values,
            # and the Core maps them: the function only runs here.
            f = next(a["name"] for a in m["args"] if a["type"] == "function")
            body = (f"        xs = {crossing}\n"
                    f'        return _as_points(call("{m["then"]}", {{{PY_CONTEXT["axes"]}, '
                    f'"xs": xs, "ys": [float({f}(x)) for x in xs]}}))\n')
        elif m["returns"] == "drawn":
            group = next(a["name"] for a in m["args"] if a["type"] == "group")
            body = f"        {group}._adopt({crossing})\n        return self\n"
        elif m["returns"] == "value":
            body = f'        return {m["returns_class"]}._of({crossing})\n'
        else:
            body = f'        return {PY_RETURNS[m["returns"]].format(crossing)}\n'
        methods.append(f'    def {m["method"]}({py_signature(m["args"], ["self"])}):\n'
                       f'{py_doc(m["doc"], "        ")}\n{body}')
    out = (f'class {cls}:\n{py_doc(op["doc"], "    ")}\n\n'
           f'    def __init__({py_signature(op["args"], ["self"])}) -> None:\n'
           f'        self._set(call("{op["name"]}", {py_data(op)}))\n\n'
           f'    @classmethod\n    def _of(cls, made: dict) -> "{cls}":\n'
           f'        """One the Authoring Core has already made."""\n'
           f'        value = cls.__new__(cls)\n        value._set(made)\n        return value\n\n'
           f'    def _set(self, made: dict) -> None:\n'
           f'        self._fields = {{k: v for k, v in made.items() if k != "payload"}}\n'
           f'{payload}' + "\n".join(fields) + "\n\n")
    if op.get("payload"):
        out += ('    def _payload(self) -> tuple:\n        """What the Engine takes."""\n'
                '        return self._made\n\n')
    out += (f'    def __repr__(self) -> str:\n        return "{cls}(" + {shown} + ")"\n')
    return out + "".join("\n" + m for m in methods)


def py_shortcut(op: dict, ops: list[dict]) -> str:
    """A function that makes a value with some of its arguments fixed."""
    made = next(o for o in ops if o.get("class") == op["makes"])
    given = []
    for a in made["args"]:
        if a["name"] in op["fixed"]:
            given.append(repr(op["fixed"][a["name"]]))
        elif a["positional"]:
            given.append(a["name"])
        else:
            given.append(f'{a["name"]}={a["name"]}')
    return (f'def {op["name"]}({py_signature(op["args"], [])}) -> "{op["makes"]}":\n'
            f'{py_doc(op["doc"], "    ")}\n    return {op["makes"]}({", ".join(given)})\n')


def python(ops: list[dict]) -> str:
    classes: dict[str, list[str]] = {}
    for op in (o for o in ops if o["on"] in ("Group", "Scene", "Handle")):
        positional = [py_param(a) for a in op["args"] if a["positional"]]
        keywords = [py_param(a) for a in op["args"] if not a["positional"]]
        variadic = any(a["variadic"] for a in op["args"])
        params = (["self", *positional]
                  + (([] if variadic else ["*"]) + keywords if keywords else []))
        data = [PY_CONTEXT[c] for c in op["context"]]
        data += [f'"{a["name"]}": _{a["type"]}({a["name"]})' for a in op["args"]]
        returns = op.get("returns") or ("Handle" if op["on"] == "Group" else "self")
        crossing = f'call("{op["name"]}", {{{", ".join(data)}}})'
        body = (f'        return self._build_character("{op["name"]}", {{{", ".join(data)}}})'
                if op.get("returns") == "character" else
                f'        return self._nest({crossing})'
                if op.get("returns") == "Group" else
                f'        self._aim({crossing})'
                if op.get("returns") == "aim" else
                f'        return self._open_overlay({crossing})'
                if op.get("returns") == "overlay" else
                f'        return self._keep_marked({crossing})'
                if op.get("returns") in ("marked", "highlight") else
                f'        return self._build("{op["name"]}", {{{", ".join(data)}}})'
                if op["on"] == "Group" else
                f'        self._change("{op["name"]}", {{{", ".join(data)}}})\n'
                f'        return self')
        annotation = {"self": op["on"], "aim": "None", "overlay": "Group", "marked": "Handle",
                      "character": "Character",
                      "highlight": "tuple[Handle, Handle]"}.get(returns, returns)
        classes.setdefault(op["on"], []).append(
            f'    def {op["name"]}({", ".join(params)}) -> "{annotation}":\n'
            f'{py_doc(op["doc"], "        ")}\n{body}\n')
    values = {op["class"] for op in ops if op["kind"] == "value"}
    for op in ops:
        if op["kind"] == "method" and op["on"] not in values:
            crossing = f'call("{op["name"]}", {py_data(op)})'
            body = f'        return {PY_RETURNS[op["returns"]].format(crossing)}\n'
            classes.setdefault(op["on"], []).append(
                f'    def {op["method"]}({py_signature(op["args"], ["self"])}):\n'
                f'{py_doc(op["doc"], "        ")}\n{body}')
    functions = [py_value(op, ops) for op in ops if op["kind"] == "value"]
    functions += [py_shortcut(op, ops) for op in ops if op["kind"] == "shortcut"]
    for op in (o for o in ops if o["on"] == "module" and o["kind"] == "query"):
        positional = [py_param(a) for a in op["args"] if a["positional"]]
        keywords = [py_param(a) for a in op["args"] if not a["positional"]]
        params = positional + (["*", *keywords] if keywords else [])
        data = [PY_CONTEXT[c] for c in op["context"]]
        data += [f'"{a["name"]}": _{a["type"]}({a["name"]})' for a in op["args"]]
        answer = PY_RETURNS[op["returns"]].format(f'call("{op["name"]}", {{{", ".join(data)}}})')
        # Items may be a generator: listed once, so counting them does not
        # use them up before they are paired with their Slots.
        listed = ("    items = items if isinstance(items, int) else list(items)\n"
                  if any(a["type"] == "items" for a in op["args"]) else "")
        functions.append(f'def {op["name"]}({", ".join(params)}):\n'
                         f'{py_doc(op["doc"], "    ")}\n{listed}    return {answer}\n')
    out = [f'"""{HEADER}\n\nThe Python Skin\'s side of the Interface Manifest: each method sends its\n'
           'arguments across the Waist and keeps what comes back."""\n',
           "from __future__ import annotations\n",
           "from typing import TYPE_CHECKING\n",
           "from ._waist import " + ", ".join(sorted(
               {"_film_font", "_canvas", "_as_points", "_as_size", "_paired", "call"}
               | {f"_{a['type']}" for op in ops for a in op["args"]
                  if a["type"] not in ("options", "function")
                  and op["kind"] != "shortcut"})) + "\n",
           "if TYPE_CHECKING:\n    from .scene import Character, Group, Handle\n"]
    for owner, methods in classes.items():
        out.append(f"\nclass _{owner}Surface:\n" + "\n".join(methods))
    out += ["\n" + f for f in functions]
    return "\n".join(out)


# -- TypeScript: the Skin's methods --------------------------------------------

TS_TYPE = {"key": "Key", "number": "number", "text": "unknown", "string": "string", "at": "At",
           "point": "Point", "points": "readonly Point[]", "radius": "number | Point",
           "pair": "readonly [number, number]", "bool": "boolean", "slot": "Slot",
           "int": "number", "items": "number | Iterable<unknown>",
           "style": "TextPresetStyle", "group": "Group", "keys": "readonly Key[]",
           "function": "(x: number) => number"}
TS_CONTEXT = {
    "group": "group: this._frame()",
    "shape": "shape: this._shape()",
    "film_font": "film_font: filmFont()",
    "canvas": "canvas: [width(), height()]",
    "preset": "preset: this._fields",
    "axes": "axes: this._fields",
    "film": "...this._film()",
}
TS_DEFAULT = {"origin": "[0, 0]"}
TS_RETURNS = {"points": ("[number, number][]", "{}"), "size": ("[number, number]", "{}"),
              "number": ("number", "{}"), "slots": ("any[]", "paired(items, {})"),
              "handles": ("Handle[]", "group._adopt({})"),
              "rules": ("Rule[]", "{}.map((made: any) => Rule._of(made))"),
              "point": ("[number, number]", "{}"), "ticks": ("[number, number, string][]", "{}"),
              "rows": ("any[][]", "{}"), "text": ("string", "{}")}


def typescript(ops: list[dict]) -> str:
    classes: dict[str, list[str]] = {}
    functions: list[str] = []
    for op in ops:
        if op["kind"] == "value":
            functions.append(ts_value(op, ops))
            continue
        if op["kind"] == "shortcut":
            functions.append(ts_shortcut(op, ops))
            continue
        if op["kind"] == "method":
            if op["on"] not in {o["class"] for o in ops if o["kind"] == "value"}:
                kind, answer = TS_RETURNS[op["returns"]]
                crossing = f'call("{op["name"]}", {ts_data(op)})'
                classes.setdefault(op["on"], []).append(
                    f'{ts_doc(op["doc"], "  ")}\n'
                    f'  {camel(op["method"])}(): {kind} {{\n    return {answer.format(crossing)};\n  }}\n')
            continue
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
                source = f"({source} ?? {TS_DEFAULT.get(default, default)})"
            data.append(f'{a["name"]}: to.{a["type"]}({source})')
        call = f'"{op["name"]}", {{ {", ".join(data)} }}'
        if op["on"] == "module":
            kind, answer = TS_RETURNS[op["returns"]]
            functions.append(
                f'{ts_doc(op["doc"], "")}\n'
                f'export function {camel(op["name"])}({", ".join(params)}): {kind} {{\n'
                f'  return {answer.format(f"call({call})")};\n}}\n')
            continue
        if op.get("returns") == "aim":
            body = f"    this._aim(call({call}));"
            returns = "void"
        elif op.get("returns") in ("marked", "highlight"):
            body = f"    return this._keepMarked(call({call}));"
            returns = "Handle" if op["returns"] == "marked" else "[Handle, Handle]"
        elif op.get("returns") == "character":
            body = f"    return this._buildCharacter({call});"
            returns = "Character"
        elif op.get("returns") == "overlay":
            body = f"    return this._openOverlay(call({call}));"
            returns = "Group"
        elif op.get("returns") == "Group":
            body = f"    return this._nest(call({call}));"
            returns = "Group"
        elif op["on"] == "Group":
            body = f"    return this._build({call});"
            returns = "Handle"
        else:
            body = f"    this._change({call});\n    return this;"
            returns = "this"
        classes.setdefault(op["on"], []).append(
            f'{ts_doc(op["doc"], "  ")}\n'
            f'  {camel(op["name"])}({", ".join(params)}): {returns} {{\n{body}\n  }}\n')
    out = [f"// {HEADER}\n//\n// The JavaScript Skin's side of the Interface Manifest: each method sends\n"
           "// its arguments across the Waist and keeps what comes back.\n",
           'import { filmFont, to, type At, type Frame, type Key, type Point, type ShapeData, type Slot } from "./waist.ts";\n'
           'import { call, height, paired, width } from "./waist.ts";\n'
           'import type { Character, Group, Handle } from "./scene.ts";\n']
    for owner, methods in classes.items():
        if owner == "Scene":
            # A Scene is a Group: its methods arrive as a mixin over Group.
            out.append("/** A Scene's own methods, mixed into a Group class. */\n"
                       "export function SceneSurface<B extends abstract new (...args: any[]) => any>"
                       "(Base: B) {\n  abstract class SceneMethods extends Base {\n"
                       "    protected abstract _frame(): Frame;\n"
                       "    protected abstract _aim(aim: unknown): void;\n"
                       "    protected abstract _openOverlay(frame: Frame): Group;\n\n"
                       + "\n".join("  " + m.replace("\n", "\n  ").rstrip() + "\n" for m in methods)
                       + "  }\n  return SceneMethods;\n}\n")
            continue
        if owner == "Explanation":
            out.append("export abstract class ExplanationSurface {\n"
                       "  protected abstract _film(): object;\n\n" + "\n".join(methods) + "}\n")
            continue
        hooks = ("  protected abstract _frame(): Frame;\n"
                 "  protected abstract _build(op: string, data: object): Handle;\n"
                 "  protected abstract _buildCharacter(op: string, data: object): Character;\n"
                 "  protected abstract _nest(frame: Frame): Group;\n"
                 "  protected abstract _keepMarked(answer: any): any;\n"
                 if owner == "Group" else
                 "  protected abstract _shape(): ShapeData;\n"
                 "  protected abstract _change(op: string, data: object): void;\n")
        out.append(f"export abstract class {owner}Surface {{\n{hooks}\n" + "\n".join(methods) + "}\n")
    out += functions
    return "\n".join(out)


def ts_params(args: list[dict]) -> tuple[list[str], list[dict]]:
    named = [a for a in args if a["type"] != "options"]
    positional = [a for a in named if a["positional"]]
    keywords = [a for a in named if not a["positional"]]
    params = [f'{camel(a["name"])}{"?" if "default" in a else ""}: {TS_TYPE[a["type"]]}'
              for a in positional]
    if keywords or any(a["type"] == "options" for a in args):
        fields = "; ".join(f'{camel(a["name"])}?: {TS_TYPE[a["type"]]}' for a in keywords)
        extra = (" & Record<string, unknown>" if any(a["type"] == "options" for a in args)
                 else "")
        params.append(f"options?: {{ {fields} }}{extra}")
    return params, keywords


def ts_data(op: dict) -> str:
    data = [TS_CONTEXT[c] for c in op["context"]]
    for a in op["args"]:
        if a["type"] == "function":
            continue
        if a["type"] == "options":
            data.append("options: rest")
            continue
        source = camel(a["name"])
        default = a.get("default")
        if default not in (None, "none"):
            source = f"({source} ?? {TS_DEFAULT.get(default, default)})"
        data.append(f'{a["name"]}: to.{a["type"]}({source})')
    return "{ " + ", ".join(data) + " }"


def ts_unpack(args: list[dict], keywords: list[dict]) -> str:
    names = [camel(a["name"]) for a in keywords]
    if any(a["type"] == "options" for a in args):
        names.append("...rest")
    return f'    const {{ {", ".join(names)} }} = options ?? {{}};\n' if names else ""


def ts_value(op: dict, ops: list[dict]) -> str:
    """A value class: made by the Core, its fields set from the answer, with
    the methods the manifest gives it."""
    cls, types = op["class"], op.get("field_types", {})
    params, keywords = ts_params(op["args"])
    declared = "\n".join(f"  {camel(f)}: any;" for f in op["fields"])
    assigned = []
    for f in op["fields"]:
        value = (f"made.{f} == null ? null : {types[f]}._of(made.{f})" if f in types
                 else f"made.{f}")
        assigned.append(f"    this.{camel(f)} = {value};")
    payload = "    this.made = made.payload;\n" if op.get("payload") else ""
    methods = []
    for m in (o for o in ops if o["kind"] == "method" and o["on"] == cls):
        m_params, m_keywords = ts_params(m["args"])
        crossing = f'call("{m["name"]}", {ts_data(m)})'
        if m["returns"] == "sampled":
            f = camel(next(a["name"] for a in m["args"] if a["type"] == "function"))
            kind = "[number, number][]"
            body = (f"    const xs: number[] = {crossing};\n"
                    f'    return call("{m["then"]}", {{ {TS_CONTEXT["axes"]}, xs, '
                    f"ys: xs.map((x) => Number({f}(x))) }});\n")
        elif m["returns"] == "drawn":
            group = camel(next(a["name"] for a in m["args"] if a["type"] == "group"))
            kind = "this"
            body = f"    {group}._adopt({crossing});\n    return this;\n"
        elif m["returns"] == "value":
            kind = m["returns_class"]
            body = f'    return {m["returns_class"]}._of({crossing});\n'
        else:
            kind, answer = TS_RETURNS[m["returns"]]
            body = f"    return {answer.format(crossing)};\n"
        methods.append(
            f'{ts_doc(m["doc"], "  ")}\n'
            f'  {camel(m["method"])}({", ".join(m_params)}): {kind} {{\n'
            f'{ts_unpack(m["args"], m_keywords)}{body}  }}\n')
    out = (f'{ts_doc(op["doc"], "")}\n'
           f'export class {cls} {{\n{declared}\n  _fields: any;\n  private made: unknown[] = [];\n\n'
           f'  constructor({", ".join(params)}) {{\n'
           f'{ts_unpack(op["args"], keywords)}'
           f'    this._set(call("{op["name"]}", {ts_data(op)}));\n  }}\n\n'
           f'  /** @internal One the Authoring Core has already made. */\n'
           f'  static _of(made: any): {cls} {{\n'
           f'    const value = Object.create({cls}.prototype);\n    value._set(made);\n'
           f'    return value;\n  }}\n\n'
           f'  private _set(made: any): void {{\n'
           f'    const {{ payload, ...fields }} = made;\n    this._fields = fields;\n'
           f'{payload}' + "\n".join(assigned) + "\n  }\n\n")
    if op.get("payload"):
        out += ('  /** @internal What the Engine takes. */\n'
                '  _payload(): unknown[] {\n    return this.made;\n  }\n\n')
    return out + "\n".join(methods) + "}\n"


def ts_shortcut(op: dict, ops: list[dict]) -> str:
    """A function that makes a value with some of its arguments fixed."""
    made = next(o for o in ops if o.get("class") == op["makes"])
    params, keywords = ts_params(op["args"])
    given, options = [], []
    for a in made["args"]:
        if a["name"] in op["fixed"]:
            given.append(f'"{op["fixed"][a["name"]]}"')
        elif a["positional"]:
            given.append(camel(a["name"]))
        else:
            options.append(camel(a["name"]))
    if options:
        given.append("options")
    return (f'{ts_doc(op["doc"], "")}\n'
            f'export function {camel(op["name"])}({", ".join(params)}): {op["makes"]} {{\n'
            f'  return new {op["makes"]}({", ".join(given)});\n}}\n')


OUTPUTS = {
    ROOT / "crates/codimate-author/src/generated.rs": rust,
    ROOT / "skins/python/codimate/_surface.py": python,
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
