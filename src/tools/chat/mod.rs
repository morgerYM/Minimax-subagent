//! Chat tool handlers.
//!
//! Provides chat completion via the ChatProvider trait.

use async_trait::async_trait;

use crate::mcp_params::ChatParams;
use crate::providers::{ChatOutput, ProviderError};

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

// ============================================================
// Trait
// ============================================================

#[async_trait]
pub trait ChatProvider: Send + Sync {
    async fn chat(&self, params: &ChatParams) -> Result<ChatOutput, ProviderError>;
}

// ============================================================
// Handler
// ============================================================

fn to_mcp_err(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

pub async fn handle_chat(
    provider: &dyn ChatProvider,
    params: ChatParams,
) -> Result<CallToolResult, ErrorData> {
    let output = provider.chat(&params).await.map_err(to_mcp_err)?;

    let mut result = output.text;
    let input_tokens = output.input_tokens.unwrap_or(0);
    let output_tokens = output.output_tokens.unwrap_or(0);
    if input_tokens > 0 || output_tokens > 0 {
        result.push_str(&format!(
            "\n\n[model: {}, input: {} tokens, output: {} tokens]",
            output.model, input_tokens, output_tokens
        ));
    }

    Ok(CallToolResult::success(vec![Content::text(result)]))
}
