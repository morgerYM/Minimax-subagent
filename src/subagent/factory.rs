//! Tool factory system — inspired by OpenClaw's `registerTool(factory)` pattern.
//!
//! Instead of a hardcoded list of tool specs and a central `match` dispatcher,
//! tools are produced by **factories** that receive a [`ToolFactoryContext`].
//! Each factory returns the tools for a capability area (TTS, video, image, …).
//!
//! ```ignore
//! let mut registry = ToolRegistry::new();
//! registry.register("tts", Arc::new(|ctx| vec![
//!     AgentTool::new("text_to_audio", "…", schema, execute_fn),
//!     // …
//! ]));
//! let all_tools = registry.resolve(&ctx);
//! let sub_tools = tools_for_subagent(&all_tools, &subagent_def);
//! ```

use std::sync::Arc;

use super::agent_tool::{AgentTool, RUN_SUBAGENT_NAME};
use super::registry::SubagentRegistry;
use super::types::SubagentDef;

// ============================================================
// Factory context
// ============================================================

/// Runtime context handed to every tool factory.
///
/// Currently carries the subagent registry (needed by the `run_subagent`
/// tool itself). As more providers are added, this is where they live.
pub struct ToolFactoryContext {
    /// Registry of named subagents (loaded from `subagents/*.json`).
    pub subagent_registry: Arc<SubagentRegistry>,
    // Future: pub provider_set: Arc<ProviderSet>,
}

// ============================================================
// Factory
// ============================================================

/// A factory that produces one or more [`AgentTool`]s given a
/// [`ToolFactoryContext`].
///
/// Factories are registered by name (for debugging / override purposes)
/// and called during resolution to build the effective tool list.
pub type ToolFactory = Arc<dyn Fn(&ToolFactoryContext) -> Vec<AgentTool> + Send + Sync>;

// ============================================================
// Registry
// ============================================================

/// Registry of tool factories. Each factory contributes tools for one
/// capability area (TTS, video, image, subagent, …).
///
/// Call [`ToolRegistry::resolve`] once at startup to produce the
/// flat list of [`AgentTool`]s used by the agent loop.
pub struct ToolRegistry {
    entries: Vec<(String, ToolFactory)>,
}

