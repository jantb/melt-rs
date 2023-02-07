pub fn bigram(word: &str) -> Vec<String> {
    let chars: Vec<char> = word
        .to_lowercase()
        .chars()
        .collect::<String>()
        .chars()
        .collect();
    if chars.len() < 2 {
        return vec![];
    }

    let bigrams = (1..chars.len())
        .into_iter()
        .filter_map(|i| {
            let trigram = &chars[i - 1..i + 1];
            let trigram_str = trigram.iter().collect();
            Some(trigram_str)
        })
        .collect();

    bigrams
}

#[test]
fn test_get_bigram() {
    let bigrams = bigram("Hello");
    assert_eq!(bigrams, vec!["he", "el", "ll", "lo"]);
}
