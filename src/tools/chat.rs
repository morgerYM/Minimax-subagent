//! Chat tool handlers.
//!
//! Provides chat completion via MiniMax Anthropic-compatible API.

use minimax_api::types::*;
use minimax_api::MiniMaxClient;

use minimax_api::mcp_params::*;
use crate::to_mcp_err;

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

pub async fn handle_chat(
    client: &MiniMaxClient,
    params: ChatParams,
) -> Result<CallToolResult, ErrorData> {
    let model = params
        .model
        .unwrap_or_else(|| minimax_api::consts::DEFAULT_CHAT_MODEL.to_string());
    let req = ChatRequest {
        model,
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: params.prompt,
        }],
        system: params.system,
        max_tokens: params.max_tokens.or(Some(4096)),
        temperature: params.temperature,
        top_p: None,
        stream: false,
    };

    let resp = client.chat(&req).await.map_err(to_mcp_err)?;

    let text: Vec<String> = resp
        .content
        .iter()
        .filter(|b| b.block_type == "text")
        .filter_map(|b| b.text.as_deref())
        .map(String::from)
        .collect();

    if text.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "聊天完成，但无文本输出。".to_string(),
        )]));
    }

    let mut result = text.join("\n");
    if let Some(usage) = &resp.usage {
        result.push_str(&format!(
            "\n\n[model: {}, input: {} tokens, output: {} tokens]",
            resp.model,
            usage.input_tokens.unwrap_or(0),
            usage.output_tokens.unwrap_or(0)
        ));
    }

    Ok(CallToolResult::success(vec![Content::text(result)]))
}
