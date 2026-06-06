//! Central font registry for Codimate.
//!
//! Owns all embedded fonts and per-character fallback lookup.
//! Zero font-engine dependencies — consumers create their own handles
//! (e.g. `ab_glyph::FontRef`, `harfbuzz-rs::Face`) from the raw bytes.

include!(concat!(env!("OUT_DIR"), "/generated_fonts.rs"));

/// Opaque identifier for a registered font.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FontId(u16);

/// A registered font: metadata, embedded TTF/OTF bytes, Unicode coverage.
pub struct FontEntry {
    id: FontId,
    name: &'static str,
    data: &'static [u8],
    ranges: &'static [std::ops::RangeInclusive<u32>],
}

/// Registry of all embedded fonts with per-character fallback resolution.
pub struct FontRegistry;

impl FontRegistry {
    pub fn global() -> &'static Self {
        &FontRegistry
    }

    /// Raw bytes for the font identified by `id`.
    pub fn data(&self, id: FontId) -> &'static [u8] {
        generated::entries()
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.data)
            .unwrap_or(&[])
    }

    /// Name of the font (file stem, e.g. "DejaVuSansMono").
    pub fn name(&self, id: FontId) -> &'static str {
        generated::entries()
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.name)
            .unwrap_or("")
    }

    /// Number of registered fonts.
    pub fn len(&self) -> usize {
        generated::entries().len()
    }

    /// Iterate all font IDs in registration (priority) order.
    pub fn ids(&self) -> impl Iterator<Item = FontId> + 'static {
        generated::entries().iter().map(|e| e.id)
    }

    /// The highest-priority font that covers `ch`.
    ///
    /// Returns the first registered font whose Unicode ranges include `ch`.
    /// Whitespace always matches the first font.
    pub fn char_font(&self, ch: char) -> FontId {
        let cp = ch as u32;
        for entry in generated::entries() {
            if ch.is_whitespace() || entry.ranges.iter().any(|r| r.contains(&cp)) {
                return entry.id;
            }
        }
        generated::entries()
            .first()
            .map(|e| e.id)
            .unwrap_or(FontId(0))
    }

    /// Whether the font with `id` covers `ch`.
    pub fn font_has(&self, id: FontId, ch: char) -> bool {
        let cp = ch as u32;
        generated::entries()
            .iter()
            .find(|e| e.id == id)
            .map(|e| ch.is_whitespace() || e.ranges.iter().any(|r| r.contains(&cp)))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_fonts() {
        assert!(FontRegistry::global().len() > 0);
    }

    #[test]
    fn dejavu_has_latin() {
        let reg = FontRegistry::global();
        let id = reg.char_font('A');
        assert!(
            reg.name(id).contains("DejaVu"),
            "Latin 'A' should resolve to DejaVuSansMono"
        );
    }

    #[test]
    fn droid_has_cjk() {
        let reg = FontRegistry::global();
        let id = reg.char_font('中');
        let name = reg.name(id);
        assert!(
            name.contains("Droid"),
            "CJK '中' should resolve to DroidSansFallbackFull"
        );
    }

    #[test]
    fn droid_has_hiragana() {
        let reg = FontRegistry::global();
        let id = reg.char_font('は');
        assert!(reg.name(id).contains("Droid"));
    }

    #[test]
    fn data_returns_non_empty() {
        let reg = FontRegistry::global();
        for id in reg.ids() {
            assert!(!reg.data(id).is_empty(), "font {} has data", reg.name(id));
        }
    }
}
