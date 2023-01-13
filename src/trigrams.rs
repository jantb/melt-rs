use ascii_utils::Check;

#[test]
fn test_get_trigram() {
    let trigrams = trigram("Hello, wor杯ld!");
    assert_eq!(trigrams, vec!["hel", "ell", "llo", "lo,", "o, ", ", w", " wo", "wor", "or杯", "r杯l","杯ld", "ld!"]);
}

pub fn trigram(word: &str) -> Vec<String> {
    let mut word = word.chars().filter(|&c| c.is_printable() || c.is_ascii()).collect::<String>();
    word.make_ascii_lowercase();
    let chars:Vec<char> = word.chars().collect();
    if chars.len() < 3 {
        return vec![];
    }

    let mut trigrams = Vec::with_capacity(chars.len() - 2);
    let mut seen = std::collections::HashSet::new();

    for i in 1..chars.len() - 1 {
        let trigram = &chars[i-1..i+2];
        if !seen.contains(trigram) {
            seen.insert(trigram);
            trigrams.push(trigram.into_iter().collect());
        }
    }

    trigrams
}