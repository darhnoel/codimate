use codimate_example_swap::{explain, swap_algorithm, swap_motion, swap_view, Swap, SwapTiming};

fn main() {
    explain("Swap")
        .state(Swap::new("A", "B"))
        .view(swap_view)
        .algorithm(swap_algorithm)
        .motion(swap_motion)
        .timing(SwapTiming {
            move_duration: 0.75,
            pause: 0.20,
        })
        .render("results/swap.mp4");
}
