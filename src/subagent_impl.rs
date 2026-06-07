//! Binary-side [`ToolDispatcher`] implementation.
//!
//! Routes each tool call to either:
//!   1. An existing MCP tool handler (via provider traits)
//!   2. A recursive `run_subagent` call
//!   3. An `unknown tool` error

use std::sync::Arc;

use async_trait::async_trait;
use minimax_api::mcp_params::*;
use minimax_api::providers::MiniMaxProvider;
use minimax_api::subagent::{
    run_agent_loop, DispatchResult, LoopResult, SubagentDef, SubagentRegistry, ToolDispatcher,
    RUN_SUBAGENT_NAME,
};
use rmcp::model::{CallToolResult, RawContent};
use serde_json::Value;

use minimax_api::error::MiniMaxError;

use minimax_api::tools::{chat, files, image, music, search, tts, usage, video};

// ============================================================
// Dispatcher implementation
// ============================================================

pub struct McpToolDispatcher {
    pub provider: MiniMaxProvider,
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

        let p = &self.provider;

        match tool_name {
            // ---------- TTS / Voice ----------
            "text_to_audio" => {
                let params: TextToAudioParams = parse(tool_input)?;
                let r = tts::handle_text_to_audio(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "text_to_audio_stream" => {
                let params: TextToAudioStreamParams = parse(tool_input)?;
                let r = tts::handle_text_to_audio_stream(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "generate_audio_async" => {
                let params: GenerateAudioAsyncParams = parse(tool_input)?;
                let r = tts::handle_generate_audio_async(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "query_audio_task" => {
                let params: QueryAudioTaskParams = parse(tool_input)?;
                let r = tts::handle_query_audio_task(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "list_voices" => {
                let params: ListVoicesParams = parse(tool_input)?;
                let r = tts::handle_list_voices(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "voice_clone" => {
                let params: VoiceCloneParams = parse(tool_input)?;
                let r = tts::handle_voice_clone(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "voice_design" => {
                let params: VoiceDesignParams = parse(tool_input)?;
                let r = tts::handle_voice_design(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "delete_voice" => {
                let params: DeleteVoiceParams = parse(tool_input)?;
                let r = tts::handle_delete_voice(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }

            // ---------- Image ----------
            "generate_image" => {
                let params: GenerateImageParams = parse(tool_input)?;
                let r = image::handle_generate_image(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "understand_image" => {
                let params: UnderstandImageParams = parse(tool_input)?;
                let r = search::handle_understand_image(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }

            // ---------- Video ----------
            "generate_video" => {
                let params: GenerateVideoParams = parse(tool_input)?;
                let r = video::handle_generate_video(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "query_video" => {
                let params: QueryVideoParams = parse(tool_input)?;
                let r = video::handle_query_video(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "generate_video_agent" => {
                let params: GenerateVideoAgentParams = parse(tool_input)?;
                let r = video::handle_generate_video_agent(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "query_video_agent" => {
                let params: QueryVideoAgentParams = parse(tool_input)?;
                let r = video::handle_query_video_agent(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }

            // ---------- Music ----------
            "generate_music" => {
                let params: GenerateMusicParams = parse(tool_input)?;
                let r = music::handle_generate_music(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "generate_lyrics" => {
                let params: GenerateLyricsParams = parse(tool_input)?;
                let r = music::handle_generate_lyrics(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "generate_music_cover" => {
                let params: GenerateMusicCoverParams = parse(tool_input)?;
                let r = music::handle_generate_music_cover(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }

            // ---------- Chat / Search ----------
            "chat" => {
                let params: ChatParams = parse(tool_input)?;
                let r = chat::handle_chat(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "web_search" => {
                let params: WebSearchParams = parse(tool_input)?;
                let r = search::handle_web_search(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }

            // ---------- Files ----------
            "list_files" => {
                let params: ListFilesParams = parse(tool_input)?;
                let r = files::handle_list_files(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "retrieve_file" => {
                let params: RetrieveFileParams = parse(tool_input)?;
                let r = files::handle_retrieve_file(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }
            "delete_file" => {
                let params: DeleteFileParams = parse(tool_input)?;
                let r = files::handle_delete_file(p, params).await.map_err(mcp_err)?;
                Ok(text_result(r))
            }

            // ---------- Account ----------
            "query_usage" => {
                let r = usage::handle_query_usage(p).await.map_err(mcp_err)?;
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
        let params: RunSubagentParams = serde_json::from_value(tool_input)
            .map_err(|e| MiniMaxError::Config(format!("run_subagent: invalid params: {e}")))?;

        let sub: &SubagentDef = self.registry.get(&params.name).ok_or_else(|| {
            MiniMaxError::Config(format!(
                "subagent '{}' not found. Use list_subagents to see available subagents.",
                params.name
            ))
        })?;

        let result: LoopResult =
            run_agent_loop(&self.provider.client, sub, &params.task, current_depth + 1, self).await?;
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
    DispatchResult { output, is_error: false }
}

fn mcp_err(e: rmcp::ErrorData) -> MiniMaxError {
    MiniMaxError::Config(format!("tool error: {}", e))
}
