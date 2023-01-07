use ascii_utils::Check;

#[test]
fn test_get_trigram() {
    let trigrams = trigram("Hello, world!");
    assert_eq!(trigrams, vec!["hel", "ell", "llo", "lo,", "o, ", ", w", " wo", "wor", "orl", "rld", "ld!"]);
}

pub fn trigram(word: &str) -> Vec<String> {
    let mut word = word.chars().filter(|&c| c.is_printable() || c.is_ascii()).collect::<String>();
    word.make_ascii_lowercase();

    if word.len() < 3 {
        return vec![];
    }

    let mut trigrams = Vec::with_capacity(word.len() - 2);
    let mut seen = std::collections::HashSet::new();

    for i in 1..word.len() - 1 {
        let trigram = &word[i-1..i+2];
        if !seen.contains(trigram) {
            seen.insert(trigram);
            trigrams.push(trigram.to_string());
        }
    }

    trigrams
}