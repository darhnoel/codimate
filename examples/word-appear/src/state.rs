/// Concept state: a sentence to animate word-by-word.
#[derive(Clone, Debug)]
pub struct WordAppear {
    sentence: String,
}

impl WordAppear {
    pub fn new() -> Self {
        Self {
            sentence: "This is What Codimate Can Do.".to_string(),
        }
    }

    pub fn sentence(&self) -> &str {
        &self.sentence
    }
}

impl Default for WordAppear {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_sentence_is_not_empty() {
        let state = WordAppear::new();
        assert!(
            !state.sentence().is_empty(),
            "default sentence should not be empty"
        );
    }

    #[test]
    fn sentence_contains_words() {
        let state = WordAppear::new();
        let words: Vec<&str> = state.sentence().split_whitespace().collect();
        assert!(
            words.len() >= 3,
            "default sentence should have at least 3 words, got {}",
            words.len()
        );
    }
}
