//! TTS tool handlers.

use async_trait::async_trait;

use crate::mcp_params::{
    DeleteVoiceParams, GenerateAudioAsyncParams, ListVoicesParams, QueryAudioTaskParams,
    TextToAudioParams, TextToAudioStreamParams, VoiceCloneParams, VoiceDesignParams,
};
use crate::providers::{
    AsyncTaskHandle, AsyncTaskResult, MediaOutput, ProviderError, VoiceCloneResult,
    VoiceListResult,
};
use crate::utils;

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

// ============================================================
// Traits
// ============================================================

#[async_trait]
pub trait TtsProvider: Send + Sync {
    async fn text_to_audio(&self, params: &TextToAudioParams) -> Result<MediaOutput, ProviderError>;
    async fn text_to_audio_stream(&self, params: &TextToAudioStreamParams) -> Result<MediaOutput, ProviderError>;
    async fn submit_async_tts(&self, params: &GenerateAudioAsyncParams) -> Result<AsyncTaskHandle, ProviderError>;
    async fn query_async_tts(&self, task_id: &str) -> Result<AsyncTaskResult, ProviderError>;
}

#[async_trait]
pub trait VoiceProvider: Send + Sync {
    async fn list_voices(&self, voice_type: Option<&str>) -> Result<VoiceListResult, ProviderError>;
    async fn voice_clone(&self, params: &VoiceCloneParams) -> Result<VoiceCloneResult, ProviderError>;
    async fn voice_design(&self, params: &VoiceDesignParams) -> Result<MediaOutput, ProviderError>;
    async fn delete_voice(&self, voice_type: &str, voice_id: &str) -> Result<(), ProviderError>;
}

// ============================================================
// Handlers
// ============================================================

fn to_mcp_err(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

// --- TtsProvider handlers ---

pub async fn handle_text_to_audio(
    provider: &dyn TtsProvider,
    params: TextToAudioParams,
) -> Result<CallToolResult, ErrorData> {
    let output = provider.text_to_audio(&params).await.map_err(to_mcp_err)?;
    match output {
        MediaOutput::Bytes { data, extension } => {
            if params.output_directory.is_some() || params.output_file.is_some() {
                if let Some(path) = utils::write_output_file(
                    params.output_file.as_deref(),
                    params.output_directory.as_deref(),
                    "text_to_audio",
                    &params.text,
                    &extension,
                    &data,
                )
                .await
                .map_err(to_mcp_err)?
                {
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "Saved to: {}",
                        path.display()
                    ))]));
                }
            }
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Audio generated! {} bytes",
                data.len()
            ))]))
        }
        MediaOutput::Url(url) => Ok(CallToolResult::success(vec![Content::text(url)])),
    }
}

pub async fn handle_text_to_audio_stream(
    provider: &dyn TtsProvider,
    params: TextToAudioStreamParams,
) -> Result<CallToolResult, ErrorData> {
    let output = provider.text_to_audio_stream(&params).await.map_err(to_mcp_err)?;
    match output {
        MediaOutput::Bytes { data, .. } => {
            if params.output_directory.is_some() || params.output_file.is_some() {
                if let Some(path) = utils::write_output_file(
                    params.output_file.as_deref(),
                    params.output_directory.as_deref(),
                    "stream_tts",
                    &params.text,
                    "mp3",
                    &data,
                )
                .await
                .map_err(to_mcp_err)?
                {
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "Stream TTS done! Saved to: {} ({} bytes)",
                        path.display(),
                        data.len()
                    ))]));
                }
            }
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Stream TTS done! Audio size: {} bytes ({} KB)",
                data.len(),
                data.len() / 1024
            ))]))
        }
        MediaOutput::Url(url) => Ok(CallToolResult::success(vec![Content::text(url)])),
    }
}

pub async fn handle_generate_audio_async(
    provider: &dyn TtsProvider,
    params: GenerateAudioAsyncParams,
) -> Result<CallToolResult, ErrorData> {
    let handle = provider.submit_async_tts(&params).await.map_err(to_mcp_err)?;

    let json = serde_json::to_string_pretty(&serde_json::json!({
        "task_id": handle.task_id,
        "output_directory": params.output_directory,
        "output_file": params.output_file,
        "message": format!(
            "Async TTS task submitted. Use query_audio_task --task_id {} to poll and download.",
            handle.task_id
        ),
    }))
    .map_err(to_mcp_err)?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

pub async fn handle_query_audio_task(
    provider: &dyn TtsProvider,
    params: QueryAudioTaskParams,
) -> Result<CallToolResult, ErrorData> {
    let result = provider.query_async_tts(&params.task_id).await.map_err(to_mcp_err)?;

    match result {
        AsyncTaskResult::Completed(output) => match output {
            MediaOutput::Bytes { data, .. } => {
                if params.output_directory.is_some() || params.output_file.is_some() {
                    if let Some(path) = utils::write_output_file(
                        params.output_file.as_deref(),
                        params.output_directory.as_deref(),
                        "async_tts",
                        &params.task_id,
                        "mp3",
                        &data,
                    )
                    .await
                    .map_err(to_mcp_err)?
                    {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Async TTS done! Saved to: {} ({} bytes)",
                            path.display(),
                            data.len()
                        ))]));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Async TTS done! Audio size: {} bytes ({} KB)",
                    data.len(),
                    data.len() / 1024
                ))]))
            }
            MediaOutput::Url(url) => Ok(CallToolResult::success(vec![Content::text(url)])),
        },
        AsyncTaskResult::Pending { status } => {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Task status: {status}\ntask_id: {}",
                params.task_id
            ))]))
        }
        AsyncTaskResult::Failed { message } => {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Task failed: {message}\ntask_id: {}",
                params.task_id
            ))]))
        }
    }
}

