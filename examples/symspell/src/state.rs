pub const DICT_ENTRIES: &[(&str, u32)] = &[("cat", 90), ("cut", 55), ("cot", 15)];
pub const K: usize = 1;
pub const QUERY: &str = "cit";

#[derive(Clone)]
pub struct SymSpell {
    pub entries: Vec<(String, u32)>,
    pub k: usize,
    pub query: String,
}

impl SymSpell {
    pub fn new() -> Self {
        Self {
            entries: DICT_ENTRIES
                .iter()
                .map(|(w, f)| (w.to_string(), *f))
                .collect(),
            k: K,
            query: QUERY.to_string(),
        }
    }
}

impl Default for SymSpell {
    fn default() -> Self {
        Self::new()
    }
}
