use codimate_animation::{animation, sequence};
use codimate_core::{
    circle, path_node, scene, tween, Animated, Color, IntoAnimated, Path, Scene, Segment, Vec2,
};
use codimate_export::{export_mp4, ExportConfig};
use codimate_layout::Viewport;

// --- helpers ---

fn edge_path(x1: f32, y1: f32, x2: f32, y2: f32, th: f32) -> Path {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let px = -dy / len * th / 2.0;
    let py = dx / len * th / 2.0;
    Path {
        segments: vec![
            Segment::Line(Vec2::new(x1 + px, y1 + py), Vec2::new(x1 - px, y1 - py)),
            Segment::Line(Vec2::new(x1 - px, y1 - py), Vec2::new(x2 - px, y2 - py)),
            Segment::Line(Vec2::new(x2 - px, y2 - py), Vec2::new(x2 + px, y2 + py)),
            Segment::Line(Vec2::new(x2 + px, y2 + py), Vec2::new(x1 + px, y1 + py)),
        ],
        closed: true,
    }
}

// --- colors ---

const DIM: Color = Color {
    r: 0.15,
    g: 0.15,
    b: 0.15,
    a: 1.0,
};

const NEURON_ON: Color = Color {
    r: 0.95,
    g: 0.15,
    b: 0.15,
    a: 1.0,
};

const EDGE_DIM: Color = Color {
    r: 0.08,
    g: 0.08,
    b: 0.08,
    a: 1.0,
};

const EDGE_FIRE: Color = Color {
    r: 0.9,
    g: 0.2,
    b: 0.1,
    a: 1.0,
};

// --- network layout ---

struct Net {
    inputs: [(f32, f32); 3],
    hiddens: [(f32, f32); 4],
    outputs: [(f32, f32); 2],
    in_hidden: [[(f32, f32, f32, f32); 4]; 3],
    hidden_out: [[(f32, f32, f32, f32); 2]; 4],
}

const N: f32 = 220.0;
const NW: f32 = 100.0;

fn network() -> Net {
    let inputs = [(NW, 300.0 - N), (NW, 300.0), (NW, 300.0 + N)];
    let hiddens = [
        (400.0, 300.0 - N * 0.75),
        (400.0, 300.0 - N * 0.25),
        (400.0, 300.0 + N * 0.25),
        (400.0, 300.0 + N * 0.75),
    ];
    let outputs = [(700.0, 300.0 - N / 2.0), (700.0, 300.0 + N / 2.0)];

    let mut in_hidden: [[(f32, f32, f32, f32); 4]; 3] = [[(0.0, 0.0, 0.0, 0.0); 4]; 3];
    for (i, &(ix, iy)) in inputs.iter().enumerate() {
        for (h, &(hx, hy)) in hiddens.iter().enumerate() {
            unsafe {
                *in_hidden.get_unchecked_mut(i).get_unchecked_mut(h) = (ix, iy, hx, hy);
            }
        }
    }

    let mut hidden_out: [[(f32, f32, f32, f32); 2]; 4] = [[(0.0, 0.0, 0.0, 0.0); 2]; 4];
    for (h, &(hx, hy)) in hiddens.iter().enumerate() {
        for (o, &(ox, oy)) in outputs.iter().enumerate() {
            unsafe {
                *hidden_out.get_unchecked_mut(h).get_unchecked_mut(o) = (hx, hy, ox, oy);
            }
        }
    }

    Net {
        inputs,
        hiddens,
        outputs,
        in_hidden,
        hidden_out,
    }
}

// Build a scene for one phase.
// Each element's `fill` is either a constant (dim), constant (bright),
// or `tween(dim, bright)` depending on the phase.

/// `Phase` describes which group transitions in this phase.
/// Elements from earlier phases hold their bright state;
/// elements from later phases stay dim.
enum Phase {
    /// t=0..1: inputs brighten.  Everything else dim.
    Inputs,
    /// inputs=bright, in→hidden edges + hidden brighten. outputs=dim.
    Hidden,
    /// inputs+hidden=bright, hidden→out edges + outputs brighten.
    Outputs,
    /// Everything bright (hold).
    Done,
}

