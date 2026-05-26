use serde::{Deserialize, Deserializer, Serialize};

fn null_to_default<'de, D: Deserializer<'de>, T: Default + Deserialize<'de>>(de: D) -> Result<T, D::Error> {
    Option::deserialize(de).map(|v| v.unwrap_or_default())
}

// ============================================================
// Base response — common to all MiniMax API responses
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct T2AResponse {
    pub base_resp: BaseResponse,
    #[serde(default)]
    pub data: Option<AudioData>,
    #[serde(default)]
    pub extra_info: Option<ExtraInfo>,
    #[serde(default)]
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    #[serde(default)]
    pub audio: Option<String>,
    #[serde(default)]
    pub status: Option<i32>,
    #[serde(default)]
    pub ced: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(default)]
    pub invisible_character_ratio: Option<f64>,
    #[serde(default)]
    pub usage_characters: Option<i32>,
    #[serde(default)]
    pub audio_format: Option<String>,
    #[serde(default)]
    pub audio_channel: Option<i32>,
}

// ============================================================
// Voice List — POST /v1/get_voice
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct VoiceListRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_feature_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timbre: Option<String>,
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

// ============================================================
// Token Plan — GET /v1/token_plan/remains
// ============================================================

#[derive(Debug, Clone, Deserialize)]
pub struct TokenPlanResponse {
    pub base_resp: BaseResponse,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

// ============================================================
// Chat — POST /v1/messages (Anthropic 兼容)
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    pub stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub content: Vec<ChatContentBlock>,
    #[serde(default)]
    pub stop_reason: Option<String>,
    #[serde(default)]
    pub usage: Option<ChatUsage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatUsage {
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
}

// ============================================================
// Lyrics Generation — POST /v1/lyrics_generation
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct LyricsGenerationRequest {
    pub mode: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LyricsGenerationResponse {
    pub base_resp: BaseResponse,
    #[serde(default)]
    pub song_title: Option<String>,
    #[serde(default)]
    pub style_tags: Option<String>,
    #[serde(default)]
    pub lyrics: Option<String>,
}

// ============================================================
// Music Cover — POST /v1/music_cover_preprocess
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct MusicCoverPreprocessRequest {
    pub audio_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MusicCoverPreprocessResponse {
    pub base_resp: BaseResponse,
    #[serde(default)]
    pub cover_feature_id: Option<String>,
}

// ============================================================
// Search — POST /v1/coding_plan/search
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct SearchRequest {
    pub q: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    #[serde(default)]
    pub organic: Vec<SearchResult>,
    #[serde(default)]
    pub related_searches: Vec<RelatedSearch>,
    #[serde(default)]
    pub base_resp: Option<BaseResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub link: String,
    pub snippet: String,
    #[serde(default)]
    pub date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RelatedSearch {
    pub query: String,
}

// ============================================================
// Async TTS — POST /v1/t2a_async_v2 & GET /v1/query/t2a_async_query_v2
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct T2AAsyncRequest {
    pub model: String,
    pub text: String,
    pub voice_setting: VoiceSetting,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_setting: Option<AsyncAudioSetting>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_boost: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AsyncAudioSetting {
    pub audio_sample_rate: i32,
    pub bitrate: i32,
    pub format: String,
    pub channel: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct T2AAsyncCreateResponse {
    pub base_resp: BaseResponse,
    pub task_id: i64,
    pub task_token: String,
    pub file_id: i64,
    #[serde(default)]
    pub usage_characters: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct T2AAsyncQueryResponse {
    pub base_resp: BaseResponse,
    pub status: String,
    pub task_id: i64,
    #[serde(default)]
    pub file_id: Option<i64>,
}

// ============================================================
// VLM (Vision Language Model) — POST /v1/coding_plan/vlm
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct VlmRequest {
    pub prompt: String,
    pub image_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VlmResponse {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub base_resp: Option<BaseResponse>,
}