impl ToolRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Register a factory under a human-readable label.
    ///
    /// The label is used for debugging and potential future override
    /// semantics — it does not affect the tool names the LLM sees.
    pub fn register(&mut self, label: impl Into<String>, factory: ToolFactory) {
        self.entries.push((label.into(), factory));
    }

    /// Register a factory from a plain `Fn` (wraps it in an `Arc`).
    pub fn register_fn(
        &mut self,
        label: impl Into<String>,
        f: impl Fn(&ToolFactoryContext) -> Vec<AgentTool> + Send + Sync + 'static,
    ) {
        self.register(label, Arc::new(f));
    }

    /// Resolve all factories against a context, producing the final
    /// flat list of self-contained tools.
    ///
    /// Call this **once** at startup and cache the result — the tool
    /// list is stable for the lifetime of the process.
    pub fn resolve(&self, ctx: &ToolFactoryContext) -> Vec<AgentTool> {
        let mut tools: Vec<AgentTool> = Vec::new();
        for (label, factory) in &self.entries {
            let produced = factory(ctx);
            tracing::debug!(
                "ToolFactory '{label}' produced {} tool(s)",
                produced.len()
            );
            tools.extend(produced);
        }
        tools
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Subagent tool filtering
// ============================================================

/// Filter a resolved tool list for a specific subagent.
///
/// - If `subagent.allowed_tools` is `None`, all tools are included.
/// - If it's a whitelist, only those tools (plus `run_subagent`) are kept.
/// - `run_subagent` is always included, even if not in the whitelist.
pub fn tools_for_subagent(
    all_tools: &[AgentTool],
    subagent: &SubagentDef,
) -> Vec<AgentTool> {
    let allowed = subagent.allowed_tools.as_ref();
    all_tools
        .iter()
        .filter(|t| {
            if t.name == RUN_SUBAGENT_NAME {
                return true;
            }
            match allowed {
                None => true,
                Some(whitelist) => whitelist.iter().any(|name| name == &t.name),
            }
        })
        .cloned()
        .collect()
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use crate::subagent::types::DispatchResult;
    use super::*;

    /// Build a minimal AgentTool for testing.
    fn fake_tool(name: &str) -> AgentTool {
        AgentTool::new(
            name,
            "test tool",
            serde_json::json!({"type": "object"}),
            Arc::new(|_input, _depth| {
                Box::pin(async { Ok(DispatchResult { output: "ok".into(), is_error: false }) })
            }),
        )
    }

    /// Build a minimal subagent def for testing.
    fn make_subagent(name: &str, allowed: Option<Vec<&str>>) -> SubagentDef {
        SubagentDef {
            name: name.to_string(),
            description: "test".to_string(),
            system: "test".to_string(),
            model: None,
            max_tokens: None,
            temperature: None,
            max_iterations: None,
            allowed_tools: allowed.map(|v| v.into_iter().map(String::from).collect()),
            max_depth: None,
        }
    }

    #[test]
    fn no_whitelist_returns_all() {
        let tools = vec![fake_tool("a"), fake_tool("b"), fake_tool("c")];
        let sub = make_subagent("any", None);
        let filtered = tools_for_subagent(&tools, &sub);
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn whitelist_filters_correctly() {
        let tools = vec![
            fake_tool("text_to_audio"),
            fake_tool("generate_image"),
            fake_tool("generate_video"),
            fake_tool(RUN_SUBAGENT_NAME),
        ];
        let sub = make_subagent(
            "limited",
            Some(vec!["text_to_audio", "generate_image"]),
        );
        let filtered = tools_for_subagent(&tools, &sub);
        let names: Vec<&str> = filtered.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"text_to_audio"));
        assert!(names.contains(&"generate_image"));
        assert!(!names.contains(&"generate_video"));
        // run_subagent is always present
        assert!(names.contains(&RUN_SUBAGENT_NAME));
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn empty_whitelist_still_allows_run_subagent() {
        let tools = vec![fake_tool("text_to_audio"), fake_tool(RUN_SUBAGENT_NAME)];
        let sub = make_subagent("none", Some(vec![]));
        let filtered = tools_for_subagent(&tools, &sub);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, RUN_SUBAGENT_NAME);
    }

    #[test]
    fn unknown_tool_name_in_whitelist_is_silently_ignored() {
        let tools = vec![fake_tool("text_to_audio"), fake_tool(RUN_SUBAGENT_NAME)];
        let sub = make_subagent(
            "typo",
            Some(vec!["text_to_audio", "fake_tool_doesnt_exist"]),
        );
        let filtered = tools_for_subagent(&tools, &sub);
        let names: Vec<&str> = filtered.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"text_to_audio"));
        assert!(!names.contains(&"fake_tool_doesnt_exist"));
        assert!(names.contains(&RUN_SUBAGENT_NAME));
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn tool_spec_schemas_are_objects() {
        // Smoke-test schema validity using real param types
        use crate::mcp_params::*;
        use crate::subagent::agent_tool::schema_of;

        // Schemas for a representative sample of tools
        let schemas = vec![
            schema_of::<TextToAudioParams>(),
            schema_of::<GenerateVideoParams>(),
            schema_of::<GenerateImageParams>(),
            schema_of::<GenerateMusicParams>(),
            schema_of::<ChatParams>(),
            schema_of::<WebSearchParams>(),
            schema_of::<RunSubagentParams>(),
            schema_of::<crate::subagent::agent_tool::NoParams>(),
        ];

        for (i, s) in schemas.iter().enumerate() {
            assert!(
                s.is_object(),
                "schema[{}] is not a JSON object: {:?}",
                i,
                s
            );
        }
    }

    #[test]
    fn run_subagent_params_schema_includes_allowed_tools() {
        use crate::mcp_params::RunSubagentParams;
        use crate::subagent::agent_tool::schema_of;

        let schema = schema_of::<RunSubagentParams>();
        assert!(schema.is_object(), "schema is not an object");
        let props = schema.get("properties").and_then(|p| p.as_object());
        assert!(props.is_some(), "schema has no 'properties'");
        let props = props.unwrap();
        assert!(props.contains_key("name"), "missing 'name'");
        assert!(props.contains_key("task"), "missing 'task'");
        assert!(props.contains_key("allowed_tools"), "missing 'allowed_tools'");
        let at = &props["allowed_tools"];
        let ty = at.get("type");
        assert!(
            ty == Some(&serde_json::json!("array"))
                || ty == Some(&serde_json::json!(["array", "null"])),
            "allowed_tools type should be array (possibly nullable): got {:?}",
            ty
        );
    }
}
