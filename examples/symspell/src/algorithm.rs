use std::collections::HashMap;

#[derive(Clone)]
pub enum SymStep {
    Intro,
    IndexWord {
        word: String,
        freq: u32,
        deletes: Vec<String>,
    },
    IndexBuilt,
    GenerateDeletes {
        term: String,
        deletes: Vec<String>,
    },
    Lookup {
        variant: String,
        hits: Vec<(String, u32)>,
    },
    Verify {
        candidate: String,
        distance: u32,
        accepted: bool,
    },
    Rank {
        ordered: Vec<(String, u32)>,
    },
    Answer {
        word: String,
        ordered: Vec<(String, u32)>,
    },
    Final,
}

pub struct SymTrace {
    pub steps: Vec<SymStep>,
}

fn generate_deletes(word: &str) -> Vec<String> {
    let mut deletes = Vec::new();
    for i in 0..word.len() {
        let mut s = String::with_capacity(word.len() - 1);
        s.push_str(&word[..i]);
        s.push_str(&word[i + 1..]);
        deletes.push(s);
    }
    deletes
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (curr[j - 1] + 1).min(prev[j] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

pub fn symspell_algorithm(state: crate::SymSpell) -> SymTrace {
    let mut steps = Vec::new();
    let mut index: HashMap<String, Vec<(String, u32)>> = HashMap::new();

    steps.push(SymStep::Intro);

    for (word, freq) in &state.entries {
        let deletes = generate_deletes(word);
        for d in &deletes {
            index
                .entry(d.clone())
                .or_default()
                .push((word.clone(), *freq));
        }
        steps.push(SymStep::IndexWord {
            word: word.clone(),
            freq: *freq,
            deletes,
        });
    }

    steps.push(SymStep::IndexBuilt);

    let query_deletes = generate_deletes(&state.query);
    let mut all_query_variants = query_deletes.clone();
    all_query_variants.push(state.query.clone());

    steps.push(SymStep::GenerateDeletes {
        term: state.query.clone(),
        deletes: query_deletes.clone(),
    });

    for variant in &all_query_variants {
        let hits = index.get(variant).cloned().unwrap_or_default();
        steps.push(SymStep::Lookup {
            variant: variant.clone(),
            hits,
        });
    }

    let mut all_candidates: Vec<(String, u32)> = Vec::new();
    for variant in &all_query_variants {
        if let Some(entries) = index.get(variant) {
            for (word, freq) in entries {
                let dist = edit_distance(word, &state.query) as u32;
                let accepted = dist <= state.k as u32;
                if accepted && !all_candidates.iter().any(|(w, _)| w == word) {
                    all_candidates.push((word.clone(), *freq));
                }
                steps.push(SymStep::Verify {
                    candidate: word.clone(),
                    distance: dist,
                    accepted,
                });
            }
        }
    }

    all_candidates.sort_by(|a, b| b.1.cmp(&a.1));
    steps.push(SymStep::Rank {
        ordered: all_candidates.clone(),
    });

    let answer = all_candidates
        .first()
        .map(|(w, _)| w.clone())
        .unwrap_or_default();
    steps.push(SymStep::Answer {
        word: answer,
        ordered: all_candidates.clone(),
    });
    steps.push(SymStep::Final);

    SymTrace { steps }
}
