//! Music generation tool handlers.
//!
//! Provides music generation, lyrics generation, and music cover
//! via MiniMax Music API.

use minimax_api::consts::*;
use minimax_api::types::*;
use minimax_api::utils;
use minimax_api::MiniMaxClient;

use minimax_api::mcp_params::*;
use crate::to_mcp_err;

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

pub async fn handle_generate_music(
    client: &MiniMaxClient,
    params: GenerateMusicParams,
) -> Result<CallToolResult, ErrorData> {
    let req = MusicGenerationRequest {
        model: params.model.unwrap_or_else(|| DEFAULT_MUSIC_MODEL.to_string()),
        prompt: params.prompt,
        lyrics: params.lyrics,
        audio_setting: MusicAudioSetting {
            sample_rate: DEFAULT_SAMPLE_RATE,
            bitrate: DEFAULT_BITRATE,
            format: params.format.unwrap_or_else(|| DEFAULT_FORMAT.to_string()),
        },
        output_format: None,
        audio_url: None,
        audio_base64: None,
        cover_feature_id: None,
        timbre: None,
        stream: params.stream,
        aigc_watermark: params.aigc_watermark,
        lyrics_optimizer: params.lyrics_optimizer,
        is_instrumental: params.is_instrumental,
    };

    let resp = client.generate_music(&req).await.map_err(to_mcp_err)?;

    if let Some(dir) = &params.output_directory {
        if let Some(data) = &resp.data {
            if let Some(audio_hex) = &data.audio {
                let path = utils::resolve_and_create_dir(dir).map_err(to_mcp_err)?;
                let ext = &req.audio_setting.format;
                let filename = utils::build_filename("music", &req.prompt, ext);
                let filepath = path.join(filename);
                let bytes = utils::decode_hex_audio(audio_hex).map_err(to_mcp_err)?;
                tokio::fs::write(&filepath, &bytes).await.map_err(to_mcp_err)?;
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Music saved to: {}",
                    filepath.display()
                ))]));
            }
        }
    }

    if let Some(data) = &resp.data {
        if let Some(audio) = &data.audio {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Music generated! Data length: {} chars",
                audio.len()
            ))]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(
                "音乐生成成功，但未返回音频数据。".to_string(),
            )]))
        }
    } else {
        Ok(CallToolResult::success(vec![Content::text(
            "音乐生成成功。".to_string(),
        )]))
    }
}

pub async fn handle_generate_lyrics(
    client: &MiniMaxClient,
    params: GenerateLyricsParams,
) -> Result<CallToolResult, ErrorData> {
    let mode = params.mode.unwrap_or_else(|| "write_full_song".to_string());
    let req = LyricsGenerationRequest {
        mode,
        prompt: params.prompt,
        lyrics: params.lyrics,
        title: params.title,
    };

    let resp = client.generate_lyrics(&req).await.map_err(to_mcp_err)?;

    let mut lines = Vec::new();
    if let Some(title) = &resp.song_title {
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
    client: &MiniMaxClient,
    params: GenerateMusicCoverParams,
) -> Result<CallToolResult, ErrorData> {
    let pre_resp = client
        .preprocess_music_cover(&params.audio_url)
        .await
        .map_err(to_mcp_err)?;

    let cover_feature_id = pre_resp
        .cover_feature_id
        .ok_or_else(|| ErrorData::internal_error("Preprocessing failed: cover_feature_id not found", None))?;

    let req = MusicGenerationRequest {
        model: "music-cover".to_string(),
        prompt: params.prompt.unwrap_or_default(),
        lyrics: params.lyrics.unwrap_or_default(),
        audio_setting: MusicAudioSetting {
            sample_rate: DEFAULT_SAMPLE_RATE,
            bitrate: DEFAULT_BITRATE,
            format: DEFAULT_FORMAT.to_string(),
        },
        output_format: None,
        audio_url: None,
        audio_base64: None,
        cover_feature_id: Some(cover_feature_id),
        timbre: None,
        stream: None,
        aigc_watermark: None,
        lyrics_optimizer: None,
        is_instrumental: None,
    };

    let resp = client.generate_music(&req).await.map_err(to_mcp_err)?;

    if let Some(dir) = &params.output_directory {
        if let Some(data) = &resp.data {
            if let Some(audio_hex) = &data.audio {
                let path = utils::resolve_and_create_dir(dir).map_err(to_mcp_err)?;
                let filename = utils::build_filename("music_cover", &req.prompt, "mp3");
                let filepath = path.join(filename);
                let bytes = utils::decode_hex_audio(audio_hex).map_err(to_mcp_err)?;
                tokio::fs::write(&filepath, &bytes).await.map_err(to_mcp_err)?;
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Cover generated! Saved to: {}",
                    filepath.display()
                ))]));
            }
        }
    }

    if let Some(data) = &resp.data {
        if let Some(audio) = &data.audio {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Cover generated! Data length: {} chars",
                audio.len()
            ))]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(
                "翻唱生成成功，但未返回音频数据。".to_string(),
            )]))
        }
    } else {
        Ok(CallToolResult::success(vec![Content::text(
            "翻唱生成成功。".to_string(),
        )]))
    }
}