// --- VoiceProvider handlers ---

fn format_voice_list(resp: &VoiceListResult) -> String {
    let mut lines = Vec::new();
    lines.push("=== System Voices ===".to_string());
    for v in &resp.system {
        lines.push(format!("  {} — {}", v.voice_id, v.voice_name));
    }
    lines.push("=== Cloned Voices ===".to_string());
    for v in &resp.cloned {
        lines.push(format!("  {} — {}", v.voice_id, v.voice_name));
    }
    lines.push("=== Designed Voices ===".to_string());
    for v in &resp.designed {
        lines.push(format!("  {} — {}", v.voice_id, v.description));
    }
    lines.push(format!(
        "{} system voices + {} cloned voices + {} designed voices",
        resp.system.len(),
        resp.cloned.len(),
        resp.designed.len()
    ));
    lines.join("\n")
}

pub async fn handle_list_voices(
    provider: &dyn VoiceProvider,
    params: ListVoicesParams,
) -> Result<CallToolResult, ErrorData> {
    let resp = provider.list_voices(params.voice_type.as_deref()).await.map_err(to_mcp_err)?;
    Ok(CallToolResult::success(vec![Content::text(format_voice_list(&resp))]))
}

pub async fn handle_voice_clone(
    provider: &dyn VoiceProvider,
    params: VoiceCloneParams,
) -> Result<CallToolResult, ErrorData> {
    let resp = provider.voice_clone(&params).await.map_err(to_mcp_err)?;

    let mut msg = format!("Voice cloned!\nvoice_id: {}", resp.voice_id);
    match &resp.demo_audio {
        Some(MediaOutput::Url(url)) => {
            msg.push_str(&format!("\ndemo_audio: {url}"));
        }
        Some(MediaOutput::Bytes { data, .. }) => {
            if params.output_directory.is_some() || params.output_file.is_some() {
                if let Some(path) = utils::write_output_file(
                    params.output_file.as_deref(),
                    params.output_directory.as_deref(),
                    "voice_clone",
                    "voice",
                    "wav",
                    data,
                )
                .await
                .map_err(to_mcp_err)?
                {
                    msg.push_str(&format!("\nSaved to: {}", path.display()));
                }
            } else {
                msg.push_str(&format!("\ndemo_audio: {} bytes", data.len()));
            }
        }
        None => {
            msg.push_str("\ndemo_audio: N/A");
        }
    }
    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

pub async fn handle_voice_design(
    provider: &dyn VoiceProvider,
    params: VoiceDesignParams,
) -> Result<CallToolResult, ErrorData> {
    let output = provider.voice_design(&params).await.map_err(to_mcp_err)?;

    match output {
        MediaOutput::Bytes { data, .. } => {
            if params.output_directory.is_some() || params.output_file.is_some() {
                if let Some(path) = utils::write_output_file(
                    params.output_file.as_deref(),
                    params.output_directory.as_deref(),
                    "voice_design",
                    &params.preview_text,
                    "mp3",
                    &data,
                )
                .await
                .map_err(to_mcp_err)?
                {
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "Voice design done!\nSaved to: {}",
                        path.display()
                    ))]));
                }
            }
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Voice design done! Audio size: {} bytes",
                data.len()
            ))]))
        }
        MediaOutput::Url(url) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Voice design done!\nURL: {url}"
        ))]))
    }
}

pub async fn handle_delete_voice(
    provider: &dyn VoiceProvider,
    params: DeleteVoiceParams,
) -> Result<CallToolResult, ErrorData> {
    provider
        .delete_voice(&params.voice_type, &params.voice_id)
        .await
        .map_err(to_mcp_err)?;
    Ok(CallToolResult::success(vec![Content::text(format!(
        "Voice {} deleted.",
        params.voice_id
    ))]))
}
