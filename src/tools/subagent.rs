//! MCP tool handlers for the subagent system: run_subagent, list_subagents,
//! get_subagent. The actual agent loop / self-contained tools live in
//! `src/subagent_impl.rs` and `minimax_api::subagent`.

use std::sync::Arc;

use minimax_api::mcp_params::*;
use minimax_api::subagent::{
    run_agent_loop, tools_for_subagent, AgentTool, SubagentRegistry,
};
use minimax_api::MiniMaxClient;
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

use crate::to_mcp_err;

pub async fn handle_run_subagent(
    client: &MiniMaxClient,
    registry: &Arc<SubagentRegistry>,
    all_tools: &[AgentTool],
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

    // Determine effective allowed_tools: param overrides config
    let sub_tools = if let Some(tool_override) = &params.allowed_tools {
        let mut effective = sub.clone();
        effective.allowed_tools = Some(tool_override.clone());
        tools_for_subagent(all_tools, &effective)
    } else {
        tools_for_subagent(all_tools, sub)
    };

    let result = run_agent_loop(client, sub, &params.task, 0, &sub_tools)
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
    all_tools: &[AgentTool],
    params: GetSubagentParams,
) -> Result<CallToolResult, ErrorData> {
    let sub = registry.get(&params.name).ok_or_else(|| {
        ErrorData::internal_error(format!("subagent '{}' not found", params.name), None)
    })?;

    let sub_tools = tools_for_subagent(all_tools, sub);
    let tool_names: Vec<String> = sub_tools.iter().map(|t| t.name.clone()).collect();

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
