//! The agent loop: drives LLM ↔ tool_use interactions until the LLM
//! signals `end_turn` (or the loop hits a configured bound).
//!
//! Unlike the old design (central `ToolDispatcher` + `match`), this loop
//! operates on a pre-resolved list of self-contained [`AgentTool`]s.
//! Each tool knows how to execute itself — the loop just calls
//! `tool.execute(tool_input, depth)`.

use crate::error::MiniMaxError;
use crate::types::{AgentChatRequest, AgentContent, AgentContentBlock, AgentMessage};
use crate::MiniMaxClient;

use super::agent_tool::{AgentTool, RUN_SUBAGENT_NAME};
use super::types::{LoopResult, SubagentDef, ToolCallRecord};

const DEFAULT_MAX_ITERATIONS: u32 = 10;
const DEFAULT_MAX_TOKENS: i32 = 16384;
const DEFAULT_MAX_DEPTH: u32 = 3;
const DEFAULT_MODEL: &str = "MiniMax-M3";
const OUTPUT_PREVIEW_CHARS: usize = 300;

/// Run one subagent from initial task to completion.
///
/// `tools` must already be filtered for the subagent (via
/// [`tools_for_subagent`](super::factory::tools_for_subagent)).
///
/// Returns the final text output, the full tool-call history, the
/// iteration count, and the recursion depth (0 for top-level calls).
pub async fn run_agent_loop(
    client: &MiniMaxClient,
    subagent: &SubagentDef,
    task: &str,
    depth: u32,
    tools: &[AgentTool],
) -> Result<LoopResult, MiniMaxError> {
    let max_iterations = subagent.max_iterations.unwrap_or(DEFAULT_MAX_ITERATIONS);
    let max_depth = subagent.max_depth.unwrap_or(DEFAULT_MAX_DEPTH);
    let model = subagent
        .model
        .clone()
        .unwrap_or_else(|| DEFAULT_MODEL.to_string());
    let max_tokens = subagent.max_tokens.or(Some(DEFAULT_MAX_TOKENS));

    // Depth guard — refuse to recurse beyond the subagent's max_depth.
    if depth > max_depth {
        return Err(MiniMaxError::Config(format!(
            "subagent '{}' exceeded max_depth={} (current depth={})",
            subagent.name, max_depth, depth
        )));
    }

    // Build the ToolSpec list for the LLM API (schema only, no execute).
    let api_tools: Vec<crate::types::ToolSpec> =
        tools.iter().map(|t| t.to_spec()).collect();

    let mut messages: Vec<AgentMessage> = vec![AgentMessage {
        role: "user".to_string(),
        content: AgentContent::Text(task.to_string()),
    }];
    let mut tool_calls: Vec<ToolCallRecord> = Vec::new();
    let mut final_output = String::new();
    let mut warning: Option<String> = None;
    let mut iterations: u32 = 0;

    for _ in 0..max_iterations {
        iterations += 1;

        let req = AgentChatRequest {
            model: model.clone(),
            messages: messages.clone(),
            system: Some(subagent.system.clone()),
            max_tokens,
            temperature: subagent.temperature,
            top_p: None,
            stream: false,
            tools: Some(api_tools.clone()),
        };

        let response = client.chat_agent(&req).await?;

        // Parse response blocks
        let mut text_parts: Vec<String> = Vec::new();
        let mut tool_uses: Vec<ToolUseCall> = Vec::new();

        for block in &response.content {
            match block.block_type.as_str() {
                "text" => {
                    if let Some(t) = &block.text {
                        text_parts.push(t.clone());
                    }
                }
                "tool_use" => {
                    if let (Some(id), Some(name), Some(input)) =
                        (&block.id, &block.name, &block.input)
                    {
                        tool_uses.push(ToolUseCall {
                            id: id.clone(),
                            name: name.clone(),
                            input: input.clone(),
                        });
                    }
                }
                _ => {
                    // Unknown block type — ignore
                }
            }
        }

        // No tool_use blocks → end_turn, take the text
        if tool_uses.is_empty() {
            final_output = text_parts.join("\n");
            break;
        }

        // Build the assistant message that includes both text and tool_use blocks
        let mut assistant_blocks: Vec<AgentContentBlock> = Vec::new();
        for t in &text_parts {
            assistant_blocks.push(AgentContentBlock::Text { text: t.clone() });
        }
        for tu in &tool_uses {
            assistant_blocks.push(AgentContentBlock::ToolUse {
                id: tu.id.clone(),
                name: tu.name.clone(),
                input: tu.input.clone(),
            });
        }
        messages.push(AgentMessage {
            role: "assistant".to_string(),
            content: AgentContent::Blocks(assistant_blocks),
        });

        // Dispatch each tool call via the tool's own execute method
        let mut result_blocks: Vec<AgentContentBlock> = Vec::new();
        for tu in &tool_uses {
            // Find the tool by name (every subagent loop has tools filtered
            // via tools_for_subagent, so the name should always exist).
            let tool = tools.iter().find(|t| t.name == tu.name).ok_or_else(|| {
                MiniMaxError::Config(format!(
                    "unknown tool '{}' — not in subagent tool catalog",
                    tu.name
                ))
            })?;

            let dispatch_result = (tool.execute)(tu.input.clone(), depth).await?;

            let subagent_name_for_record = if tu.name == RUN_SUBAGENT_NAME {
                tu.input
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            } else {
                None
            };

            tool_calls.push(ToolCallRecord {
                iteration: iterations,
                tool: tu.name.clone(),
                subagent: subagent_name_for_record,
                input: tu.input.clone(),
                output_preview: truncate(&dispatch_result.output, OUTPUT_PREVIEW_CHARS),
                is_error: dispatch_result.is_error,
            });

            result_blocks.push(AgentContentBlock::ToolResult {
                tool_use_id: tu.id.clone(),
                content: dispatch_result.output,
                is_error: dispatch_result.is_error,
            });
        }
        messages.push(AgentMessage {
            role: "user".to_string(),
            content: AgentContent::Blocks(result_blocks),
        });
    }

    // If we exited the loop without end_turn, the LLM never said it was done
    if final_output.is_empty() {
        warning = Some(format!(
            "subagent '{}' hit max_iterations={} without end_turn",
            subagent.name, max_iterations
        ));
        final_output = format!(
            "[max_iterations={} reached; see tool_calls for partial progress]",
            max_iterations
        );
    }

    Ok(LoopResult {
        subagent: subagent.name.clone(),
        final_output,
        iterations,
        depth,
        tool_calls,
        warning,
    })
}

struct ToolUseCall {
    id: String,
    name: String,
    input: serde_json::Value,
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{}…[truncated]", truncated)
    }
}
