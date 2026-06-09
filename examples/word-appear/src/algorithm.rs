use crate::WordAppear;

/// One word in the appearance sequence.
#[derive(Clone, Debug, PartialEq)]
pub struct WordAppearEvent {
    pub word: String,
    pub index: usize,
}

/// Ordered trace of words to appear.
#[derive(Clone, Debug)]
pub struct WordAppearTrace {
    pub events: Vec<WordAppearEvent>,
}

pub fn word_appear_algorithm(state: WordAppear) -> WordAppearTrace {
    let events: Vec<WordAppearEvent> = state
        .sentence()
        .split_whitespace()
        .enumerate()
        .map(|(i, w)| WordAppearEvent {
            word: w.to_string(),
            index: i,
        })
        .collect();
    WordAppearTrace { events }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algorithm_splits_sentence_into_words() {
        let state = WordAppear::new();
        let trace = word_appear_algorithm(state);
        assert!(trace.events.len() >= 3, "should split into words");
    }

    #[test]
    fn each_event_has_unique_index() {
        let state = WordAppear::new();
        let trace = word_appear_algorithm(state);
        let indices: Vec<usize> = trace.events.iter().map(|e| e.index).collect();
        let mut sorted = indices.clone();
        sorted.sort();
        assert_eq!(indices, sorted, "indices should be sequential");
        for (i, &idx) in indices.iter().enumerate() {
            assert_eq!(idx, i, "index {} should equal position {}", idx, i);
        }
    }

    #[test]
    fn words_preserve_order() {
        let state = WordAppear::new();
        let sentence = state.sentence().to_string();
        let trace = word_appear_algorithm(state);
        let expected: Vec<&str> = sentence.split_whitespace().collect();
        for (event, expected_word) in trace.events.iter().zip(expected.iter()) {
            assert_eq!(event.word, *expected_word);
        }
    }

    #[test]
    fn trace_events_are_non_empty() {
        let state = WordAppear::new();
        let trace = word_appear_algorithm(state);
        for event in &trace.events {
            assert!(
                !event.word.is_empty(),
                "word at index {} should not be empty",
                event.index
            );
        }
    }
}
