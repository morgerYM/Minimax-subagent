//! MCP tool handlers for the subagent system: run_subagent, list_subagents,
//! get_subagent. The actual agent loop / dispatcher live in
//! `src/subagent_impl.rs` and `minimax_api::subagent`.

use std::sync::Arc;

use minimax_api::mcp_params::*;
use minimax_api::subagent::{run_agent_loop, specs_for, SubagentRegistry};
use minimax_api::MiniMaxClient;
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

use crate::subagent_impl::McpToolDispatcher;
use crate::to_mcp_err;

pub async fn handle_run_subagent(
    client: &MiniMaxClient,
    registry: &Arc<SubagentRegistry>,
    params: RunSubagentParams,
) -> Result<CallToolResult, ErrorData> {
    let sub = registry.get(&params.name).ok_or_else(|| {
        ErrorData::internal_error(
            format!(
                "subagent '{}' not found. Use list_subagents to see available subagents.",
                params.name
            ),
            None,
        )
    })?;

    let dispatcher = McpToolDispatcher {
        client: client.clone(),
        registry: registry.clone(),
    };

    let result = run_agent_loop(client, sub, &params.task, 0, &dispatcher)
        .await
        .map_err(to_mcp_err)?;

    let json = serde_json::to_string_pretty(&result).map_err(to_mcp_err)?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

pub async fn handle_list_subagents(
    registry: &Arc<SubagentRegistry>,
) -> Result<CallToolResult, ErrorData> {
    let list = registry.list();
    let json = serde_json::to_string_pretty(&list).map_err(to_mcp_err)?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

pub async fn handle_get_subagent(
    registry: &Arc<SubagentRegistry>,
    params: GetSubagentParams,
) -> Result<CallToolResult, ErrorData> {
    let sub = registry.get(&params.name).ok_or_else(|| {
        ErrorData::internal_error(format!("subagent '{}' not found", params.name), None)
    })?;

    // Include the effective tool whitelist so callers can see what tools
    // the LLM will be offered.
    let specs = specs_for(sub);
    let tool_names: Vec<String> = specs.into_iter().map(|s| s.name).collect();

    let view = serde_json::json!({
        "name": sub.name,
        "description": sub.description,
        "system": sub.system,
        "model": sub.model,
        "max_tokens": sub.max_tokens,
        "temperature": sub.temperature,
        "max_iterations": sub.max_iterations,
        "max_depth": sub.max_depth,
        "allowed_tools": sub.allowed_tools,
        "effective_tools": tool_names,
    });

    let json = serde_json::to_string_pretty(&view).map_err(to_mcp_err)?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}
