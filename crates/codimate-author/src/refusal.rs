/// Why an authoring call was refused, worded for the author.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct Refusal(pub String);

impl<E: std::fmt::Display> From<E> for Refusal {
    fn from(e: E) -> Self {
        Refusal(e.to_string())
    }
}
