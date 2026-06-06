//! TTS (Text-to-Speech) tool handlers.
//!
//! Provides text-to-audio conversion via MiniMax TTS API,
//! including streaming, async, voice cloning, and voice design.

use minimax_api::consts::*;
use minimax_api::types::*;
use minimax_api::utils;
use minimax_api::MiniMaxClient;

use minimax_api::mcp_params::*;
use crate::to_mcp_err;

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

pub async fn handle_text_to_audio(
    client: &MiniMaxClient,
    params: TextToAudioParams,
) -> Result<CallToolResult, ErrorData> {
    let req = T2ARequest {
        model: params.model.unwrap_or_else(|| DEFAULT_TTS_MODEL.to_string()),
        text: params.text,
        stream: Some(false),
        stream_options: None,
        voice_setting: VoiceSetting {
            voice_id: params
                .voice_id
                .unwrap_or_else(|| DEFAULT_VOICE_ID.to_string()),
            speed: params.speed,
            vol: params.vol,
            pitch: params.pitch,
            emotion: params.emotion,
            text_normalization: None,
            latex_read: None,
            english_normalization: None,
        },
        audio_setting: AudioSetting {
            sample_rate: params.sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE),
            bitrate: params.bitrate.unwrap_or(DEFAULT_BITRATE),
            format: params.format.unwrap_or_else(|| DEFAULT_FORMAT.to_string()),
            channel: DEFAULT_CHANNEL,
            force_cbr: None,
        },
        pronunciation_dict: None,
        timbre_weights: None,
        language_boost: Some(DEFAULT_LANGUAGE_BOOST.to_string()),
        voice_modify: None,
        subtitle_enable: None,
        subtitle_type: None,
        output_format: None,
        aigc_watermark: None,
    };

    let resp = client.text_to_audio(&req).await.map_err(to_mcp_err)?;

    if params.output_directory.is_some() || params.output_file.is_some() {
        if let Some(data) = &resp.data {
            if let Some(audio_hex) = &data.audio {
                let bytes = utils::decode_hex_audio(audio_hex).map_err(to_mcp_err)?;
                if let Some(path) = utils::write_output_file(
                    params.output_file.as_deref(),
                    params.output_directory.as_deref(),
                    "text_to_audio",
                    &req.text,
                    &req.audio_setting.format,
                    &bytes,
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
        }
    }

    let json = serde_json::to_string_pretty(&resp).map_err(to_mcp_err)?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

pub async fn handle_list_voices(
    client: &MiniMaxClient,
    params: ListVoicesParams,
) -> Result<CallToolResult, ErrorData> {
    let resp = client.list_voices(params.voice_type.as_deref()).await.map_err(to_mcp_err)?;

    let mut lines = Vec::new();
    lines.push("=== System Voices ===".to_string());
    for v in &resp.system_voice {
        lines.push(format!("  {} — {}", v.voice_id, v.voice_name));
    }
    lines.push("=== Cloned Voices ===".to_string());
    for v in &resp.voice_cloning {
        lines.push(format!("  {} — {}", v.voice_id, v.voice_name));
    }
    lines.push("=== Designed Voices ===".to_string());
    for v in &resp.voice_generation {
        let desc = if v.description.is_string() {
            v.description.as_str().unwrap_or_default()
        } else {
            ""
        };
        lines.push(format!("  {} — {}", v.voice_id, desc));
    }
    lines.push(format!(
        "{} system voices + {} cloned voices + {} designed voices",
        resp.system_voice.len(),
        resp.voice_cloning.len(),
        resp.voice_generation.len()
    ));

    Ok(CallToolResult::success(vec![Content::text(lines.join("\n"))]))
}

pub async fn handle_voice_clone(
    client: &MiniMaxClient,
    params: VoiceCloneParams,
) -> Result<CallToolResult, ErrorData> {
    let file_id = if params.is_url.unwrap_or(false) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp = std::env::temp_dir().join(format!("minimax_voice_clone_{ts}"));
        client
            .download_to_path(&params.file, &tmp)
            .await
            .map_err(to_mcp_err)?;
        let upload = client
            .upload_file(&tmp, "voice_clone")
            .await
            .map_err(to_mcp_err)?;
        let _ = std::fs::remove_file(&tmp);
        upload
            .file
            .ok_or_else(|| ErrorData::internal_error("upload failed", None))?
            .file_id
    } else {
        let upload = client
            .upload_file(std::path::Path::new(&params.file), "voice_clone")
            .await
            .map_err(to_mcp_err)?;
        upload
            .file
            .ok_or_else(|| ErrorData::internal_error("upload failed", None))?
            .file_id
    };

    // 当传 text 时必须传 model (官方要求)；不传 text 时也不需要 model
    let model = if params.text.is_some() {
        Some(DEFAULT_TTS_MODEL.to_string())
    } else {
        None
    };
    let req = VoiceCloneRequest {
        file_id,
        voice_id: params.voice_id,
        clone_prompt: None,
        text: params.text,
        model,
        language_boost: params.language_boost,
        need_noise_reduction: params.need_noise_reduction,
        need_volume_normalization: params.need_volume_normalization,
        aigc_watermark: None,
    };

    let resp = client.voice_clone(&req).await.map_err(to_mcp_err)?;

    if params.output_directory.is_some() || params.output_file.is_some() {
        if let Some(demo_url) = &resp.demo_audio {
            let text = req.text.as_deref().unwrap_or("voice");
            if let Some(path) = utils::resolve_output_file(
                params.output_file.as_deref(),
                params.output_directory.as_deref(),
                "voice_clone",
                text,
                "wav",
            )
            .map_err(to_mcp_err)?
            {
                client
                    .download_to_path(demo_url, &path)
                    .await
                    .map_err(to_mcp_err)?;
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Voice cloned!\nvoice_id: {}\nSaved to: {}",
                    req.voice_id,
                    path.display()
                ))]));
            }
        }
    }

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Voice cloned!\nvoice_id: {}\ndemo_audio: {}",
        req.voice_id,
        resp.demo_audio.unwrap_or_else(|| "N/A".to_string())
    ))]))
}

