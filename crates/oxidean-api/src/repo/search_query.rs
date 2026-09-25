//! Modest search qualifier parser (D-SRCH-12).
//!
//! Supports: bare keywords, `is:open` / `is:closed`, `author:<login>`, `path:<prefix>`.
//! Unknown `key:value` tokens are **stripped** (not treated as literal keywords).

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedSearchQuery {
    /// Remaining free-text keywords (joined with spaces).
    pub keywords: String,
    /// `open` | `closed` when `is:` present.
    pub is_state: Option<String>,
    pub author: Option<String>,
    pub path: Option<String>,
}

/// Parse `q` into keywords + known qualifiers. Unknown qualifiers are dropped.
pub fn parse_search_query(q: &str) -> ParsedSearchQuery {
    let mut keywords = Vec::new();
    let mut is_state = None;
    let mut author = None;
    let mut path = None;

    for token in q.split_whitespace() {
        if let Some((key, value)) = token.split_once(':') {
            let key = key.to_ascii_lowercase();
            let value = value.trim();
            if value.is_empty() {
                continue;
            }
            match key.as_str() {
                "is" => {
                    let v = value.to_ascii_lowercase();
                    if v == "open" || v == "closed" {
                        is_state = Some(v);
                    }
                    // unknown is: values stripped
                }
                "author" => author = Some(value.to_string()),
                "path" => path = Some(value.to_string()),
                _ => {
                    // Unknown qualifier — strip (D-SRCH-12 / RESEARCH).
                }
            }
        } else if !token.is_empty() {
            keywords.push(token.to_string());
        }
    }

    ParsedSearchQuery {
        keywords: keywords.join(" "),
        is_state,
        author,
        path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_known_qualifiers_and_keywords() {
        let p = parse_search_query("fix login author:ada is:open path:src/ language:rust");
        assert_eq!(p.keywords, "fix login");
        assert_eq!(p.author.as_deref(), Some("ada"));
        assert_eq!(p.is_state.as_deref(), Some("open"));
        assert_eq!(p.path.as_deref(), Some("src/"));
    }

    #[test]
    fn strips_unknown_qualifiers() {
        let p = parse_search_query("hello label:bug org:acme");
        assert_eq!(p.keywords, "hello");
        assert!(p.author.is_none());
    }
}