fn phase_scene(phase: Phase) -> Scene {
    let net = network();

    let in_state = match phase {
        Phase::Inputs => Fill::Tween,
        Phase::Hidden | Phase::Outputs | Phase::Done => Fill::Bright,
    };

    let hidden_state = match phase {
        Phase::Inputs => Fill::Dim,
        Phase::Hidden => Fill::Tween,
        Phase::Outputs | Phase::Done => Fill::Bright,
    };

    let out_state = match phase {
        Phase::Inputs | Phase::Hidden => Fill::Dim,
        Phase::Outputs => Fill::Tween,
        Phase::Done => Fill::Bright,
    };

    let in_edge_state = match phase {
        Phase::Inputs => Fill::Dim,
        Phase::Hidden => Fill::Tween,
        Phase::Outputs | Phase::Done => Fill::Bright,
    };

    let out_edge_state = match phase {
        Phase::Inputs | Phase::Hidden => Fill::Dim,
        Phase::Outputs => Fill::Tween,
        Phase::Done => Fill::Bright,
    };

    let mut sc = scene();

    // Edges: input -> hidden
    for row in &net.in_hidden {
        for &(x1, y1, x2, y2) in row {
            sc = sc.node(
                path_node()
                    .path(edge_path(x1, y1, x2, y2, 3.0))
                    .fill(fill_for(in_edge_state, EDGE_DIM, EDGE_FIRE)),
            );
        }
    }

    // Edges: hidden -> output
    for row in &net.hidden_out {
        for &(x1, y1, x2, y2) in row {
            sc = sc.node(
                path_node()
                    .path(edge_path(x1, y1, x2, y2, 3.0))
                    .fill(fill_for(out_edge_state, EDGE_DIM, EDGE_FIRE)),
            );
        }
    }

    // Input neurons
    for &(x, y) in &net.inputs {
        sc = sc.node(
            circle()
                .x(x)
                .y(y)
                .radius(24.0)
                .fill(fill_for(in_state, DIM, NEURON_ON)),
        );
    }

    // Hidden neurons
    for &(x, y) in &net.hiddens {
        sc = sc.node(
            circle()
                .x(x)
                .y(y)
                .radius(24.0)
                .fill(fill_for(hidden_state, DIM, NEURON_ON)),
        );
    }

    // Output neurons
    for &(x, y) in &net.outputs {
        sc = sc.node(
            circle()
                .x(x)
                .y(y)
                .radius(24.0)
                .fill(fill_for(out_state, DIM, NEURON_ON)),
        );
    }

    sc
}

#[derive(Clone, Copy)]
enum Fill {
    Dim,
    Bright,
    Tween,
}

fn fill_for(f: Fill, dim: Color, bright: Color) -> Animated<Color> {
    match f {
        Fill::Dim => dim.into_animated(),
        Fill::Bright => bright.into_animated(),
        Fill::Tween => tween(dim, bright),
    }
}

fn main() {
    let phases = [
        animation("inputs", 1.0, phase_scene(Phase::Inputs)),
        animation("hidden", 1.0, phase_scene(Phase::Hidden)),
        animation("outputs", 1.0, phase_scene(Phase::Outputs)),
        animation("done", 1.0, phase_scene(Phase::Done)),
    ];
    let timeline = sequence("neural-net", phases);

    let viewport = Viewport::new(800.0, 600.0);
    let config = ExportConfig::new(30.0, viewport);

    std::fs::create_dir_all("results").ok();
    println!("Exporting results/neural-net.mp4 …");
    match export_mp4(&timeline, &config, "results/neural-net.mp4") {
        Ok(()) => println!("Written results/neural-net.mp4"),
        Err(e) => eprintln!("export skipped: {e}"),
    }
}
