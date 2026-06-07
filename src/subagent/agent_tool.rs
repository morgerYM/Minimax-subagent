//! Self-contained tool type — inspired by OpenClaw's AnyAgentTool.
//!
//! Each [`AgentTool`] bundles a tool's schema (what the LLM sees) with its
//! execution logic (what happens when the LLM calls it). The agent loop
//! never knows which provider or client is behind the tool — it just calls
//! [`AgentTool::execute`].
//!
//! This replaces the old pattern where `ToolSpec` (schema) lived in
//! `tool_catalog.rs` and execution lived in `McpToolDispatcher` (a big
//! `match` in `subagent_impl.rs`).

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use rmcp::model::{CallToolResult, RawContent};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::MiniMaxError;
use crate::types::ToolSpec;

// ============================================================
// Re-export from types.rs for convenience
// ============================================================

pub use crate::subagent::types::DispatchResult;

// ============================================================
// Constants
// ============================================================

/// Name of the recursive subagent tool. Always implicitly available to
/// every subagent, regardless of `allowed_tools` whitelist.
pub const RUN_SUBAGENT_NAME: &str = "run_subagent";

// ============================================================
// Sentinel param type
// ============================================================

/// Sentinel Param type for tools that take no input.
#[derive(Debug, Clone, Default, serde::Deserialize, JsonSchema)]
pub struct NoParams {}

// ============================================================
// Async boxed future
// ============================================================

/// Type-erased, pinned, heap-allocated future that is `Send`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// ============================================================
// Execute function type
// ============================================================

/// Signature for a tool's execute function.
///
/// - `tool_input`: the deserialized JSON parameters the LLM passed
/// - `current_depth`: recursion depth (0 for top-level; used by subagent tool)
///
/// Returns a [`DispatchResult`] on success, or a [`MiniMaxError`] on
/// unrecoverable failure (which terminates the agent loop).
pub type ExecuteFn =
    Arc<dyn Fn(Value, u32) -> BoxFuture<'static, Result<DispatchResult, MiniMaxError>> + Send + Sync>;

// ============================================================
// Self-contained tool
// ============================================================

/// A self-contained tool that bundles schema + execution logic.
///
/// # Design (OpenClaw AnyAgentTool pattern)
///
/// ```
/// // LLM sees:        name + description + input_schema (parameters)
/// // Agent loop:      tool.execute(tool_input, depth)
/// // Tool internally: captures provider/client — loop is unaware
/// ```
///
/// This eliminates the central `match` dispatcher: each tool knows how
/// to execute itself. Adding a new tool means creating one `AgentTool`
/// and registering it — no changes to dispatch logic.
pub struct AgentTool {
    /// Tool name (the LLM uses this to invoke the tool).
    pub name: String,
    /// Tool description (rendered in the LLM's system prompt).
    pub description: String,
    /// JSON Schema for the tool's input parameters.
    pub input_schema: Value,
    /// Execution logic, closed over whatever provider/client it needs.
    pub execute: ExecuteFn,
}

impl AgentTool {
    /// Create a new self-contained tool.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
        execute: ExecuteFn,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
            execute,
        }
    }

    /// Convert to a `ToolSpec` for the LLM API.
    ///
    /// The agent loop calls this to produce the `tools` field of the
    /// chat request — the LLM sees the schema but never the execute fn.
    pub fn to_spec(&self) -> ToolSpec {
        ToolSpec {
            name: self.name.clone(),
            description: self.description.clone(),
            input_schema: self.input_schema.clone(),
        }
    }
}

// Manual Debug — skip the `execute` closure (not Debug).
impl std::fmt::Debug for AgentTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentTool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("input_schema", &self.input_schema)
            .finish()
    }
}

// Manual Clone — Arc<ExecuteFn> clone is cheap.
impl Clone for AgentTool {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            description: self.description.clone(),
            input_schema: self.input_schema.clone(),
            execute: Arc::clone(&self.execute),
        }
    }
}

// ============================================================
// Helpers
// ============================================================

/// Generate a JSON Schema `Value` for any type that implements `JsonSchema`.
pub fn schema_of<T: JsonSchema>() -> Value {
    serde_json::to_value(schemars::schema_for!(T))
        .expect("schema_for! must produce a serializable schema")
}

/// Parse tool input from a `serde_json::Value`.
///
/// Used inside `ExecuteFn` closures to deserialize the LLM's JSON input
/// into the typed params struct the handler function expects.
pub fn parse_input<T: DeserializeOwned>(input: Value) -> Result<T, MiniMaxError> {
    serde_json::from_value(input)
        .map_err(|e| MiniMaxError::Config(format!("param parse error: {e}")))
}

/// Convert an MCP `CallToolResult` into a `DispatchResult`.
///
/// Extracts all text content blocks and joins them with newlines.
/// Errors are propagated from the handler's error path.
pub fn call_tool_result_to_dispatch(r: CallToolResult) -> DispatchResult {
    let output = r
        .content
        .iter()
        .filter_map(|c| match &c.raw {
            RawContent::Text(t) => Some(t.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    DispatchResult {
        output,
        is_error: false,
    }
}

/// Wrap a displayable error into a `MiniMaxError::Config`.
pub fn to_tool_err(e: impl std::fmt::Display) -> MiniMaxError {
    MiniMaxError::Config(format!("tool error: {e}"))
}
