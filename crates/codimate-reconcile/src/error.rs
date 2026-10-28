use std::fmt;

/// Why a Scene could not be built.
///
/// Every one of these is an authoring mistake — an unknown shape kind, a
/// motion path that does not exist, LaTeX that will not typeset. The frontend
/// decides how to present it; this crate only says what went wrong.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Error(pub String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

/// A `Result` whose error is a Scene-building [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
