pub const BUCKET_PREFIX_LEN: usize = 3;
pub const BUCKET_CAP: usize = 10_000;

pub fn spill_key(token: &str) -> Option<String> {
    if token.chars().count() < BUCKET_PREFIX_LEN + 1 {
        return None;
    }
    return Some(token.chars().take(BUCKET_PREFIX_LEN + 1).collect());
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct EntryKey {
    pub name: String,
    pub lat: i64,
    pub lng: i64,
}

pub fn entry_key(entry: &crate::PlaceEntry) -> EntryKey {
    return EntryKey { name: entry.name.clone(), lat: entry.lat, lng: entry.lng };
}

pub fn normalize(s: &str) -> String {
    return deunicode::deunicode(s).to_lowercase();
}

pub const SYNONYMS: &[(&str, &[&str])] = &[
    ("st", &["saint", "street"]),
    ("ave", &["avenue"]),
    ("av", &["avenue"]),
    ("rd", &["road"]),
    ("blvd", &["boulevard"]),
    ("dr", &["drive"]),
    ("ln", &["lane"]),
    ("hwy", &["highway"]),
    ("mt", &["mount", "mountain"]),
    ("ft", &["fort"]),
    ("sq", &["square"]),
];

pub fn expand_token(token: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (abbr, fulls) in SYNONYMS {
        if token == *abbr {
            out.extend(fulls.iter().map(|s| s.to_string()));
        }
    }
    if token.len() > 4 && token.ends_with("str") {
        out.push(format!("{token}asse"));
    }
    return out;
}

pub fn token_matches(query_token: &str, entry_token: &str) -> bool {
    if entry_token.starts_with(query_token) {
        return true;
    }
    for expansion in expand_token(query_token) {
        if entry_token.starts_with(&expansion) {
            return true;
        }
    }
    for expansion in expand_token(entry_token) {
        if expansion.starts_with(query_token) {
            return true;
        }
    }
    return false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synonyms_match() {
        assert!(token_matches("st", "saint"));
        assert!(token_matches("st", "street"));
        assert!(token_matches("hauptstrasse", "hauptstr"));
        assert!(token_matches("hauptstr", "hauptstrasse"));
        assert!(token_matches("ave", "avenue"));
        assert!(token_matches("koben", "kobenhavn"));
        assert!(!token_matches("street", "saint"));
        assert!(!token_matches("kastellet", "kastanievej"));
    }

    #[test]
    fn normalize_folds() {
        assert_eq!(normalize("København"), "kobenhavn");
        assert_eq!(normalize("Straße"), "strasse");
    }

    #[test]
    fn spill_keys() {
        assert_eq!(spill_key("berlin"), Some("berl".to_string()));
        assert_eq!(spill_key("berl"), Some("berl".to_string()));
        assert_eq!(spill_key("ber"), None);
    }
}
