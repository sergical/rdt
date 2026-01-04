use crate::error::Result;
use serde::Serialize;

/// Format and print output based on the format type
pub fn format_output<T: Serialize>(data: &T, format: &str) -> Result<()> {
    match format {
        "json" => {
            let output = serde_json::to_string_pretty(data)?;
            println!("{}", output);
        }
        "table" => {
            // For now, fall back to JSON for table format
            // TODO: Implement proper table formatting
            let output = serde_json::to_string_pretty(data)?;
            println!("{}", output);
        }
        _ => {
            let output = serde_json::to_string_pretty(data)?;
            println!("{}", output);
        }
    }
    Ok(())
}

/// Wrapper for consistent API response format
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
    pub meta: ResponseMeta,
}

#[derive(Serialize)]
pub struct ResponseMeta {
    pub rate_limit_remaining: Option<u32>,
    pub rate_limit_reset: Option<u64>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            meta: ResponseMeta {
                rate_limit_remaining: None,
                rate_limit_reset: None,
            },
        }
    }

    pub fn with_rate_limit(mut self, remaining: u32, reset: u64) -> Self {
        self.meta.rate_limit_remaining = Some(remaining);
        self.meta.rate_limit_reset = Some(reset);
        self
    }
}
