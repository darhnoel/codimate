//! Central font registry for Codimate.
//!
//! Owns the embedded fonts, the fonts an author names at render time, and
//! per-character fallback lookup. Consumers create their own handles (e.g.
//! `ab_glyph::FontRef`, `harfbuzz-rs::Face`) from the raw bytes and face index.

mod entry;
mod registry;

pub use entry::{FontEntry, FontId};
pub use registry::FontRegistry;

include!(concat!(env!("OUT_DIR"), "/generated_fonts.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{is_path, squash};

    #[test]
    fn registry_has_fonts() {
        assert!(!FontRegistry::global().is_empty());
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

    fn khmer_file() -> String {
        format!(
            "{}/fonts/NotoSansKhmer-Regular.ttf",
            env!("CARGO_MANIFEST_DIR")
        )
    }

    #[test]
    fn a_font_file_loads_once_and_falls_back_for_what_it_lacks() {
        let reg = FontRegistry::global();
        let id = reg.load(&khmer_file()).unwrap();
        assert_eq!(reg.load(&khmer_file()).unwrap(), id, "cached by name");
        assert_eq!(reg.name(id), "NotoSansKhmer-Regular");
        assert!(!reg.data(id).is_empty());
        assert_eq!(reg.char_font_in(Some(id), 'ក'), id);
        // It has no CJK, so a Chinese character still finds Droid.
        assert!(reg.name(reg.char_font_in(Some(id), '中')).contains("Droid"));
        // Loading it changes nothing for text that did not ask for it.
        assert!(reg.name(reg.char_font('ក')).contains("Khmer"));
        assert_ne!(reg.char_font('ក'), id);
    }

    #[test]
    fn a_missing_file_or_family_says_so() {
        let reg = FontRegistry::global();
        let err = reg.load("fonts/nope.ttf").unwrap_err();
        assert!(err.contains("cannot read the font file"), "{err}");
        let err = reg.load("No Such Family Anywhere").unwrap_err();
        assert!(err.contains("no installed font"), "{err}");
    }

    #[test]
    fn a_path_is_told_from_a_family_name() {
        assert!(is_path("fonts/Inter.ttf"));
        assert!(is_path("Inter-Bold.OTF"));
        assert!(is_path(r"C:\Fonts\x"));
        assert!(!is_path("Inter"));
        assert!(!is_path("Helvetica Neue"));
        assert_eq!(squash("Inter-Bold"), squash("Inter Bold"));
    }

    #[test]
    fn data_returns_non_empty() {
        let reg = FontRegistry::global();
        for id in reg.ids() {
            assert!(!reg.data(id).is_empty(), "font {} has data", reg.name(id));
        }
    }
}
