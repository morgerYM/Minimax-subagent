//! Subagent subsystem: registry, agent loop, self-contained tools.
//!
//! Public surface: [`types::SubagentDef`], [`types::LoopResult`], etc.
//! Tools follow the OpenClaw AnyAgentTool pattern: each [`agent_tool::AgentTool`]
//! bundles schema + execution, eliminating the central `match` dispatcher.

pub mod agent_tool;
pub mod factory;
pub mod loop_runner;
pub mod registry;
pub mod types;

pub use agent_tool::{
    call_tool_result_to_dispatch, parse_input, schema_of, to_tool_err, AgentTool, DispatchResult,
    ExecuteFn, NoParams, RUN_SUBAGENT_NAME,
};
pub use factory::{tools_for_subagent, ToolFactory, ToolFactoryContext, ToolRegistry};
pub use loop_runner::run_agent_loop;
pub use registry::SubagentRegistry;
pub use types::{LoopResult, SubagentDef, SubagentSummary, ToolCallRecord};