pub async fn handle_voice_design(
    client: &MiniMaxClient,
    params: VoiceDesignParams,
) -> Result<CallToolResult, ErrorData> {
    let req = VoiceDesignRequest {
        prompt: params.prompt,
        preview_text: params.preview_text,
        voice_id: params.voice_id,
    };

    let resp = client.voice_design(&req).await.map_err(to_mcp_err)?;

    if params.output_directory.is_some() || params.output_file.is_some() {
        if let Some(audio_hex) = &resp.trial_audio {
            let bytes = utils::decode_hex_audio(audio_hex).map_err(to_mcp_err)?;
            if let Some(path) = utils::write_output_file(
                params.output_file.as_deref(),
                params.output_directory.as_deref(),
                "voice_design",
                &req.preview_text,
                "mp3",
                &bytes,
            )
            .await
            .map_err(to_mcp_err)?
            {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Voice design done!\nvoice_id: {}\nSaved to: {}",
                    resp.voice_id.unwrap_or_else(|| "N/A".to_string()),
                    path.display()
                ))]));
            }
        }
    }

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Voice design done!\nvoice_id: {}\ntrial_audio length: {} chars",
        resp.voice_id.unwrap_or_else(|| "N/A".to_string()),
        resp.trial_audio
            .as_ref()
            .map(|a| a.len())
            .unwrap_or(0)
    ))]))
}

pub async fn handle_delete_voice(
    client: &MiniMaxClient,
    params: DeleteVoiceParams,
) -> Result<CallToolResult, ErrorData> {
    let req = DeleteVoiceRequest {
        voice_type: params.voice_type,
        voice_id: params.voice_id,
    };
    let _resp = client.delete_voice(&req).await.map_err(to_mcp_err)?;
    Ok(CallToolResult::success(vec![Content::text(format!(
        "Voice {} deleted.",
        req.voice_id
    ))]))
}

pub async fn handle_generate_audio_async(
    client: &MiniMaxClient,
    params: GenerateAudioAsyncParams,
) -> Result<CallToolResult, ErrorData> {
    let req = T2AAsyncRequest {
        model: params.model.unwrap_or_else(|| DEFAULT_TTS_MODEL.to_string()),
        text: params.text,
        text_file_id: params.text_file_id,
        voice_setting: VoiceSetting {
            voice_id: params
                .voice_id
                .unwrap_or_else(|| DEFAULT_VOICE_ID.to_string()),
            speed: params.speed,
            vol: params.vol,
            pitch: params.pitch,
            emotion: params.emotion,
            text_normalization: None,
            latex_read: None,
            english_normalization: None,
        },
        audio_setting: Some(AsyncAudioSetting {
            audio_sample_rate: params.sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE),
            bitrate: params.bitrate.unwrap_or(DEFAULT_BITRATE),
            format: params.format.unwrap_or_else(|| DEFAULT_FORMAT.to_string()),
            channel: params.channel.unwrap_or(DEFAULT_CHANNEL_ASYNC),
        }),
        pronunciation_dict: None,
        language_boost: Some(
            params
                .language_boost
                .unwrap_or_else(|| DEFAULT_LANGUAGE_BOOST.to_string()),
        ),
        voice_modify: None,
        aigc_watermark: params.aigc_watermark,
    };

    let resp = client.create_async_tts(&req).await.map_err(to_mcp_err)?;

    let json = serde_json::to_string_pretty(&serde_json::json!({
        "task_id": resp.task_id,
        "file_id": resp.file_id,
        "usage_characters": resp.usage_characters,
        "output_directory": params.output_directory,
        "output_file": params.output_file,
        "message": format!("Async TTS task submitted. Use query_audio_task --task_id {} to poll progress and download result. (output_directory/output_file are echoed back for reference; the actual save happens in the query step.)", resp.task_id),
    }))
    .map_err(to_mcp_err)?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

