use codimate_example_symspell::{
    explain, symspell_algorithm, symspell_motion, symspell_view, SymSpell, SymSpellTiming,
};

fn main() {
    explain("SymSpell")
        .state(SymSpell::new())
        .view(symspell_view)
        .algorithm(symspell_algorithm)
        .motion(symspell_motion)
        .timing(SymSpellTiming::default())
        .render("results/symspell.mp4");
}
