use crate::config::Config;
use crate::error::{RdtError, Result};
use crate::nlp::patterns::PatternMatcher;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// How the query was parsed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParseMethod {
    Pattern,
    AI,
    Fallback,
}

/// Search parameters extracted from query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchParams {
    pub query: String,
    pub subreddit: Option<String>,
    pub sort: String,
    pub time: String,
    pub limit: u32,
    pub search_type: String,
    #[serde(skip)]
    pub parse_method: Option<ParseMethod>,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            query: String::new(),
            subreddit: None,
            sort: "relevance".to_string(),
            time: "all".to_string(),
            limit: 25,
            search_type: "posts".to_string(),
            parse_method: None,
        }
    }
}

/// Router that decides between pattern matching and AI
pub struct NlpRouter {
    pattern_matcher: PatternMatcher,
    needs_ai_patterns: Vec<Regex>,
}

impl NlpRouter {
    pub fn new() -> Self {
        // Patterns that indicate complex queries needing AI
        let needs_ai_patterns = vec![
            // Questions
            Regex::new(r"(?i)^(what|how|why|which|who|where)\b").unwrap(),
            // Conversational phrases
            Regex::new(r"(?i)\b(people saying|talking about|discussions? on|opinions? on)\b")
                .unwrap(),
            // Subjective queries
            Regex::new(r"(?i)\b(best|worst|controversial|popular|unpopular)\b").unwrap(),
            // Temporal with ambiguity
            Regex::new(r"(?i)\b(recently|lately|nowadays)\b").unwrap(),
            // Complex requests
            Regex::new(r"(?i)\b(compare|versus|vs\.?|difference between)\b").unwrap(),
        ];

        Self {
            pattern_matcher: PatternMatcher::new(),
            needs_ai_patterns,
        }
    }

    /// Check if the query needs NLP/AI processing
    pub fn needs_nlp(&self, query: &str) -> bool {
        // First, try pattern matching - if it matches, no need for AI
        if self.pattern_matcher.try_match(query).is_some() {
            return false;
        }

        // Check if query matches any "needs AI" patterns
        for pattern in &self.needs_ai_patterns {
            if pattern.is_match(query) {
                return true;
            }
        }

        // Check for multi-word natural language that doesn't match simple patterns
        let words: Vec<&str> = query.split_whitespace().collect();
        if words.len() > 5 {
            return true;
        }

        false
    }

    /// Parse query using pattern matching first, then AI fallback
    pub async fn parse_query(&self, query: &str) -> Result<SearchParams> {
        // Layer 1: Try pattern matching (instant, free)
        if let Some(mut params) = self.pattern_matcher.try_match(query) {
            params.parse_method = Some(ParseMethod::Pattern);
            return Ok(params);
        }

        // Layer 2: AI fallback (Claude Haiku on Bedrock)
        // If Bedrock fails, fall back to raw query
        match self.parse_with_ai(query).await {
            Ok(mut params) => {
                params.parse_method = Some(ParseMethod::AI);
                Ok(params)
            }
            Err(_) => Ok(SearchParams {
                query: query.to_string(),
                parse_method: Some(ParseMethod::Fallback),
                ..Default::default()
            }),
        }
    }

    /// Use Claude Haiku on Bedrock to parse complex queries
    async fn parse_with_ai(&self, query: &str) -> Result<SearchParams> {
        let config = Config::load()?;

        // Load AWS config with region from config or default to us-east-1
        let region = config
            .aws
            .region
            .as_deref()
            .unwrap_or("us-east-1");
        let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region.to_string()))
            .load()
            .await;

        let bedrock = aws_sdk_bedrockruntime::Client::new(&aws_config);

        let model_id = config.bedrock_model_id();

        let prompt = format!(
            r#"Parse the following Reddit search query into structured parameters. Return only valid JSON.

Query: "{}"

Return JSON with these fields:
- query: the main search terms (required)
- subreddit: specific subreddit if mentioned (optional, without r/ prefix)
- sort: one of "relevance", "hot", "new", "top" (default: "relevance")
- time: one of "hour", "day", "week", "month", "year", "all" (default: "all")
- limit: number of results 1-100 (default: 25)

Example input: "what are the best rust tutorials from this week"
Example output: {{"query": "rust tutorials", "sort": "top", "time": "week", "limit": 25}}

Now parse the query and return only the JSON:"#,
            query
        );

        let request = serde_json::json!({
            "anthropic_version": "bedrock-2023-05-31",
            "max_tokens": 200,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let response = bedrock
            .invoke_model()
            .model_id(&model_id)
            .content_type("application/json")
            .body(aws_sdk_bedrockruntime::primitives::Blob::new(
                serde_json::to_vec(&request).map_err(|e| RdtError::Bedrock(e.to_string()))?,
            ))
            .send()
            .await
            .map_err(|e| RdtError::Bedrock(format!("Bedrock invoke error: {}", e)))?;

        let body_bytes = response.body().as_ref();
        if body_bytes.is_empty() {
            return Err(RdtError::Bedrock("Empty response body from Bedrock".to_string()));
        }

        let response_body: serde_json::Value =
            serde_json::from_slice(body_bytes)
                .map_err(|e| RdtError::Bedrock(format!("JSON parse error: {}", e)))?;

        // Extract the text content from Claude's response
        let text = response_body["content"][0]["text"]
            .as_str()
            .ok_or_else(|| RdtError::Bedrock("No text in response".to_string()))?;

        // Extract JSON from markdown code blocks if present
        let json_text = if text.contains("```") {
            // Find JSON between code blocks
            text.lines()
                .skip_while(|line| !line.starts_with('{'))
                .take_while(|line| !line.starts_with("```"))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            text.to_string()
        };

        // Parse the JSON from Claude's response
        let parsed: serde_json::Value =
            serde_json::from_str(&json_text).map_err(|e| RdtError::Bedrock(format!("Failed to parse AI response: {}", e)))?;

        Ok(SearchParams {
            query: parsed["query"]
                .as_str()
                .unwrap_or(query)
                .to_string(),
            subreddit: parsed["subreddit"].as_str().map(String::from),
            sort: parsed["sort"]
                .as_str()
                .unwrap_or("relevance")
                .to_string(),
            time: parsed["time"].as_str().unwrap_or("all").to_string(),
            limit: parsed["limit"].as_u64().unwrap_or(25) as u32,
            search_type: "posts".to_string(),
            parse_method: None, // Set by caller
        })
    }
}

impl Default for NlpRouter {
    fn default() -> Self {
        Self::new()
    }
}
