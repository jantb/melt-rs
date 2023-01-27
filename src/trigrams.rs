use ascii_utils::Check;

#[test]
fn test_get_trigram() {
    let trigrams = trigram("Hello, wor杯ld!");
    assert_eq!(trigrams, vec!["hel", "ell", "llo", "lo,", "o, ", ", w", " wo", "wor", "or杯", "r杯l","杯ld", "ld!"]);
}

pub fn trigram(word: &str) -> Vec<String> {
    let chars: Vec<char> = word.to_lowercase()
        .chars()
        .filter(|&c| c.is_printable() || c.is_ascii())
        .collect::<String>().chars().collect();
    if chars.len() < 3 {
        return vec![];
    }

    let trigrams = (1..chars.len() - 1)
        .into_iter()
        .filter_map(|i| {
            let trigram = &chars[i - 1..i + 2];
            let trigram_str = trigram.iter().collect();
            Some(trigram_str)
        })
        .collect();

    trigrams
}