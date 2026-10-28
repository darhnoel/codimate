//! A font's identity in the registry, and the entry that describes an embedded one.

/// Opaque identifier for a registered font.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FontId(pub(crate) u16);

/// A registered font: metadata, embedded TTF/OTF bytes, Unicode coverage.
pub struct FontEntry {
    pub(crate) id: FontId,
    pub(crate) name: &'static str,
    pub(crate) data: &'static [u8],
    pub(crate) ranges: &'static [std::ops::RangeInclusive<u32>],
}
