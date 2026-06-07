//! Video generation tool handlers.

use async_trait::async_trait;

use crate::mcp_params::{GenerateVideoAgentParams, GenerateVideoParams, QueryVideoAgentParams, QueryVideoParams};
use crate::providers::{AsyncTaskHandle, AsyncTaskResult, MediaOutput, ProviderError};
use crate::utils;

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

// ============================================================
// Trait
// ============================================================

#[async_trait]
pub trait VideoProvider: Send + Sync {
    async fn create_video(&self, params: &GenerateVideoParams) -> Result<AsyncTaskHandle, ProviderError>;
    async fn query_video(&self, task_id: &str) -> Result<AsyncTaskResult, ProviderError>;
    async fn generate_video_and_wait(&self, params: &GenerateVideoParams) -> Result<MediaOutput, ProviderError>;
    async fn create_video_agent(&self, params: &GenerateVideoAgentParams) -> Result<AsyncTaskHandle, ProviderError>;
    async fn query_video_agent(&self, task_id: &str) -> Result<AsyncTaskResult, ProviderError>;
}

// ============================================================
// Handlers
// ============================================================

fn to_mcp_err(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

pub async fn handle_generate_video(
    provider: &dyn VideoProvider,
    params: GenerateVideoParams,
) -> Result<CallToolResult, ErrorData> {
    let async_mode = params.async_mode.unwrap_or(true);

    if async_mode {
        let handle = provider.create_video(&params).await.map_err(to_mcp_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Video task submitted!\ntask_id: {}\nUse query_video to poll progress.",
            handle.task_id
        ))]))
    } else {
        let output = provider.generate_video_and_wait(&params).await.map_err(to_mcp_err)?;
        match output {
            MediaOutput::Bytes { data, .. } => {
                if params.output_directory.is_some() || params.output_file.is_some() {
                    if let Some(path) = utils::write_output_file(
                        params.output_file.as_deref(),
                        params.output_directory.as_deref(),
                        "video",
                        &params.prompt,
                        "mp4",
                        &data,
                    )
                    .await
                    .map_err(to_mcp_err)?
                    {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Video generated! Saved to: {}",
                            path.display()
                        ))]));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Video generated! Size: {:.1} MB",
                    data.len() as f64 / 1_048_576.0
                ))]))
            }
            MediaOutput::Url(url) => {
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Video generated! URL: {url}"
                ))]))
            }
        }
    }
}

pub async fn handle_query_video(
    provider: &dyn VideoProvider,
    params: QueryVideoParams,
) -> Result<CallToolResult, ErrorData> {
    let result = provider.query_video(&params.task_id).await.map_err(to_mcp_err)?;

    match result {
        AsyncTaskResult::Completed(output) => match output {
            MediaOutput::Bytes { data, .. } => {
                if params.output_directory.is_some() || params.output_file.is_some() {
                    if let Some(path) = utils::write_output_file(
                        params.output_file.as_deref(),
                        params.output_directory.as_deref(),
                        "video",
                        &params.task_id,
                        "mp4",
                        &data,
                    )
                    .await
                    .map_err(to_mcp_err)?
                    {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Video generated! Saved to: {}",
                            path.display()
                        ))]));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Video generated! Size: {:.1} MB",
                    data.len() as f64 / 1_048_576.0
                ))]))
            }
            MediaOutput::Url(url) => {
                if params.output_directory.is_some() || params.output_file.is_some() {
                    // download URL to path
                    if let Some(path) = utils::resolve_output_file(
                        params.output_file.as_deref(),
                        params.output_directory.as_deref(),
                        "video",
                        &params.task_id,
                        "mp4",
                    )
                    .map_err(to_mcp_err)?
                    {
                        let bytes = reqwest::get(&url).await.map_err(|e| {
                            ErrorData::internal_error(format!("download: {e}"), None)
                        })?.bytes().await.map_err(|e| {
                            ErrorData::internal_error(format!("download: {e}"), None)
                        })?;
                        tokio::fs::write(&path, &bytes).await.map_err(to_mcp_err)?;
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Video generated! Saved to: {}",
                            path.display()
                        ))]));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Video generated!\nURL: {url}"
                ))]))
            }
        },
        AsyncTaskResult::Pending { status } => Ok(CallToolResult::success(vec![Content::text(
            format!("Video task status: {}\ntask_id: {}", status, params.task_id)
        )])),
        AsyncTaskResult::Failed { message } => Ok(CallToolResult::success(vec![Content::text(
            format!("Video generation failed: {message}")
        )])),
    }
}

