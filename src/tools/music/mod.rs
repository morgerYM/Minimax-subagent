//! Music generation tool handlers.

use async_trait::async_trait;

use crate::mcp_params::{GenerateLyricsParams, GenerateMusicCoverParams, GenerateMusicParams};
use crate::providers::{LyricsResult, MediaOutput, ProviderError};
use crate::utils;

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

// ============================================================
// Trait
// ============================================================

#[async_trait]
pub trait MusicProvider: Send + Sync {
    async fn generate_music(&self, params: &GenerateMusicParams) -> Result<MediaOutput, ProviderError>;
    async fn generate_lyrics(&self, params: &GenerateLyricsParams) -> Result<LyricsResult, ProviderError>;
    async fn generate_music_cover(&self, params: &GenerateMusicCoverParams) -> Result<MediaOutput, ProviderError>;
}

// ============================================================
// Handlers
// ============================================================

fn to_mcp_err(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

pub async fn handle_generate_music(
    provider: &dyn MusicProvider,
    params: GenerateMusicParams,
) -> Result<CallToolResult, ErrorData> {
    let output = provider.generate_music(&params).await.map_err(to_mcp_err)?;

    match output {
        MediaOutput::Bytes { data, extension } => {
            if params.output_directory.is_some() || params.output_file.is_some() {
                if let Some(path) = utils::write_output_file(
                    params.output_file.as_deref(),
                    params.output_directory.as_deref(),
                    "music",
                    &params.prompt,
                    &extension,
                    &data,
                )
                .await
                .map_err(to_mcp_err)?
                {
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "Music saved to: {}",
                        path.display()
                    ))]));
                }
            }
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Music generated! {} bytes",
                data.len()
            ))]))
        }
        MediaOutput::Url(url) => {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Music generated! URL: {url}"
            ))]))
        }
    }
}

pub async fn handle_generate_lyrics(
    provider: &dyn MusicProvider,
    params: GenerateLyricsParams,
) -> Result<CallToolResult, ErrorData> {
    let resp = provider.generate_lyrics(&params).await.map_err(to_mcp_err)?;

    let mut lines = Vec::new();
    if let Some(title) = &resp.title {
        lines.push(format!("Title: {}", title));
    }
    if let Some(tags) = &resp.style_tags {
        lines.push(format!("Style tags: {}", tags));
    }
    if let Some(lyrics) = &resp.lyrics {
        lines.push(format!("Lyrics:\n{}", lyrics));
    }

    Ok(CallToolResult::success(vec![Content::text(
        if lines.is_empty() {
            "歌词生成成功。".to_string()
        } else {
            lines.join("\n")
        },
    )]))
}

pub async fn handle_generate_music_cover(
    provider: &dyn MusicProvider,
    params: GenerateMusicCoverParams,
) -> Result<CallToolResult, ErrorData> {
    let output = provider.generate_music_cover(&params).await.map_err(to_mcp_err)?;

    match output {
        MediaOutput::Bytes { data, .. } => {
            if params.output_directory.is_some() || params.output_file.is_some() {
                if let Some(path) = utils::write_output_file(
                    params.output_file.as_deref(),
                    params.output_directory.as_deref(),
                    "music_cover",
                    &params.audio_url,
                    "mp3",
                    &data,
                )
                .await
                .map_err(to_mcp_err)?
                {
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "Cover generated! Saved to: {}",
                        path.display()
                    ))]));
                }
            }
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Cover generated! {} bytes",
                data.len()
            ))]))
        }
        MediaOutput::Url(url) => {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Cover generated! URL: {url}"
            ))]))
        }
    }
}
