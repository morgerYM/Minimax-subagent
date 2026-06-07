//! TtsProvider + VoiceProvider impl for MiniMaxProvider.

use async_trait::async_trait;

use crate::consts::*;
use crate::mcp_params::*;
use crate::providers::*;
use crate::types::*;
use crate::ws_client::WsTtsClient;

use super::MiniMaxProvider;

// ============================================================
// TtsProvider
// ============================================================

#[async_trait]
impl crate::tools::tts::TtsProvider for MiniMaxProvider {
    async fn text_to_audio(&self, params: &TextToAudioParams) -> Result<MediaOutput, ProviderError> {
        let req = T2ARequest {
            model: params.model.clone().unwrap_or_else(|| DEFAULT_TTS_MODEL.to_string()),
            text: params.text.clone(),
            stream: Some(false),
            stream_options: None,
            voice_setting: VoiceSetting {
                voice_id: params.voice_id.clone().unwrap_or_else(|| DEFAULT_VOICE_ID.to_string()),
                speed: params.speed,
                vol: params.vol,
                pitch: params.pitch,
                emotion: params.emotion.clone(),
                text_normalization: None,
                latex_read: None,
                english_normalization: None,
            },
            audio_setting: AudioSetting {
                sample_rate: params.sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE),
                bitrate: params.bitrate.unwrap_or(DEFAULT_BITRATE),
                format: params.format.clone().unwrap_or_else(|| DEFAULT_FORMAT.to_string()),
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

        let resp = self.client.text_to_audio(&req).await?;
        let ext = req.audio_setting.format.clone();
        match resp.data.and_then(|d| d.audio) {
            Some(hex_audio) => {
                let bytes = hex::decode(&hex_audio)
                    .map_err(|e| ProviderError::Other(format!("hex decode: {e}")))?;
                Ok(MediaOutput::Bytes { data: bytes, extension: ext })
            }
            None => Err(ProviderError::Api("no audio in response".into())),
        }
    }

    async fn text_to_audio_stream(
        &self,
        params: &TextToAudioStreamParams,
    ) -> Result<MediaOutput, ProviderError> {
        let ws_req = WsTaskStart {
            event: "task_start".to_string(),
            model: params.model.clone().unwrap_or_else(|| DEFAULT_TTS_MODEL.to_string()),
            voice_setting: WsVoiceSetting {
                voice_id: params.voice_id.clone().unwrap_or_else(|| DEFAULT_VOICE_ID.to_string()),
                speed: params.speed,
                vol: params.vol,
                pitch: params.pitch,
                emotion: params.emotion.clone(),
                english_normalization: None,
                latex_read: None,
            },
            audio_setting: Some(WsAudioSetting {
                sample_rate: params.sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE),
                bitrate: params.bitrate.unwrap_or(DEFAULT_BITRATE),
                format: params.format.clone().unwrap_or_else(|| DEFAULT_FORMAT.to_string()),
                channel: params.channel.unwrap_or(1),
            }),
            language_boost: params.language_boost.clone(),
            pronunciation_dict: None,
            timbre_weights: None,
            voice_modify: None,
            subtitle_enable: None,
            subtitle_type: None,
            continuous_sound: params.continuous_sound,
        };

        let mut ws = WsTtsClient::connect(&self.client.base_url, &self.client.api_key).await?;
        ws.task_start(&ws_req).await?;
        let (audio_bytes, _is_final) = ws.task_continue(&params.text).await?;
        ws.task_finish().await?;

        Ok(MediaOutput::Bytes {
            data: audio_bytes,
            extension: ws_req.audio_setting.map(|a| a.format).unwrap_or_else(|| "mp3".to_string()),
        })
    }

    async fn submit_async_tts(
        &self,
        params: &GenerateAudioAsyncParams,
    ) -> Result<AsyncTaskHandle, ProviderError> {
        let req = T2AAsyncRequest {
            model: params.model.clone().unwrap_or_else(|| DEFAULT_TTS_MODEL.to_string()),
            text: params.text.clone(),
            text_file_id: params.text_file_id,
            voice_setting: VoiceSetting {
                voice_id: params.voice_id.clone().unwrap_or_else(|| DEFAULT_VOICE_ID.to_string()),
                speed: params.speed,
                vol: params.vol,
                pitch: params.pitch,
                emotion: params.emotion.clone(),
                text_normalization: None,
                latex_read: None,
                english_normalization: None,
            },
            audio_setting: Some(AsyncAudioSetting {
                audio_sample_rate: params.sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE),
                bitrate: params.bitrate.unwrap_or(DEFAULT_BITRATE),
                format: params.format.clone().unwrap_or_else(|| DEFAULT_FORMAT.to_string()),
                channel: params.channel.unwrap_or(DEFAULT_CHANNEL_ASYNC),
            }),
            pronunciation_dict: None,
            language_boost: Some(params.language_boost.clone().unwrap_or_else(|| DEFAULT_LANGUAGE_BOOST.to_string())),
            voice_modify: None,
            aigc_watermark: params.aigc_watermark,
        };

