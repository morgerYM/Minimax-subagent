//! MusicProvider impl for MiniMaxProvider.

use async_trait::async_trait;

use crate::consts::*;
use crate::mcp_params::*;
use crate::providers::*;
use crate::types::*;

use super::MiniMaxProvider;

#[async_trait]
impl crate::tools::music::MusicProvider for MiniMaxProvider {
    async fn generate_music(&self, params: &GenerateMusicParams) -> Result<MediaOutput, ProviderError> {
        let req = MusicGenerationRequest {
            model: params.model.clone().unwrap_or_else(|| DEFAULT_MUSIC_MODEL.to_string()),
            prompt: params.prompt.clone(),
            lyrics: params.lyrics.clone(),
            audio_setting: MusicAudioSetting {
                sample_rate: DEFAULT_SAMPLE_RATE,
                bitrate: DEFAULT_BITRATE,
                format: params.format.clone().unwrap_or_else(|| DEFAULT_FORMAT.to_string()),
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

        let fmt = req.audio_setting.format.clone();
        let resp = self.client.generate_music(&req).await?;

        match resp.data.and_then(|d| d.audio) {
            Some(hex_audio) => {
                let bytes = hex::decode(&hex_audio)
                    .map_err(|e| ProviderError::Other(format!("hex decode: {e}")))?;
                Ok(MediaOutput::Bytes { data: bytes, extension: fmt })
            }
            None => Err(ProviderError::Api("no audio in response".into())),
        }
    }

    async fn generate_lyrics(&self, params: &GenerateLyricsParams) -> Result<LyricsResult, ProviderError> {
        let req = LyricsGenerationRequest {
            mode: params.mode.clone().unwrap_or_else(|| "write_full_song".to_string()),
            prompt: params.prompt.clone(),
            lyrics: params.lyrics.clone(),
            title: params.title.clone(),
        };

        let resp = self.client.generate_lyrics(&req).await?;
        Ok(LyricsResult {
            title: resp.song_title,
            style_tags: resp.style_tags,
            lyrics: resp.lyrics,
        })
    }

    async fn generate_music_cover(&self, params: &GenerateMusicCoverParams) -> Result<MediaOutput, ProviderError> {
        let pre_resp = self.client.preprocess_music_cover(&params.audio_url).await?;

        let cover_feature_id = pre_resp.cover_feature_id
            .ok_or_else(|| ProviderError::Api("cover_feature_id not found".into()))?;

        let req = MusicGenerationRequest {
            model: "music-cover".to_string(),
            prompt: params.prompt.clone().unwrap_or_default(),
            lyrics: params.lyrics.clone().unwrap_or_default(),
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

        let resp = self.client.generate_music(&req).await?;

        match resp.data.and_then(|d| d.audio) {
            Some(hex_audio) => {
                let bytes = hex::decode(&hex_audio)
                    .map_err(|e| ProviderError::Other(format!("hex decode: {e}")))?;
                Ok(MediaOutput::Bytes { data: bytes, extension: "mp3".to_string() })
            }
            None => Err(ProviderError::Api("no audio in response".into())),
        }
    }
}
