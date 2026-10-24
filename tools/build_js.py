"""Build the JavaScript Skin: the Rust crates as WebAssembly with the bindings
JavaScript loads them with (skins/js/pkg/), and the Skin's TypeScript as plain
JavaScript for browsers (skins/js/dist/). Node runs the TypeScript as it is.

    python tools/build_js.py

Needs the wasm32-unknown-unknown target (`rustup target add ...`) and the
wasm-bindgen CLI at the version in Cargo.lock.
"""

import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WASM = ROOT / "target/wasm32-unknown-unknown/release/codimate_waist.wasm"


def main() -> int:
    subprocess.run(["cargo", "build", "-q", "-p", "codimate-waist",
                    "--features", "wasm", "--release",
                    "--target", "wasm32-unknown-unknown"], cwd=ROOT, check=True)
    bindgen = (shutil.which("wasm-bindgen")
               or str(Path.home() / ".cargo/bin/wasm-bindgen"))
    subprocess.run([bindgen, "--target", "web", "--out-dir", str(ROOT / "skins/js/pkg"),
                    str(WASM)], check=True)
    # Types stripped by Node itself (`module.stripTypeScriptTypes`), so the
    # browser build needs no compiler; imports of ".ts" become ".js".
    strip = (
        "const fs=require('fs'),path=require('path'),m=require('module');"
        "const [src,out]=process.argv.slice(1);fs.mkdirSync(out,{recursive:true});"
        "for(const f of fs.readdirSync(src).filter(f=>f.endsWith('.ts'))){"
        "let js=m.stripTypeScriptTypes(fs.readFileSync(path.join(src,f),'utf8'));"
        "js=js.replace(/(from\\s+[\"'][^\"']+)\\.ts([\"'])/g,'$1.js$2');"
        "fs.writeFileSync(path.join(out,f.replace(/\\.ts$/,'.js')),js)}"
    )
    subprocess.run(["node", "-e", strip, str(ROOT / "skins/js/src"),
                    str(ROOT / "skins/js/dist")], check=True)
    print("built skins/js/pkg and skins/js/dist")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
