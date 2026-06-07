//! Usage query tool handlers.

use async_trait::async_trait;

use crate::providers::{ProviderError, UsageResult};

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

// ============================================================
// Trait
// ============================================================

#[async_trait]
pub trait UsageProvider: Send + Sync {
    async fn query_usage(&self) -> Result<UsageResult, ProviderError>;
}

// ============================================================
// Handler
// ============================================================

fn to_mcp_err(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

pub async fn handle_query_usage(
    provider: &dyn UsageProvider,
) -> Result<CallToolResult, ErrorData> {
    let resp = provider.query_usage().await.map_err(to_mcp_err)?;

    let mut keys: Vec<&String> = resp.fields.keys().collect();
    keys.sort();
    let lines: Vec<String> = keys
        .iter()
        .filter_map(|k| resp.fields.get(*k).map(|v| format!("{}: {}", k, v)))
        .collect();

    Ok(CallToolResult::success(vec![Content::text(if lines.is_empty() {
        "Query succeeded.".to_string()
    } else {
        format!("Account usage:\n{}", lines.join("\n"))
    })]))
}
