use serde::{Deserialize, Deserializer, Serialize};

fn null_to_default<'de, D: Deserializer<'de>, T: Default + Deserialize<'de>>(de: D) -> Result<T, D::Error> {
    Option::deserialize(de).map(|v| v.unwrap_or_default())
}

// ============================================================
// Base response — common to all MiniMax API responses
// ============================================================

#[derive(Debug, Clone, Deserialize)]
pub struct BaseResponse {
    pub status_code: i32,
    pub status_msg: String,
}

// ============================================================
// T2A (Text to Audio) — POST /v1/t2a_v2
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct T2ARequest {
    pub model: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    pub voice_setting: VoiceSetting,
    pub audio_setting: AudioSetting,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_boost: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VoiceSetting {
    pub voice_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vol: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emotion: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioSetting {
    pub sample_rate: i32,
    pub bitrate: i32,
    pub format: String,
    pub channel: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct T2AResponse {
    pub base_resp: BaseResponse,
    #[serde(default)]
    pub data: Option<AudioData>,
    #[serde(default)]
    pub extra_info: Option<ExtraInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AudioData {
    #[serde(default)]
    pub audio: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExtraInfo {
    #[serde(default)]
    pub audio_length: Option<i32>,
    #[serde(default)]
    pub audio_sample_rate: Option<i32>,
    #[serde(default)]
    pub audio_size: Option<i32>,
    #[serde(default)]
    pub bitrate: Option<i32>,
    #[serde(default)]
    pub word_count: Option<i32>,
}

// ============================================================
// Voice List — POST /v1/get_voice
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct VoiceListRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VoiceListResponse {
    pub base_resp: BaseResponse,
    #[serde(default, deserialize_with = "null_to_default")]
    pub system_voice: Vec<VoiceInfo>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub voice_cloning: Vec<VoiceInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VoiceInfo {
    pub voice_name: String,
    pub voice_id: String,
}

// ============================================================
// Voice Clone — POST /v1/voice_clone
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct VoiceCloneRequest {
    pub file_id: String,
    pub voice_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VoiceCloneResponse {
    pub base_resp: BaseResponse,
    #[serde(default)]
    pub demo_audio: Option<String>,
}

// ============================================================
// Voice Design — POST /v1/voice_design
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct VoiceDesignRequest {
    pub prompt: String,
    pub preview_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VoiceDesignResponse {
    pub base_resp: BaseResponse,
    #[serde(default)]
    pub voice_id: Option<String>,
    #[serde(default)]
    pub trial_audio: Option<String>,
}

// ============================================================
// Video Generation — POST /v1/video_generation
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct VideoGenerationRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_frame_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VideoTaskResponse {
    pub base_resp: BaseResponse,
    pub task_id: String,
}

/// Response from GET /v1/query/video_generation?task_id=
#[derive(Debug, Clone, Deserialize)]
pub struct VideoQueryResponse {
    pub base_resp: BaseResponse,
    pub status: String,
    #[serde(default)]
    pub file_id: Option<String>,
}

/// Response from GET /v1/files/retrieve?file_id=
#[derive(Debug, Clone, Deserialize)]
pub struct FileRetrieveResponse {
    pub base_resp: BaseResponse,
    pub file: Option<FileDownloadInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileDownloadInfo {
    pub download_url: String,
}

// ============================================================
// Image Generation — POST /v1/image_generation
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct ImageGenerationRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_optimizer: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageGenerationResponse {
    pub base_resp: BaseResponse,
    #[serde(default)]
    pub data: Option<ImageData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageData {
    #[serde(default, deserialize_with = "null_to_default")]
    pub image_urls: Vec<String>,
}

// ============================================================
// Music Generation — POST /v1/music_generation
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct MusicGenerationRequest {
    pub model: String,
    pub prompt: String,
    pub lyrics: String,
    pub audio_setting: MusicAudioSetting,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MusicAudioSetting {
    pub sample_rate: i32,
    pub bitrate: i32,
    pub format: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MusicGenerationResponse {
    pub base_resp: BaseResponse,
    #[serde(default)]
    pub data: Option<AudioData>,
}

// ============================================================
// File Upload — POST /v1/files/upload
// ============================================================

#[derive(Debug, Clone, Deserialize)]
pub struct FileUploadResponse {
    pub base_resp: BaseResponse,
    pub file: Option<UploadedFileInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UploadedFileInfo {
    pub file_id: String,
}