pub async fn handle_generate_video_agent(
    provider: &dyn VideoProvider,
    params: GenerateVideoAgentParams,
) -> Result<CallToolResult, ErrorData> {
    let handle = provider.create_video_agent(&params).await.map_err(to_mcp_err)?;

    let mut msg = format!(
        "Video Agent task submitted!\ntask_id: {}\nUse query_video_agent to poll progress.",
        handle.task_id
    );
    if params.output_directory.is_some() || params.output_file.is_some() {
        msg.push_str(&format!(
            "\n(output_directory: {:?}, output_file: {:?} — echoed for reference; actual save happens in the query step.)",
            params.output_directory, params.output_file
        ));
    }

    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

pub async fn handle_query_video_agent(
    provider: &dyn VideoProvider,
    params: QueryVideoAgentParams,
) -> Result<CallToolResult, ErrorData> {
    let result = provider.query_video_agent(&params.task_id).await.map_err(to_mcp_err)?;

    match result {
        AsyncTaskResult::Completed(output) => match output {
            MediaOutput::Bytes { data, .. } => {
                if params.output_directory.is_some() || params.output_file.is_some() {
                    if let Some(path) = utils::write_output_file(
                        params.output_file.as_deref(),
                        params.output_directory.as_deref(),
                        "video_agent",
                        &params.task_id,
                        "mp4",
                        &data,
                    )
                    .await
                    .map_err(to_mcp_err)?
                    {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Video Agent task succeeded!\ntask_id: {}\nSaved to: {}",
                            params.task_id,
                            path.display()
                        ))]));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Video Agent task succeeded!\ntask_id: {}\nSize: {:.1} MB",
                    params.task_id,
                    data.len() as f64 / 1_048_576.0
                ))]))
            }
            MediaOutput::Url(url) => {
                if (params.output_directory.is_some() || params.output_file.is_some()) && !url.is_empty() {
                    if let Some(path) = utils::resolve_output_file(
                        params.output_file.as_deref(),
                        params.output_directory.as_deref(),
                        "video_agent",
                        &params.task_id,
                        "mp4",
                    )
                    .map_err(to_mcp_err)?
                    {
                        let bytes = reqwest::get(&url).await.map_err(|e| {
                            ErrorData::internal_error(format!("download: {e}"), None)
                        })?.bytes().await.map_err(|e| {
                            ErrorData::internal_error(format!("download: {e}"), None)
                        })?;
                        tokio::fs::write(&path, &bytes).await.map_err(to_mcp_err)?;
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Video Agent task succeeded!\ntask_id: {}\nSaved to: {}",
                            params.task_id,
                            path.display()
                        ))]));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Video Agent task succeeded!\ntask_id: {}\nvideo_url: {}",
                    params.task_id,
                    if url.is_empty() { "N/A" } else { &url }
                ))]))
            }
        },
        AsyncTaskResult::Pending { status } => Ok(CallToolResult::success(vec![Content::text(
            format!("Video Agent task status: {}\ntask_id: {}", status, params.task_id)
        )])),
        AsyncTaskResult::Failed { message } => Ok(CallToolResult::success(vec![Content::text(
            format!("Video Agent task failed: {message}")
        )])),
    }
}
