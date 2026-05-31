//! Search and understanding tool handlers.
//!
//! Provides web search and image understanding via MiniMax
//! Coding Plan API.

use minimax_api::types::*;
use minimax_api::utils;
use minimax_api::MiniMaxClient;

use crate::mcp_params::*;
use crate::to_mcp_err;

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

pub async fn handle_web_search(
    client: &MiniMaxClient,
    params: WebSearchParams,
) -> Result<CallToolResult, ErrorData> {
    let req = SearchRequest { q: params.query };
    let resp = client.search(&req).await.map_err(to_mcp_err)?;

    let mut lines = Vec::new();
    lines.push(format!("Search results ({}):", resp.organic.len()));
    lines.push(String::new());

    for (i, result) in resp.organic.iter().enumerate() {
        lines.push(format!("{}. {}", i + 1, result.title));
        lines.push(format!("   URL: {}", result.link));
        lines.push(format!("   {}", result.snippet));
        if let Some(date) = &result.date {
            lines.push(format!("   Date: {}", date));
        }
        lines.push(String::new());
    }

    if !resp.related_searches.is_empty() {
        lines.push("Related searches:".to_string());
        for rs in &resp.related_searches {
            lines.push(format!("  - {}", rs.query));
        }
    }

    Ok(CallToolResult::success(vec![Content::text(lines.join("\n"))]))
}

pub async fn handle_understand_image(
    client: &MiniMaxClient,
    params: UnderstandImageParams,
) -> Result<CallToolResult, ErrorData> {
    let processed = utils::process_image_url(&params.image_source).await;
    let req = VlmRequest {
        prompt: params.prompt,
        image_url: processed,
    };
    let resp = client.vlm(&req).await.map_err(to_mcp_err)?;

    let content = resp.content.unwrap_or_else(|| "No content returned".to_string());
    Ok(CallToolResult::success(vec![Content::text(content)]))
}