        let resp = self.client.create_async_tts(&req).await?;
        Ok(AsyncTaskHandle {
            task_id: resp.task_id,
            extra: serde_json::to_value(&serde_json::json!({
                "file_id": resp.file_id,
                "usage_characters": resp.usage_characters,
            })).ok(),
        })
    }

    async fn query_async_tts(&self, task_id: &str) -> Result<AsyncTaskResult, ProviderError> {
        let tid: i64 = task_id.parse().map_err(|e| ProviderError::Other(format!("invalid task_id: {e}")))?;
        let status = self.client.query_async_tts(tid).await?;

        match status.status.as_str() {
            "Failed" | "Expired" => {
                return Ok(AsyncTaskResult::Failed {
                    message: format!("Task status: {} (task_id: {})", status.status, tid),
                });
            }
            _ => {}
        }

        let file_id = status.file_id.unwrap_or(tid);
        let download_url = self.client.poll_file_download_url(
            file_id,
            ASYNC_TTS_MAX_POLL_RETRIES,
            ASYNC_TTS_POLL_INTERVAL_SECS,
        ).await?;
        let tar_bytes = self.client.download_bytes(&download_url).await?;
        let mut tar = tar::Archive::new(std::io::Cursor::new(&tar_bytes));
        let mut mp3_bytes: Option<Vec<u8>> = None;

        for entry in tar.entries().map_err(|e| ProviderError::Other(e.to_string()))? {
            let mut entry = entry.map_err(|e| ProviderError::Other(e.to_string()))?;
            let path = entry.path().map_err(|e| ProviderError::Other(e.to_string()))?;
            if path.to_string_lossy().ends_with(".mp3") {
                let mut out = Vec::new();
                std::io::copy(&mut entry, &mut out).map_err(|e| ProviderError::Other(e.to_string()))?;
                mp3_bytes = Some(out);
                break;
            }
        }

        match mp3_bytes {
            Some(data) => Ok(AsyncTaskResult::Completed(MediaOutput::Bytes {
                data,
                extension: "mp3".to_string(),
            })),
            None => Err(ProviderError::Other("tar 包中未找到 mp3 文件".into())),
        }
    }
}

// ============================================================
// VoiceProvider
// ============================================================

#[async_trait]
impl crate::tools::tts::VoiceProvider for MiniMaxProvider {
    async fn list_voices(&self, voice_type: Option<&str>) -> Result<VoiceListResult, ProviderError> {
        let resp = self.client.list_voices(voice_type).await?;
        Ok(VoiceListResult {
            system: resp.system_voice.into_iter().map(|v| VoiceInfoOutput {
                voice_id: v.voice_id,
                voice_name: v.voice_name,
                description: String::new(),
            }).collect(),
            cloned: resp.voice_cloning.into_iter().map(|v| VoiceInfoOutput {
                voice_id: v.voice_id,
                voice_name: v.voice_name,
                description: String::new(),
            }).collect(),
            designed: resp.voice_generation.into_iter().map(|v| {
                let desc = if v.description.is_string() {
                    v.description.as_str().unwrap_or_default().to_string()
                } else {
                    String::new()
                };
                VoiceDesignedOutput { voice_id: v.voice_id, description: desc }
            }).collect(),
        })
    }

    async fn voice_clone(&self, params: &VoiceCloneParams) -> Result<VoiceCloneResult, ProviderError> {
        let file_id = if params.is_url.unwrap_or(false) {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let tmp = std::env::temp_dir().join(format!("minimax_voice_clone_{ts}"));
            self.client.download_to_path(&params.file, &tmp).await?;
            let upload = self.client.upload_file(&tmp, "voice_clone").await?;
            let _ = std::fs::remove_file(&tmp);
            upload.file.ok_or_else(|| ProviderError::Api("upload failed".into()))?.file_id
        } else {
            let upload = self.client.upload_file(std::path::Path::new(&params.file), "voice_clone").await?;
            upload.file.ok_or_else(|| ProviderError::Api("upload failed".into()))?.file_id
        };

        let model = if params.text.is_some() {
            Some(DEFAULT_TTS_MODEL.to_string())
        } else {
            None
        };

        let req = VoiceCloneRequest {
            file_id,
            voice_id: params.voice_id.clone(),
            clone_prompt: None,
            text: params.text.clone(),
            model,
            language_boost: params.language_boost.clone(),
            need_noise_reduction: params.need_noise_reduction,
            need_volume_normalization: params.need_volume_normalization,
            aigc_watermark: None,
        };

        let resp = self.client.voice_clone(&req).await?;

        let demo_audio = resp.demo_audio.map(|url| MediaOutput::Url(url));
        Ok(VoiceCloneResult {
            voice_id: req.voice_id,
            demo_audio,
        })
    }

    async fn voice_design(&self, params: &VoiceDesignParams) -> Result<MediaOutput, ProviderError> {
        let req = VoiceDesignRequest {
            prompt: params.prompt.clone(),
            preview_text: params.preview_text.clone(),
            voice_id: params.voice_id.clone(),
        };

        let resp = self.client.voice_design(&req).await?;
        match resp.trial_audio {
            Some(hex_audio) => {
                let bytes = hex::decode(&hex_audio)
                    .map_err(|e| ProviderError::Other(format!("hex decode: {e}")))?;
                Ok(MediaOutput::Bytes { data: bytes, extension: "mp3".to_string() })
            }
            None => Err(ProviderError::Api("no trial audio in response".into())),
        }
    }

    async fn delete_voice(&self, voice_type: &str, voice_id: &str) -> Result<(), ProviderError> {
        let req = DeleteVoiceRequest {
            voice_type: voice_type.to_string(),
            voice_id: voice_id.to_string(),
        };
        self.client.delete_voice(&req).await?;
        Ok(())
    }
}
