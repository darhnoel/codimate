use codimate_animation::Playable;
use codimate_layout::Viewport;
use codimate_previewer::{PreviewConfig, Previewer};
use std::env;

struct Example {
    name: &'static str,
    desc: &'static str,
    w: f32,
    h: f32,
    build: fn() -> (Box<dyn Playable>, Viewport),
}

include!(concat!(env!("OUT_DIR"), "/examples.rs"));

fn print_help(binary_name: &str) {
    eprintln!("Usage: {binary_name} [OPTIONS]");
    eprintln!();
    eprintln!("Preview an animation example in an interactive window.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -e, --example NAME   Example to preview");
    eprintln!("  -l, --list           List available examples and exit");
    eprintln!("  -h, --help           Print this help and exit");
    eprintln!();
    eprintln!("If no example is given, the list is shown.");
    eprintln!();
    eprintln!("Controls:  Space=play/pause  ←→=step  C=screenshot  Q/Esc=quit");
}

fn list_examples() {
    let exs = examples();
    eprintln!("Available examples:");
    let max_len = exs.iter().map(|e| e.name.len()).max().unwrap_or(0);
    for ex in &exs {
        eprintln!(
            "  {name:>width$}  {desc}  ({w}×{h})",
            name = ex.name,
            width = max_len,
            desc = ex.desc,
            w = ex.w as u16,
            h = ex.h as u16,
        );
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let binary_name = args
        .first()
        .map(|s| s.as_str())
        .unwrap_or("codimate-previewer");

    let mut example_name = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help(binary_name);
                return;
            }
            "-l" | "--list" => {
                list_examples();
                return;
            }
            "-e" | "--example" => {
                i += 1;
                if i < args.len() {
                    example_name = Some(&args[i]);
                } else {
                    eprintln!("error: --example requires a name");
                    std::process::exit(1);
                }
            }
            _ if !args[i].starts_with('-') && example_name.is_none() => {
                example_name = Some(&args[i]);
            }
            _ => {
                eprintln!("error: unknown flag {}", args[i]);
                eprintln!("Try '{binary_name} --help' for usage.");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let exs = examples();
    let name = match example_name {
        Some(n) => n,
        None => {
            list_examples();
            return;
        }
    };

    let example = exs.iter().find(|e| e.name == name).unwrap_or_else(|| {
        eprintln!("error: unknown example '{name}'");
        eprintln!();
        list_examples();
        std::process::exit(1);
    });

    let (playable, viewport) = (example.build)();
    let config =
        PreviewConfig::new(30.0, viewport).title(format!("Codimate Preview — {}", example.name));
    let previewer = Previewer::new(playable, config);
    previewer.run().unwrap();
}
