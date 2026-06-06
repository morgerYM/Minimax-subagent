//! Subagent data types: user-facing config, MCP params, runtime results.
//!
//! `SubagentDef` is loaded from `subagents/<name>.json` at startup.
//! `LoopResult` / `ToolCallRecord` describe what the agent loop did.

use serde::{Deserialize, Serialize};

/// One subagent definition, loaded from `subagents/<name>.json` at startup.
#[derive(Debug, Clone, Deserialize)]
pub struct SubagentDef {
    /// Subagent identifier (must be unique across the registry).
    pub name: String,
    /// One-line description; surfaced by `list_subagents`.
    pub description: String,
    /// System prompt (user-authored).
    pub system: String,
    /// LLM model. Defaults to `MiniMax-M3` (1M context).
    #[serde(default)]
    pub model: Option<String>,
    /// Per-call LLM `max_tokens`. Defaults to 16384.
    #[serde(default)]
    pub max_tokens: Option<i32>,
    /// LLM temperature 0-1. Omit to use the API default.
    #[serde(default)]
    pub temperature: Option<f64>,
    /// Maximum agent-loop iterations. Defaults to 10.
    #[serde(default)]
    pub max_iterations: Option<u32>,
    /// Tool whitelist. `run_subagent` is always implicitly allowed.
    /// `None` means "all tools".
    #[serde(default)]
    pub allowed_tools: Option<Vec<String>>,
    /// Maximum recursion depth when this subagent calls `run_subagent`.
    /// Defaults to 3.
    #[serde(default)]
    pub max_depth: Option<u32>,
}

impl SubagentDef {
    /// Validate that required fields are present and consistent.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("name is required".into());
        }
        if self.description.trim().is_empty() {
            return Err("description is required".into());
        }
        if self.system.trim().is_empty() {
            return Err("system is required".into());
        }
        Ok(())
    }
}

/// Lightweight summary returned by `list_subagents`.
#[derive(Debug, Clone, Serialize)]
pub struct SubagentSummary {
    pub name: String,
    pub description: String,
}

/// Result returned by `run_subagent`.
///
/// `tool_calls` records the full trajectory: every tool the LLM decided
/// to invoke, in order, including recursive `run_subagent` invocations.
#[derive(Debug, Clone, Serialize)]
pub struct LoopResult {
    pub subagent: String,
    pub final_output: String,
    pub iterations: u32,
    pub depth: u32,
    pub tool_calls: Vec<ToolCallRecord>,
    /// Set when the loop hit `max_iterations` without reaching `end_turn`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// Record of one tool invocation inside the agent loop.
#[derive(Debug, Clone, Serialize)]
pub struct ToolCallRecord {
    /// 1-based iteration number (each LLM call is one iteration).
    pub iteration: u32,
    /// Tool name as the LLM called it.
    pub tool: String,
    /// Subagent name if `tool == "run_subagent"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent: Option<String>,
    /// Arguments the LLM passed (as JSON).
    pub input: serde_json::Value,
    /// Truncated preview of the tool's output (or error message).
    pub output_preview: String,
    /// `true` if the tool returned an error.
    pub is_error: bool,
}