pub async fn handle_query_audio_task(
    client: &MiniMaxClient,
    params: QueryAudioTaskParams,
) -> Result<CallToolResult, ErrorData> {
    let task_id: i64 = params
        .task_id
        .parse()
        .map_err(|e| ErrorData::internal_error(format!("Invalid task_id: {e}"), None))?;

    let status = client.query_async_tts(task_id).await.map_err(to_mcp_err)?;

    match status.status.as_str() {
        "Failed" | "Expired" => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Task status: {}\ntask_id: {}",
                status.status, task_id
            ))]));
        }
        _ => {}
    }

    let file_id = status.file_id.unwrap_or(task_id);

    let download_url = client
        .poll_file_download_url(file_id, ASYNC_TTS_MAX_POLL_RETRIES, ASYNC_TTS_POLL_INTERVAL_SECS)
        .await
        .map_err(to_mcp_err)?;

    let tar_bytes = client
        .download_bytes(&download_url)
        .await
        .map_err(to_mcp_err)?;

    let mut tar = tar::Archive::new(std::io::Cursor::new(&tar_bytes));
    let mut mp3_bytes: Option<Vec<u8>> = None;

    for entry in tar.entries().map_err(to_mcp_err)? {
        let mut entry = entry.map_err(to_mcp_err)?;
        let path = entry.path().map_err(to_mcp_err)?;
        let name = path.to_string_lossy();
        if name.ends_with(".mp3") {
            let mut out = Vec::new();
            std::io::copy(&mut entry, &mut out).map_err(to_mcp_err)?;
            mp3_bytes = Some(out);
            break;
        }
    }

    let mp3 = mp3_bytes.ok_or_else(|| {
        ErrorData::internal_error("tar 包中未找到 mp3 文件", None)
    })?;

    if params.output_directory.is_some() || params.output_file.is_some() {
        if let Some(path) = utils::write_output_file(
            params.output_file.as_deref(),
            params.output_directory.as_deref(),
            "async_tts",
            &task_id.to_string(),
            "mp3",
            &mp3,
        )
        .await
        .map_err(to_mcp_err)?
        {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Async TTS done! Saved to: {} ({} bytes)",
                path.display(),
                mp3.len()
            ))]));
        }
    }

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Async TTS done! Audio size: {} bytes ({} KB)",
        mp3.len(),
        mp3.len() / 1024
    ))]))
}

pub async fn handle_text_to_audio_stream(
    client: &MiniMaxClient,
    params: TextToAudioStreamParams,
) -> Result<CallToolResult, ErrorData> {
    let base_url = &client.base_url;
    let api_key = &client.api_key;

    let ws_req = WsTaskStart {
        event: "task_start".to_string(),
        model: params.model.unwrap_or_else(|| DEFAULT_TTS_MODEL.to_string()),
        voice_setting: WsVoiceSetting {
            voice_id: params.voice_id.unwrap_or_else(|| DEFAULT_VOICE_ID.to_string()),
            speed: params.speed,
            vol: params.vol,
            pitch: params.pitch,
            emotion: params.emotion,
            english_normalization: None,
            latex_read: None,
        },
        audio_setting: Some(WsAudioSetting {
            sample_rate: params.sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE),
            bitrate: params.bitrate.unwrap_or(DEFAULT_BITRATE),
            format: params.format.unwrap_or_else(|| DEFAULT_FORMAT.to_string()),
            channel: params.channel.unwrap_or(1),
        }),
        language_boost: params.language_boost,
        pronunciation_dict: None,
        timbre_weights: None,
        voice_modify: None,
        subtitle_enable: None,
        subtitle_type: None,
        continuous_sound: params.continuous_sound,
    };

    let mut ws = minimax_api::ws_client::WsTtsClient::connect(base_url, api_key)
        .await
        .map_err(to_mcp_err)?;

    ws.task_start(&ws_req).await.map_err(to_mcp_err)?;

    let (audio_bytes, _is_final) = ws.task_continue(&params.text).await.map_err(to_mcp_err)?;

    ws.task_finish().await.map_err(to_mcp_err)?;

    if params.output_directory.is_some() || params.output_file.is_some() {
        let ext_owned = ws_req
            .audio_setting
            .as_ref()
            .map(|a| a.format.clone())
            .unwrap_or_else(|| "mp3".to_string());
        if let Some(path) = utils::write_output_file(
            params.output_file.as_deref(),
            params.output_directory.as_deref(),
            "stream_tts",
            &params.text,
            &ext_owned,
            &audio_bytes,
        )
        .await
        .map_err(to_mcp_err)?
        {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Stream TTS done! Saved to: {} ({} bytes)",
                path.display(),
                audio_bytes.len()
            ))]));
        }
    }

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Stream TTS done! Audio size: {} bytes ({} KB)",
        audio_bytes.len(),
        audio_bytes.len() / 1024
    ))]))
}
