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
            .header("MM-API-Source", "Minimax-MCP")
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
            .header("MM-API-Source", "Minimax-MCP")
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
            voice_type: voice_type.map(String::from).or_else(|| Some("system".to_string())),
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
    // Token Plan
    // ============================================================

    /// GET /v1/token_plan/remains — query token usage and plan balance.
    pub async fn get_token_plan_remains(
        &self,
    ) -> Result<TokenPlanResponse, MiniMaxError> {
        self.get_json("/v1/token_plan/remains").await
    }

    // ============================================================
    // Chat — Anthropic-compatible
    // ============================================================

    /// POST /v1/messages — Anthropic-compatible chat.
    ///
    /// Does NOT use `post_json` because this endpoint returns
    /// pure Anthropic format without `base_resp` wrapper.
    pub async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse, MiniMaxError> {
        let url = format!("{}/anthropic/v1/messages", self.base_url);
        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("MM-API-Source", "Minimax-MCP")
            .json(req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(MiniMaxError::Api {
                code: status as i32,
                message: body,
                trace_id: None,
            });
        }

        Ok(response.json().await?)
    }

    // ============================================================
    // Lyrics Generation
    // ============================================================

    /// POST /v1/lyrics_generation — generate song lyrics.
    pub async fn generate_lyrics(
        &self,
        req: &LyricsGenerationRequest,
    ) -> Result<LyricsGenerationResponse, MiniMaxError> {
        self.post_json("/v1/lyrics_generation", req).await
    }

    // ============================================================
    // Search — POST /v1/coding_plan/search
    // ============================================================

    /// POST /v1/coding_plan/search — web search via MiniMax Coding Plan.
    pub async fn search(&self, req: &SearchRequest) -> Result<SearchResponse, MiniMaxError> {
        self.post_json("/v1/coding_plan/search", req).await
    }

    // ============================================================
    // VLM — POST /v1/coding_plan/vlm
    // ============================================================

    /// POST /v1/coding_plan/vlm — vision language model for image understanding.
    pub async fn vlm(&self, req: &VlmRequest) -> Result<VlmResponse, MiniMaxError> {
        self.post_json("/v1/coding_plan/vlm", req).await
    }

    // ============================================================
    // Async TTS — POST /v1/t2a_async_v2
    // ============================================================

    /// POST /v1/t2a_async_v2 — submit an async text-to-speech task.
    pub async fn create_async_tts(
        &self,
        req: &T2AAsyncRequest,
    ) -> Result<T2AAsyncCreateResponse, MiniMaxError> {
        self.post_json("/v1/t2a_async_v2", req).await
    }

    /// GET /v1/query/t2a_async_query_v2?task_id= — query async TTS task status.
    pub async fn query_async_tts(
        &self,
        task_id: i64,
    ) -> Result<T2AAsyncQueryResponse, MiniMaxError> {
        let endpoint = format!("/v1/query/t2a_async_query_v2?task_id={}", task_id);
        self.get_json(&endpoint).await
    }

    /// Poll `/v1/files/retrieve?file_id=` until the download URL is available.
    ///
    /// Returns the download_url when ready. Handles 2013 (file not found) as
    /// "not ready yet" rather than an error.
    pub async fn poll_file_download_url(
        &self,
        file_id: i64,
        max_retries: i32,
        interval: u64,
    ) -> Result<String, MiniMaxError> {
        let endpoint = format!("/v1/files/retrieve?file_id={}", file_id);
        let url = format!("{}{}", self.base_url, endpoint);

        for attempt in 0..max_retries {
            let response = self
                .http
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("MM-API-Source", "Minimax-MCP")
                .send()
                .await?;

            let value: serde_json::Value = response.json().await?;

            // Check base_resp manually to handle 2013 as "not ready"
            let code = value
                .get("base_resp")
                .and_then(|b| b.get("status_code"))
                .and_then(|v| v.as_i64())
                .unwrap_or(-1);

            if code == 2013 {
                info!(
                    "Async TTS file {} not ready yet (attempt {}/{})",
                    file_id,
                    attempt + 1,
                    max_retries
                );
                sleep(Duration::from_secs(interval)).await;
                continue;
            }
            if code != 0 {
                let message = value
                    .get("base_resp")
                    .and_then(|b| b.get("status_msg"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error");
                return Err(MiniMaxError::Api {
                    code: code as i32,
                    message: message.to_string(),
                    trace_id: None,
                });
            }

            let resp: FileRetrieveResponse = serde_json::from_value(value)?;
            if let Some(file) = resp.file {
                return Ok(file.download_url);
            }
        }

        Err(MiniMaxError::TaskTimeout {
            task_id: file_id.to_string(),
            max_retries,
        })
    }

    /// Submit async TTS, poll for completion, download tar, extract mp3 bytes.
    ///
    /// Returns the extracted mp3 file bytes.
    pub async fn async_tts_and_extract_mp3(
        &self,
        req: &T2AAsyncRequest,
    ) -> Result<Vec<u8>, MiniMaxError> {
        let task = self.create_async_tts(req).await?;
        info!("Async TTS task created: {}", task.task_id);

        let download_url = self
            .poll_file_download_url(
                task.file_id,
                ASYNC_TTS_MAX_POLL_RETRIES,
                ASYNC_TTS_POLL_INTERVAL_SECS,
            )
            .await?;

        info!("Downloading async TTS result tar...");
        let tar_bytes = self
            .http
            .get(&download_url)
            .send()
            .await?
            .bytes()
            .await?;

        // Extract the mp3 from the tar archive
        let mut tar = tar::Archive::new(std::io::Cursor::new(&tar_bytes));
        for entry in tar.entries().map_err(|e| MiniMaxError::Api {
            code: -1,
            message: format!("tar read error: {e}"),
            trace_id: None,
        })? {
            let mut entry = entry.map_err(|e| MiniMaxError::Api {
                code: -1,
                message: format!("tar entry error: {e}"),
                trace_id: None,
            })?;
            let path = entry.path().map_err(|e| MiniMaxError::Api {
                code: -1,
                message: format!("tar path error: {e}"),
                trace_id: None,
            })?;
            let name = path.to_string_lossy();
            if name.ends_with(".mp3") {
                let mut out = Vec::new();
                std::io::copy(&mut entry, &mut out).map_err(|e| MiniMaxError::Api {
                    code: -1,
                    message: format!("tar extract error: {e}"),
                    trace_id: None,
                })?;
                return Ok(out);
            }
        }

        Err(MiniMaxError::Api {
            code: -1,
            message: "no mp3 found in tar".to_string(),
            trace_id: None,
        })
    }

    // ============================================================
    // Music Cover
    // ============================================================

    /// POST /v1/music_cover_preprocess — preprocess audio for cover generation.
    pub async fn preprocess_music_cover(
        &self,
        audio_url: &str,
    ) -> Result<MusicCoverPreprocessResponse, MiniMaxError> {
        let req = MusicCoverPreprocessRequest {
            audio_url: audio_url.to_string(),
        };
        self.post_json("/v1/music_cover_preprocess", &req).await
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
            .header("MM-API-Source", "Minimax-MCP")
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

    /// Download raw bytes from a URL.
    pub async fn download_bytes(&self, url: &str) -> Result<Vec<u8>, MiniMaxError> {
        let bytes = self.http.get(url).send().await?.bytes().await?;
        Ok(bytes.to_vec())
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
