//! Search and understanding tool handlers.

use async_trait::async_trait;

use crate::mcp_params::{UnderstandImageParams, WebSearchParams};
use crate::providers::{ProviderError, SearchOutput};

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

// ============================================================
// Trait
// ============================================================

#[async_trait]
pub trait SearchProvider: Send + Sync {
    async fn web_search(&self, query: &str) -> Result<SearchOutput, ProviderError>;
    async fn understand_image(&self, params: &UnderstandImageParams) -> Result<String, ProviderError>;
}

// ============================================================
// Handlers
// ============================================================

fn to_mcp_err(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

pub async fn handle_web_search(
    provider: &dyn SearchProvider,
    params: WebSearchParams,
) -> Result<CallToolResult, ErrorData> {
    let output = provider.web_search(&params.query).await.map_err(to_mcp_err)?;

    let mut lines = vec![format!("Search results ({}):", output.results.len()), String::new()];
    for (i, r) in output.results.iter().enumerate() {
        lines.push(format!("{}. {}", i + 1, r.title));
        lines.push(format!("   URL: {}", r.url));
        lines.push(format!("   {}", r.snippet));
        if let Some(date) = &r.date {
            lines.push(format!("   Date: {}", date));
        }
        lines.push(String::new());
    }
    if !output.related.is_empty() {
        lines.push("Related searches:".to_string());
        for rs in &output.related {
            lines.push(format!("  - {}", rs));
        }
    }

    Ok(CallToolResult::success(vec![Content::text(lines.join("\n"))]))
}

pub async fn handle_understand_image(
    provider: &dyn SearchProvider,
    params: UnderstandImageParams,
) -> Result<CallToolResult, ErrorData> {
    let content = provider.understand_image(&params).await.map_err(to_mcp_err)?;
    Ok(CallToolResult::success(vec![Content::text(content)]))
}
