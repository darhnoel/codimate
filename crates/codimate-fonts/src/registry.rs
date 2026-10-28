//! The registry: embedded fonts, fonts loaded by name or path, and per-character fallback.

use std::sync::{OnceLock, RwLock};

use crate::{generated, FontId};

/// A font loaded at render time: from a file, or found among installed fonts.
struct Loaded {
    spec: String,
    name: &'static str,
    data: &'static [u8],
    index: u32,
    face: ttf_parser::Face<'static>,
}

fn loaded() -> &'static RwLock<Vec<Loaded>> {
    static LOADED: OnceLock<RwLock<Vec<Loaded>>> = OnceLock::new();
    LOADED.get_or_init(|| RwLock::new(Vec::new()))
}

/// Every installed font, scanned once and only when a family name is asked for.
fn system() -> &'static fontdb::Database {
    static SYSTEM: OnceLock<fontdb::Database> = OnceLock::new();
    SYSTEM.get_or_init(|| {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        db
    })
}

/// A font named by file rather than by family: it has a separator or a font
/// file's extension.
pub(crate) fn is_path(spec: &str) -> bool {
    let lower = spec.to_ascii_lowercase();
    spec.contains(['/', '\\'])
        || [".ttf", ".otf", ".ttc", ".otc"]
            .iter()
            .any(|ext| lower.ends_with(ext))
}

