//! Dispatcher trait: how the agent loop invokes tools.
//!
//! The lib-side `run_agent_loop` is generic over `D: ToolDispatcher`.
//! The binary implements it (in `src/subagent_impl.rs`) and routes
//! each call to either an existing MCP tool handler or a recursive
//! `run_subagent` invocation.

use async_trait::async_trait;
use serde_json::Value;

use crate::error::MiniMaxError;

/// Result of dispatching a single tool call.
///
/// `output` is sent back to the LLM as a `tool_result` content block.
/// For errors, `is_error = true` and `output` is the error message —
/// this lets the LLM see the failure and decide what to do next.
pub struct DispatchResult {
    pub output: String,
    pub is_error: bool,
}

/// Routes one tool invocation. The `current_depth` parameter is the
/// agent loop's current recursion depth (0 for top-level calls);
/// implementations can refuse recursive `run_subagent` beyond a
/// configured `max_depth`.
#[async_trait]
pub trait ToolDispatcher: Send + Sync {
    async fn dispatch(
        &self,
        tool_name: &str,
        tool_input: Value,
        current_depth: u32,
    ) -> Result<DispatchResult, MiniMaxError>;
}
