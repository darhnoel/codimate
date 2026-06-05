use std::{env, fs, path::PathBuf};

fn main() {
    let ws = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent().unwrap()
        .parent().unwrap()
        .to_path_buf();
    let manifest_path = ws.join("examples.manifest.toml");
    let manifest = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("failed to read {:?}: {e}", manifest_path));

    let out = env::var("OUT_DIR").unwrap();
    let dest = PathBuf::from(out).join("examples.rs");

    let toml: toml::Value = manifest.parse().unwrap();
    let examples = toml["example"].as_array().unwrap();

    let mut code = String::from("fn examples() -> Vec<Example> {\n    vec![\n");

    for ex in examples {
        let name = ex["name"].as_str().unwrap();
        let crate_name = ex["crate"].as_str().unwrap();
        let desc = ex["desc"].as_str().unwrap();
        let w = ex["w"].as_float().unwrap();
        let h = ex["h"].as_float().unwrap();

        code.push_str(&format!(
            "        Example {{ name: {name:?}, desc: {desc:?}, w: {w:.1}f32, h: {h:.1}f32, build: {crate_name}::create }},\n",
        ));
    }

    code.push_str("    ]\n}\n");

    fs::write(&dest, &code).unwrap_or_else(|e| panic!("failed to write {dest:?}: {e}"));
}
