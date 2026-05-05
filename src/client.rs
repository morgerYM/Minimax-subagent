use std::path::Path;
use std::time::Duration;

use reqwest::multipart;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::time::sleep;
use tracing::info;

use crate::consts::*;
use crate::error::MiniMaxError;
use crate::types::*;

/// MiniMax API client — typed async wrapper for all MiniMax endpoints.
///
/// ```no_run
/// use minimax_api::MiniMaxClient;
/// let client = MiniMaxClient::from_env()?;
/// ```
#[derive(Clone)]
pub struct MiniMaxClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl MiniMaxClient {
    /// Create a new client with explicit credentials.
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url,
            api_key,
        }
    }

    /// Create a client from `MINIMAX_API_KEY` and `MINIMAX_API_HOST` env vars.
    ///
    /// `MINIMAX_API_HOST` defaults to `https://api.minimaxi.com`.
    pub fn from_env() -> Result<Self, MiniMaxError> {
        let api_key = std::env::var(ENV_API_KEY)
            .map_err(|_| MiniMaxError::MissingEnv(ENV_API_KEY.to_string()))?;
        let base_url = std::env::var(ENV_API_HOST)
            .unwrap_or_else(|_| DEFAULT_API_HOST.to_string());
        Ok(Self::new(api_key, base_url))
    }

    // ============================================================
    // Internal helpers
    // ============================================================

    async fn post_json<T: Serialize, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<R, MiniMaxError> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(body)
            .send()
            .await?;

        let trace_id = response
            .headers()
            .get("Trace-Id")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let value: serde_json::Value = response.json().await?;
        Self::check_base_resp(&value, trace_id)?;
        Ok(serde_json::from_value(value)?)
    }

    async fn get_json<R: DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> Result<R, MiniMaxError> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        let trace_id = response
            .headers()
            .get("Trace-Id")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let value: serde_json::Value = response.json().await?;
        Self::check_base_resp(&value, trace_id)?;
        Ok(serde_json::from_value(value)?)
    }

    fn check_base_resp(
        value: &serde_json::Value,
        trace_id: Option<String>,
    ) -> Result<(), MiniMaxError> {
        if let Some(base_resp) = value.get("base_resp") {
            let code = base_resp
                .get("status_code")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            if code != 0 {
                let message = base_resp
                    .get("status_msg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
                    .to_string();
                return Err(MiniMaxError::Api {
                    code: code as i32,
                    message,
                    trace_id,
                });
            }
        }
        Ok(())
    }

    // ============================================================
    // TTS (Text to Speech)
    // ============================================================

    /// POST /v1/t2a_v2 — synchronous text-to-speech.
    ///
    /// Returns the response containing hex-encoded audio or an audio URL.
    pub async fn text_to_audio(
        &self,
        req: &T2ARequest,
    ) -> Result<T2AResponse, MiniMaxError> {
        self.post_json("/v1/t2a_v2", req).await
    }

    // ============================================================
    // Voices
    // ============================================================

    /// POST /v1/get_voice — list available voices.
    ///
    /// `voice_type`: `"all"`, `"system"`, or `"voice_cloning"`.
    pub async fn list_voices(
        &self,
        voice_type: Option<&str>,
    ) -> Result<VoiceListResponse, MiniMaxError> {
        let req = VoiceListRequest {
            voice_type: voice_type.map(String::from),
        };
        self.post_json("/v1/get_voice", &req).await
    }

    /// POST /v1/voice_clone — clone a voice from an uploaded audio file.
    pub async fn voice_clone(
        &self,
        req: &VoiceCloneRequest,
    ) -> Result<VoiceCloneResponse, MiniMaxError> {
        self.post_json("/v1/voice_clone", req).await
    }

    /// POST /v1/voice_design — create a custom voice from a text description.
    pub async fn voice_design(
        &self,
        req: &VoiceDesignRequest,
    ) -> Result<VoiceDesignResponse, MiniMaxError> {
        self.post_json("/v1/voice_design", req).await
    }

    // ============================================================
    // Video Generation
    // ============================================================

    /// POST /v1/video_generation — submit a video generation task.
    ///
    /// Returns the `task_id` to poll with `query_video`.
    pub async fn create_video(
        &self,
        req: &VideoGenerationRequest,
    ) -> Result<VideoTaskResponse, MiniMaxError> {
        self.post_json("/v1/video_generation", req).await
    }

    /// GET /v1/query/video_generation?task_id= — poll video task status.
    pub async fn query_video(
        &self,
        task_id: &str,
    ) -> Result<VideoQueryResponse, MiniMaxError> {
        let endpoint = format!("/v1/query/video_generation?task_id={}", task_id);
        self.get_json(&endpoint).await
    }

    /// GET /v1/files/retrieve?file_id= — get download URL for a generated file.
    pub async fn get_file_download_url(
        &self,
        file_id: &str,
    ) -> Result<String, MiniMaxError> {
        let endpoint = format!("/v1/files/retrieve?file_id={}", file_id);
        let resp: FileRetrieveResponse = self.get_json(&endpoint).await?;
        resp.file
            .map(|f| f.download_url)
            .ok_or_else(|| MiniMaxError::Api {
                code: -1,
                message: "no download_url in response".to_string(),
                trace_id: None,
            })
    }

    /// Submit video task and poll until completion. Returns the video bytes.
    ///
    /// Uses 20s polling interval. For Hailuo-02 models, polls up to 60 times;
    /// for other models, up to 30 times.
    pub async fn generate_video_and_download(
        &self,
        req: &VideoGenerationRequest,
    ) -> Result<Vec<u8>, MiniMaxError> {
        let task = self.create_video(req).await?;
        info!("Video task created: {}", task.task_id);

        let is_hailuo_02 = req.model.contains("Hailuo-02");
        let max_retries = if is_hailuo_02 {
            MAX_POLL_RETRIES_HAILUO_02
        } else {
            MAX_POLL_RETRIES
        };

        for attempt in 0..max_retries {
            sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;

            let status = self.query_video(&task.task_id).await?;
            match status.status.as_str() {
                "Success" => {
                    let file_id = status.file_id.ok_or_else(|| MiniMaxError::Api {
                        code: -1,
                        message: "task succeeded but no file_id".to_string(),
                        trace_id: None,
                    })?;
                    let download_url = self.get_file_download_url(&file_id).await?;
                    let bytes = self
                        .http
                        .get(&download_url)
                        .send()
                        .await?
                        .bytes()
                        .await?;
                    return Ok(bytes.to_vec());
                }
                "Fail" => {
                    return Err(MiniMaxError::TaskFailed {
                        task_id: task.task_id,
                    });
                }
                _ => {
                    info!(
                        "Video task {} status: {} (attempt {}/{})",
                        task.task_id,
                        status.status,
                        attempt + 1,
                        max_retries
                    );
                }
            }
        }

        Err(MiniMaxError::TaskTimeout {
            task_id: task.task_id,
            max_retries,
        })
    }

    // ============================================================
    // Image Generation
    // ============================================================

    /// POST /v1/image_generation — generate images from a text prompt.
    pub async fn generate_image(
        &self,
        req: &ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, MiniMaxError> {
        self.post_json("/v1/image_generation", req).await
    }

    // ============================================================
    // Music Generation
    // ============================================================

    /// POST /v1/music_generation — generate music from a prompt and lyrics.
    pub async fn generate_music(
        &self,
        req: &MusicGenerationRequest,
    ) -> Result<MusicGenerationResponse, MiniMaxError> {
        self.post_json("/v1/music_generation", req).await
    }

    // ============================================================
    // File Management
    // ============================================================

    /// POST /v1/files/upload — upload a file via multipart form.
    pub async fn upload_file(
        &self,
        file_path: &Path,
        purpose: &str,
    ) -> Result<FileUploadResponse, MiniMaxError> {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");

        let mime_type = mime_guess(file_path);

        let file_bytes = tokio::fs::read(file_path).await?;

        let form = multipart::Form::new()
            .part(
                "file",
                multipart::Part::bytes(file_bytes)
                    .file_name(file_name.to_string())
                    .mime_str(&mime_type)
                    .map_err(|e| MiniMaxError::Api {
                        code: -1,
                        message: format!("invalid MIME type: {e}"),
                        trace_id: None,
                    })?,
            )
            .text("purpose", purpose.to_string());

        let url = format!("{}/v1/files/upload", self.base_url);
        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        let trace_id = response
            .headers()
            .get("Trace-Id")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let value: serde_json::Value = response.json().await?;
        Self::check_base_resp(&value, trace_id)?;
        Ok(serde_json::from_value(value)?)
    }

    /// Download a file from a URL to a local path.
    pub async fn download_to_path(
        &self,
        url: &str,
        output: &Path,
    ) -> Result<(), MiniMaxError> {
        let bytes = self.http.get(url).send().await?.bytes().await?;
        if let Some(parent) = output.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(output, bytes).await?;
        Ok(())
    }
}

fn mime_guess(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("mp3") | Some("mpeg") => "audio/mpeg".to_string(),
        Some("wav") => "audio/wav".to_string(),
        Some("flac") => "audio/flac".to_string(),
        Some("m4a") => "audio/mp4".to_string(),
        Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
        Some("png") => "image/png".to_string(),
        Some("gif") => "image/gif".to_string(),
        Some("webp") => "image/webp".to_string(),
        Some("mp4") => "video/mp4".to_string(),
        Some("pdf") => "application/pdf".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}