/// "Inter Bold", "Inter-Bold" and "interbold" are the same name.
pub(crate) fn squash(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// The bytes and face index of an installed font, by family or face name.
fn installed(name: &str) -> Result<(Vec<u8>, u32), String> {
    let db = system();
    let wanted = squash(name);
    let id = db
        .query(&fontdb::Query {
            families: &[fontdb::Family::Name(name)],
            ..Default::default()
        })
        .or_else(|| {
            db.faces()
                .find(|f| {
                    squash(&f.post_script_name) == wanted
                        || f.families
                            .iter()
                            .any(|(family, _)| squash(family) == wanted)
                })
                .map(|f| f.id)
        })
        .ok_or_else(|| {
            format!(
                "no installed font is called \"{name}\" — install it, or give the \
                 path to a .ttf or .otf file"
            )
        })?;
    db.with_face_data(id, |data, index| (data.to_vec(), index))
        .ok_or_else(|| format!("could not read the installed font \"{name}\""))
}

/// Registry of all embedded fonts with per-character fallback resolution.
pub struct FontRegistry;

impl FontRegistry {
    #[must_use]
    pub fn global() -> &'static Self {
        &FontRegistry
    }

    /// The font `spec` names — a `.ttf`/`.otf` path or an installed family
    /// name — loaded once per process and kept for its life.
    ///
    /// Loaded fonts only take part when asked for by id; [`char_font`]'s
    /// fallback stays the embedded set, so loading a font never changes how
    /// other text looks.
    ///
    /// [`char_font`]: Self::char_font
    pub fn load(&self, spec: &str) -> Result<FontId, String> {
        let embedded = generated::entries().len();
        if let Some(i) = loaded()
            .read()
            .expect("font registry lock is not poisoned")
            .iter()
            .position(|f| f.spec == spec)
        {
            return Ok(FontId((embedded + i) as u16));
        }
        let (bytes, index) = if is_path(spec) {
            let bytes = std::fs::read(spec)
                .map_err(|e| format!("cannot read the font file \"{spec}\": {e}"))?;
            (bytes, 0)
        } else {
            installed(spec)?
        };
        self.register(spec, bytes, index)
    }

    /// Register a font from bytes the caller already has, under the `spec` its
    /// scenes name it by. Where there is no file system (the browser), this is
    /// how a scene's font reaches the registry: [`load`] then finds it by `spec`
    /// and never reads a file.
    ///
    /// [`load`]: Self::load
    pub fn provide(&self, spec: &str, bytes: Vec<u8>) -> Result<FontId, String> {
        let embedded = generated::entries().len();
        if let Some(i) = loaded()
            .read()
            .expect("font registry lock is not poisoned")
            .iter()
            .position(|f| f.spec == spec)
        {
            return Ok(FontId((embedded + i) as u16));
        }
        self.register(spec, bytes, 0)
    }

    fn register(&self, spec: &str, bytes: Vec<u8>, index: u32) -> Result<FontId, String> {
        let embedded = generated::entries().len();
        // ponytail: leaked — a film names a handful of fonts and keeps them all
        // to the end, the way formulas and pictures are cached.
        let data: &'static [u8] = Box::leak(bytes.into_boxed_slice());
        let face = ttf_parser::Face::parse(data, index)
            .map_err(|e| format!("\"{spec}\" is not a font Codimate can read: {e}"))?;
        let name = match is_path(spec) {
            true => std::path::Path::new(spec)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(spec),
            false => spec,
        };
        let mut fonts = loaded()
            .write()
            .expect("font registry lock is not poisoned");
        // Another thread may have loaded it while this one read the file.
        if let Some(i) = fonts.iter().position(|f| f.spec == spec) {
            return Ok(FontId((embedded + i) as u16));
        }
        fonts.push(Loaded {
            spec: spec.to_string(),
            name: Box::leak(name.to_string().into_boxed_str()),
            data,
            index,
            face,
        });
        Ok(FontId((embedded + fonts.len() - 1) as u16))
    }

    /// The loaded font for `id`, or `None` when `id` is an embedded one.
    fn with_loaded<T>(&self, id: FontId, f: impl FnOnce(&Loaded) -> T) -> Option<T> {
        let i = (id.0 as usize).checked_sub(generated::entries().len())?;
        loaded()
            .read()
            .expect("font registry lock is not poisoned")
            .get(i)
            .map(f)
    }

    /// Raw bytes for the font identified by `id`.
    #[must_use]
    pub fn data(&self, id: FontId) -> &'static [u8] {
        if let Some(data) = self.with_loaded(id, |f| f.data) {
            return data;
        }
        generated::entries()
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.data)
            .unwrap_or(&[])
    }

    /// Which face in [`data`](Self::data) to use — non-zero only for an
    /// installed font that lives in a collection (`.ttc`).
    #[must_use]
    pub fn index(&self, id: FontId) -> u32 {
        self.with_loaded(id, |f| f.index).unwrap_or(0)
    }

    /// Name of the font (file stem, e.g. "DejaVuSansMono").
    #[must_use]
    pub fn name(&self, id: FontId) -> &'static str {
        if let Some(name) = self.with_loaded(id, |f| f.name) {
            return name;
        }
        generated::entries()
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.name)
            .unwrap_or("")
    }

    /// Number of embedded fonts.
    #[must_use]
    pub fn len(&self) -> usize {
        generated::entries().len()
    }

    /// Whether no font is embedded — never, in a working build.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate the embedded font IDs in registration (priority) order.
    pub fn ids(&self) -> impl Iterator<Item = FontId> + 'static {
        generated::entries().iter().map(|e| e.id)
    }

    /// `primary` if it has `ch`, else the embedded font that does.
    #[must_use]
    pub fn char_font_in(&self, primary: Option<FontId>, ch: char) -> FontId {
        match primary {
            Some(id) if self.font_has(id, ch) => id,
            _ => self.char_font(ch),
        }
    }

    /// The highest-priority font that covers `ch`.
    ///
    /// Returns the first registered font whose Unicode ranges include `ch`.
    /// Whitespace always matches the first font.
    #[must_use]
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
    #[must_use]
    pub fn font_has(&self, id: FontId, ch: char) -> bool {
        if let Some(has) = self.with_loaded(id, |f| f.face.glyph_index(ch).is_some()) {
            return ch.is_whitespace() || has;
        }
        let cp = ch as u32;
        generated::entries()
            .iter()
            .find(|e| e.id == id)
            .map(|e| ch.is_whitespace() || e.ranges.iter().any(|r| r.contains(&cp)))
            .unwrap_or(false)
    }
}
