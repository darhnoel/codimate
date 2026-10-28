/// Why a [`formula()`] could not be produced.
#[derive(Debug)]
#[non_exhaustive]
pub enum FormulaError {
    /// `mitex` could not translate the LaTeX into Typst markup.
    Mitex(String),
    /// The `typst` binary could not be spawned — is it installed / on `PATH`?
    TypstSpawn(std::io::Error),
    /// `typst` ran but reported a compile error.
    TypstCompile(String),
    /// The emitted SVG could not be parsed into paths.
    Svg(String),
    /// An imported SVG file could not be read.
    SvgRead(String),
    /// An imported SVG draws text, which this build cannot render.
    SvgText(String),
}
