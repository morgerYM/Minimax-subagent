//! Binary-side [`ToolDispatcher`] implementation.
//!
//! Routes each tool call to either:
//!   1. An existing MCP tool handler (e.g. `handle_text_to_audio`)
//!   2. A recursive `run_subagent` call (re-enters the agent loop
//!      with `depth + 1` and `self` as the dispatcher)
//!   3. An `unknown tool` error for anything not in the catalog

use std::sync::Arc;

use async_trait::async_trait;
use minimax_api::mcp_params::*;
use minimax_api::subagent::{
    run_agent_loop, DispatchResult, LoopResult, SubagentDef, SubagentRegistry, ToolDispatcher,
    RUN_SUBAGENT_NAME,
};
use minimax_api::MiniMaxClient;
use rmcp::model::{CallToolResult, RawContent};
use serde_json::Value;

use minimax_api::error::MiniMaxError;

use crate::tools;

// ============================================================
// Dispatcher implementation
// ============================================================

pub struct McpToolDispatcher {
    pub client: MiniMaxClient,
    pub registry: Arc<SubagentRegistry>,
}

#[async_trait]
impl ToolDispatcher for McpToolDispatcher {
    async fn dispatch(
        &self,
        tool_name: &str,
        tool_input: Value,
        current_depth: u32,
    ) -> Result<DispatchResult, MiniMaxError> {
        // Special-case: run_subagent recurses into the agent loop
        if tool_name == RUN_SUBAGENT_NAME {
            return self.dispatch_run_subagent(tool_input, current_depth).await;
        }

        match tool_name {
            // ---------- TTS / Voice ----------
            "text_to_audio" => {
                let p: TextToAudioParams = parse(tool_input)?;
                let r = tools::tts::handle_text_to_audio(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "text_to_audio_stream" => {
                let p: TextToAudioStreamParams = parse(tool_input)?;
                let r = tools::tts::handle_text_to_audio_stream(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "generate_audio_async" => {
                let p: GenerateAudioAsyncParams = parse(tool_input)?;
                let r = tools::tts::handle_generate_audio_async(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "query_audio_task" => {
                let p: QueryAudioTaskParams = parse(tool_input)?;
                let r = tools::tts::handle_query_audio_task(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "list_voices" => {
                let p: ListVoicesParams = parse(tool_input)?;
                let r = tools::tts::handle_list_voices(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "voice_clone" => {
                let p: VoiceCloneParams = parse(tool_input)?;
                let r = tools::tts::handle_voice_clone(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "voice_design" => {
                let p: VoiceDesignParams = parse(tool_input)?;
                let r = tools::tts::handle_voice_design(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "delete_voice" => {
                let p: DeleteVoiceParams = parse(tool_input)?;
                let r = tools::tts::handle_delete_voice(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }

            // ---------- Image ----------
            "generate_image" => {
                let p: GenerateImageParams = parse(tool_input)?;
                let r = tools::image::handle_generate_image(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "understand_image" => {
                let p: UnderstandImageParams = parse(tool_input)?;
                let r = tools::search::handle_understand_image(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }

            // ---------- Video ----------
            "generate_video" => {
                let p: GenerateVideoParams = parse(tool_input)?;
                let r = tools::video::handle_generate_video(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "query_video" => {
                let p: QueryVideoParams = parse(tool_input)?;
                let r = tools::video::handle_query_video(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "generate_video_agent" => {
                let p: GenerateVideoAgentParams = parse(tool_input)?;
                let r = tools::video::handle_generate_video_agent(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "query_video_agent" => {
                let p: QueryVideoAgentParams = parse(tool_input)?;
                let r = tools::video::handle_query_video_agent(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }

            // ---------- Music ----------
            "generate_music" => {
                let p: GenerateMusicParams = parse(tool_input)?;
                let r = tools::music::handle_generate_music(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "generate_lyrics" => {
                let p: GenerateLyricsParams = parse(tool_input)?;
                let r = tools::music::handle_generate_lyrics(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "generate_music_cover" => {
                let p: GenerateMusicCoverParams = parse(tool_input)?;
                let r = tools::music::handle_generate_music_cover(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }

            // ---------- Chat / Search ----------
            "chat" => {
                let p: ChatParams = parse(tool_input)?;
                let r = tools::chat::handle_chat(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "web_search" => {
                let p: WebSearchParams = parse(tool_input)?;
                let r = tools::search::handle_web_search(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }

            // ---------- Files ----------
            "list_files" => {
                let p: ListFilesParams = parse(tool_input)?;
                let r = tools::files::handle_list_files(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "retrieve_file" => {
                let p: RetrieveFileParams = parse(tool_input)?;
                let r = tools::files::handle_retrieve_file(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "delete_file" => {
                let p: DeleteFileParams = parse(tool_input)?;
                let r = tools::files::handle_delete_file(&self.client, p)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }

            // ---------- Account ----------
            "query_usage" => {
                let r = tools::usage::handle_query_usage(&self.client)
                    .await
                    .map_err(mcp_err)?;
                Ok(text_result(r))
            }

            // ---------- Fallback ----------
            other => Err(MiniMaxError::Config(format!(
                "unknown tool '{}' (not in subagent catalog)",
                other
            ))),
        }
    }
}

impl McpToolDispatcher {
    async fn dispatch_run_subagent(
        &self,
        tool_input: Value,
        current_depth: u32,
    ) -> Result<DispatchResult, MiniMaxError> {
        let p: RunSubagentParams = serde_json::from_value(tool_input)
            .map_err(|e| MiniMaxError::Config(format!("run_subagent: invalid params: {e}")))?;

        let sub: &SubagentDef = self.registry.get(&p.name).ok_or_else(|| {
            MiniMaxError::Config(format!(
                "subagent '{}' not found. Use list_subagents to see available subagents.",
                p.name
            ))
        })?;

        let result: LoopResult =
            run_agent_loop(&self.client, sub, &p.task, current_depth + 1, self).await?;
        Ok(DispatchResult {
            output: result.final_output,
            is_error: false,
        })
    }
}

// ============================================================
// Helpers
// ============================================================

fn parse<T: serde::de::DeserializeOwned>(input: Value) -> Result<T, MiniMaxError> {
    serde_json::from_value(input)
        .map_err(|e| MiniMaxError::Config(format!("param parse error: {e}")))
}

/// Extract the text content from a `CallToolResult`. All existing
/// handlers return `CallToolResult::success(vec![Content::text(...)])`,
/// so this collects those text bodies into a single newline-joined string.
pub fn text_result(r: CallToolResult) -> DispatchResult {
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

fn mcp_err(e: rmcp::ErrorData) -> MiniMaxError {
    MiniMaxError::Config(format!("tool error: {}", e))
}
