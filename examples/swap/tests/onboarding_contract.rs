use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).expect("file should be readable")
}

fn examples_cargo_tomls(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let entries = fs::read_dir(root.join("examples")).expect("examples dir should be readable");
    for entry in entries {
        let entry = entry.expect("entry should be readable");
        let cargo = entry.path().join("Cargo.toml");
        if cargo.exists() {
            out.push(cargo);
        }
    }
    out.sort();
    out
}

fn package_name_from_cargo_toml(path: &Path) -> Option<String> {
    let content = read(path);
    let mut in_package = false;

    for raw in content.lines() {
        let line = raw.trim();

        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }

        if in_package && line.starts_with("name") {
            let mut parts = line.splitn(2, '=');
            let _key = parts.next()?;
            let value = parts.next()?.trim();
            if let Some(name) = value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
                return Some(name.to_string());
            }
        }
    }

    None
}

fn workspace_packages(root: &Path) -> HashSet<String> {
    let mut out = HashSet::new();

    for dir in ["crates", "examples"] {
        let base = root.join(dir);
        let entries = fs::read_dir(&base).expect("workspace dir should be readable");
        for entry in entries {
            let entry = entry.expect("entry should be readable");
            let cargo = entry.path().join("Cargo.toml");
            if cargo.exists() {
                if let Some(name) = package_name_from_cargo_toml(&cargo) {
                    out.insert(name);
                }
            }
        }
    }

    out
}

fn cargo_run_packages_from_markdown(markdown: &str) -> Vec<String> {
    let mut out = Vec::new();

    for raw in markdown.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix("cargo run -p ") {
            if let Some(pkg) = rest.split_whitespace().next() {
                out.push(pkg.to_string());
            }
        }
    }

    out
}

#[test]
fn canonical_onboarding_doc_exists_and_sets_less_than_30_min_goal() {
    let root = repo_root();
    let workflow = root.join("docs/daily-workflow.md");
    assert!(workflow.exists(), "missing docs/daily-workflow.md");

    let text = read(workflow);
    assert!(
        text.contains("less than 30") && text.contains("minutes"),
        "daily workflow should state the onboarding success metric"
    );
}

#[test]
fn readme_points_to_canonical_workflow_and_two_stage_path() {
    let root = repo_root();
    let readme = read(root.join("README.md"));

    assert!(
        readme.contains("[Daily Workflow](docs/daily-workflow.md)"),
        "README should link canonical workflow"
    );
    assert!(
        readme.contains("circle-to-square"),
        "README should include first-win stage"
    );
    assert!(
        readme.contains("swap"),
        "README should include real-template stage"
    );
}

#[test]
fn canonical_template_split_exists_in_swap() {
    let root = repo_root();
    let swap_src = root.join("examples/swap/src");

    for name in [
        "state.rs",
        "algorithm.rs",
        "view.rs",
        "motion.rs",
        "timing.rs",
        "builder.rs",
        "lib.rs",
        "main.rs",
    ] {
        assert!(
            swap_src.join(name).exists(),
            "missing canonical file: {name}"
        );
    }
}

#[test]
fn swap_create_uses_canonical_builder_chain() {
    let root = repo_root();
    let lib = read(root.join("examples/swap/src/lib.rs"));

    for call in [
        ".state(",
        ".view(",
        ".algorithm(",
        ".motion(",
        ".timing(",
        ".build()",
    ] {
        assert!(lib.contains(call), "swap create() should include {call}");
    }
}

fn assert_markdown_cargo_run_packages_exist(
    markdown: &str,
    packages: &HashSet<String>,
    source_name: &str,
) {
    let referenced = cargo_run_packages_from_markdown(markdown);
    assert!(
        !referenced.is_empty(),
        "{source_name} should contain at least one cargo run command"
    );

    for pkg in referenced {
        assert!(
            packages.contains(&pkg),
            "{source_name} references unknown package: {pkg}"
        );
    }
}

#[test]
fn daily_workflow_cargo_run_commands_reference_real_workspace_packages() {
    let root = repo_root();
    let workflow = read(root.join("docs/daily-workflow.md"));
    let packages = workspace_packages(&root);

    assert_markdown_cargo_run_packages_exist(&workflow, &packages, "docs/daily-workflow.md");
}

#[test]
fn readme_cargo_run_commands_reference_real_workspace_packages() {
    let root = repo_root();
    let readme = read(root.join("README.md"));
    let packages = workspace_packages(&root);

    assert_markdown_cargo_run_packages_exist(&readme, &packages, "README.md");
}

#[test]
fn examples_depend_on_facade_not_split_beginner_crates() {
    let root = repo_root();

    for cargo in examples_cargo_tomls(&root) {
        let content = read(&cargo);
        let path = cargo.display();

        for forbidden in [
            "codimate-animation = {",
            "codimate-core = {",
            "codimate-export = {",
            "codimate-layout = {",
        ] {
            assert!(
                !content.contains(forbidden),
                "{path} should not directly depend on {forbidden}; use codimate facade"
            );
        }

        assert!(
            content.contains("codimate = {"),
            "{path} should depend on codimate facade"
        );
    }
}
