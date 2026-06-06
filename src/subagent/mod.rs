//! Subagent subsystem: registry, agent loop, tool dispatcher.
//!
//! Public surface: [`types::SubagentDef`], [`types::LoopResult`], etc.
//! Implementation modules are populated in subsequent steps.

pub mod dispatcher;
pub mod loop_runner;
pub mod registry;
pub mod tool_catalog;
pub mod types;

pub use dispatcher::{DispatchResult, ToolDispatcher};
pub use loop_runner::run_agent_loop;
pub use registry::SubagentRegistry;
pub use tool_catalog::{all_tool_specs, specs_for, RUN_SUBAGENT_NAME};
pub use types::{LoopResult, SubagentDef, SubagentSummary, ToolCallRecord};
