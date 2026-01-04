use crate::nlp::router::SearchParams;
use regex::Regex;

/// ezff-style pattern matcher for common search queries
/// Returns Some(SearchParams) if a pattern matches, None otherwise
pub struct PatternMatcher {
    patterns: Vec<Pattern>,
}

struct Pattern {
    regex: Regex,
    extractor: Box<dyn Fn(&regex::Captures) -> SearchParams + Send + Sync>,
}

impl PatternMatcher {
    pub fn new() -> Self {
        let patterns = vec![
            // "<query> in <subreddit>"
            Pattern {
                regex: Regex::new(r"(?i)^(.+?)\s+in\s+r?/?(\w+)$").unwrap(),
                extractor: Box::new(|caps| SearchParams {
                    query: caps[1].trim().to_string(),
                    subreddit: Some(caps[2].to_string()),
                    ..Default::default()
                }),
            },
            // "<query> from <subreddit>"
            Pattern {
                regex: Regex::new(r"(?i)^(.+?)\s+from\s+r?/?(\w+)$").unwrap(),
                extractor: Box::new(|caps| SearchParams {
                    query: caps[1].trim().to_string(),
                    subreddit: Some(caps[2].to_string()),
                    ..Default::default()
                }),
            },
            // "top <query>"
            Pattern {
                regex: Regex::new(r"(?i)^top\s+(.+)$").unwrap(),
                extractor: Box::new(|caps| SearchParams {
                    query: caps[1].trim().to_string(),
                    sort: "top".to_string(),
                    ..Default::default()
                }),
            },
            // "<query> sorted by <sort>"
            Pattern {
                regex: Regex::new(r"(?i)^(.+?)\s+sorted\s+by\s+(hot|new|top|relevance)$").unwrap(),
                extractor: Box::new(|caps| SearchParams {
                    query: caps[1].trim().to_string(),
                    sort: caps[2].to_lowercase(),
                    ..Default::default()
                }),
            },
            // "<query> from this week"
            Pattern {
                regex: Regex::new(r"(?i)^(.+?)\s+from\s+this\s+week$").unwrap(),
                extractor: Box::new(|caps| SearchParams {
                    query: caps[1].trim().to_string(),
                    time: "week".to_string(),
                    ..Default::default()
                }),
            },
            // "<query> from this month"
            Pattern {
                regex: Regex::new(r"(?i)^(.+?)\s+from\s+this\s+month$").unwrap(),
                extractor: Box::new(|caps| SearchParams {
                    query: caps[1].trim().to_string(),
                    time: "month".to_string(),
                    ..Default::default()
                }),
            },
            // "<query> from this year"
            Pattern {
                regex: Regex::new(r"(?i)^(.+?)\s+from\s+this\s+year$").unwrap(),
                extractor: Box::new(|caps| SearchParams {
                    query: caps[1].trim().to_string(),
                    time: "year".to_string(),
                    ..Default::default()
                }),
            },
            // "<query> from today"
            Pattern {
                regex: Regex::new(r"(?i)^(.+?)\s+from\s+today$").unwrap(),
                extractor: Box::new(|caps| SearchParams {
                    query: caps[1].trim().to_string(),
                    time: "day".to_string(),
                    ..Default::default()
                }),
            },
            // "recent <query>"
            Pattern {
                regex: Regex::new(r"(?i)^recent\s+(.+)$").unwrap(),
                extractor: Box::new(|caps| SearchParams {
                    query: caps[1].trim().to_string(),
                    sort: "new".to_string(),
                    ..Default::default()
                }),
            },
            // "<query> limit <n>"
            Pattern {
                regex: Regex::new(r"(?i)^(.+?)\s+limit\s+(\d+)$").unwrap(),
                extractor: Box::new(|caps| SearchParams {
                    query: caps[1].trim().to_string(),
                    limit: caps[2].parse().unwrap_or(25),
                    ..Default::default()
                }),
            },
            // "posts about <query> in <subreddit>"
            Pattern {
                regex: Regex::new(r"(?i)^posts?\s+about\s+(.+?)\s+in\s+r?/?(\w+)$").unwrap(),
                extractor: Box::new(|caps| SearchParams {
                    query: caps[1].trim().to_string(),
                    subreddit: Some(caps[2].to_string()),
                    ..Default::default()
                }),
            },
            // "top <query> in <subreddit>"
            Pattern {
                regex: Regex::new(r"(?i)^top\s+(.+?)\s+in\s+r?/?(\w+)$").unwrap(),
                extractor: Box::new(|caps| SearchParams {
                    query: caps[1].trim().to_string(),
                    subreddit: Some(caps[2].to_string()),
                    sort: "top".to_string(),
                    ..Default::default()
                }),
            },
            // "top <query> from this week"
            Pattern {
                regex: Regex::new(r"(?i)^top\s+(.+?)\s+from\s+this\s+week$").unwrap(),
                extractor: Box::new(|caps| SearchParams {
                    query: caps[1].trim().to_string(),
                    sort: "top".to_string(),
                    time: "week".to_string(),
                    ..Default::default()
                }),
            },
            // "top <query> in <subreddit> from this week"
            Pattern {
                regex: Regex::new(
                    r"(?i)^top\s+(.+?)\s+in\s+r?/?(\w+)\s+from\s+this\s+week$",
                )
                .unwrap(),
                extractor: Box::new(|caps| SearchParams {
                    query: caps[1].trim().to_string(),
                    subreddit: Some(caps[2].to_string()),
                    sort: "top".to_string(),
                    time: "week".to_string(),
                    ..Default::default()
                }),
            },
            // "<query> in <subreddit> from this week"
            Pattern {
                regex: Regex::new(r"(?i)^(.+?)\s+in\s+r?/?(\w+)\s+from\s+this\s+week$").unwrap(),
                extractor: Box::new(|caps| SearchParams {
                    query: caps[1].trim().to_string(),
                    subreddit: Some(caps[2].to_string()),
                    time: "week".to_string(),
                    ..Default::default()
                }),
            },
        ];

        Self { patterns }
    }

    /// Try to match the query against all patterns
    pub fn try_match(&self, query: &str) -> Option<SearchParams> {
        for pattern in &self.patterns {
            if let Some(caps) = pattern.regex.captures(query) {
                return Some((pattern.extractor)(&caps));
            }
        }
        None
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_in_subreddit() {
        let matcher = PatternMatcher::new();
        let result = matcher.try_match("rust async in programming").unwrap();
        assert_eq!(result.query, "rust async");
        assert_eq!(result.subreddit, Some("programming".to_string()));
    }

    #[test]
    fn test_top_query() {
        let matcher = PatternMatcher::new();
        let result = matcher.try_match("top rust tutorials").unwrap();
        assert_eq!(result.query, "rust tutorials");
        assert_eq!(result.sort, "top");
    }

    #[test]
    fn test_from_this_week() {
        let matcher = PatternMatcher::new();
        let result = matcher.try_match("rust news from this week").unwrap();
        assert_eq!(result.query, "rust news");
        assert_eq!(result.time, "week");
    }

    #[test]
    fn test_complex_pattern() {
        let matcher = PatternMatcher::new();
        let result = matcher
            .try_match("top rust in programming from this week")
            .unwrap();
        assert_eq!(result.query, "rust");
        assert_eq!(result.subreddit, Some("programming".to_string()));
        assert_eq!(result.sort, "top");
        assert_eq!(result.time, "week");
    }
}
