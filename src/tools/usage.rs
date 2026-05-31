//! Usage query tool handlers.
//!
//! Provides token plan and usage querying via MiniMax API.

use minimax_api::MiniMaxClient;

use crate::to_mcp_err;

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

pub async fn handle_query_usage(
    client: &MiniMaxClient,
) -> Result<CallToolResult, ErrorData> {
    let resp = client
        .get_token_plan_remains()
        .await
        .map_err(to_mcp_err)?;

    let mut lines: Vec<String> = Vec::new();
    let mut keys: Vec<&String> = resp.extra.keys().collect();
    keys.sort();
    for key in keys {
        if let Some(val) = resp.extra.get(key) {
            lines.push(format!("{}: {}", key, val));
        }
    }

    Ok(CallToolResult::success(vec![Content::text(if lines.is_empty() {
        format!(
            "Query succeeded.\nstatus: {}",
            resp.base_resp.status_msg
        )
    } else {
        format!(
            "Account usage:\n{}",
            lines.join("\n")
        )
    })]))
}
