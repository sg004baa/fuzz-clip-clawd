use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::history::ClipboardEntry;

/// Search entries by fuzzy matching against the query.
/// - Empty query: returns all entries in order (with score 0).
/// - Non-empty query: returns only matching entries, sorted by score descending.
pub fn search<'a>(query: &str, entries: &'a [ClipboardEntry]) -> Vec<(&'a ClipboardEntry, i64)> {
    if query.is_empty() {
        return entries.iter().map(|e| (e, 0i64)).collect();
    }

    let matcher = SkimMatcherV2::default();
    let mut results: Vec<(&ClipboardEntry, i64)> = entries
        .iter()
        .filter_map(|entry| {
            matcher
                .fuzzy_match(&entry.content, query)
                .map(|score| (entry, score))
        })
        .collect();

    results.sort_by(|a, b| b.1.cmp(&a.1));
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_entry(id: u64, content: &str) -> ClipboardEntry {
        ClipboardEntry {
            id,
            content: content.to_string(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_empty_query_returns_all() {
        let entries = vec![
            make_entry(1, "hello"),
            make_entry(2, "world"),
            make_entry(3, "foo"),
        ];
        let results = search("", &entries);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_fuzzy_match_filters() {
        let entries = vec![
            make_entry(1, "hello world"),
            make_entry(2, "goodbye world"),
            make_entry(3, "foo bar"),
        ];
        let results = search("helo", &entries);
        // "hello world" should match "helo" fuzzily
        assert!(!results.is_empty());
        assert!(results.iter().any(|(e, _)| e.content == "hello world"));
    }

    #[test]
    fn test_no_match_returns_empty() {
        let entries = vec![make_entry(1, "hello"), make_entry(2, "world")];
        let results = search("zzzzz", &entries);
        assert!(results.is_empty());
    }

    #[test]
    fn test_results_sorted_by_score() {
        let entries = vec![
            make_entry(1, "abc"),
            make_entry(2, "abcdef"),
            make_entry(3, "xyzabc"),
        ];
        let results = search("abc", &entries);
        // All should match; check they're sorted by score descending
        assert!(results.len() >= 2);
        for i in 0..results.len() - 1 {
            assert!(results[i].1 >= results[i + 1].1);
        }
    }
}
