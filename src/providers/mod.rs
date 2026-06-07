//! Provider traits — generic output types, error type, and factory.
//!
//! This module defines the common types shared by all provider implementations
//! and the `create_provider()` factory that reads `provider.toml`.

use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;
use thiserror::Error;

// ============================================================
// ProviderError
// ============================================================

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("auth: {0}")]
    Auth(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("HTTP: {0}")]
    Http(String),

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("task failed: {0}")]
    TaskFailed(String),

    #[error("config: {0}")]
    Config(String),

    #[error("IO: {0}")]
    Io(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("not supported: {0}")]
    NotSupported(String),

    #[error("{0}")]
    Other(String),
}

impl From<crate::error::MiniMaxError> for ProviderError {
    fn from(e: crate::error::MiniMaxError) -> Self {
        match e {
            crate::error::MiniMaxError::Auth(msg) => ProviderError::Auth(msg),
            crate::error::MiniMaxError::Api { code, message, .. } => {
                ProviderError::Api(format!("code={code}: {message}"))
            }
            crate::error::MiniMaxError::Http(err) => ProviderError::Http(err.to_string()),
            crate::error::MiniMaxError::TaskTimeout { task_id, max_retries } => {
                ProviderError::Timeout(format!("task {task_id} after {max_retries} retries"))
            }
            crate::error::MiniMaxError::TaskFailed { task_id } => {
                ProviderError::TaskFailed(task_id)
            }
            crate::error::MiniMaxError::Json(err) => ProviderError::Other(err.to_string()),
            crate::error::MiniMaxError::Io(err) => ProviderError::Io(err.to_string()),
            crate::error::MiniMaxError::HexDecode(msg) => ProviderError::Other(msg),
            crate::error::MiniMaxError::Config(msg) => ProviderError::Config(msg),
            crate::error::MiniMaxError::MissingEnv(msg) => ProviderError::Config(msg),
            crate::error::MiniMaxError::InvalidPath(msg) => ProviderError::Config(msg),
        }
    }
}

impl From<std::io::Error> for ProviderError {
    fn from(e: std::io::Error) -> Self {
        ProviderError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(e: serde_json::Error) -> Self {
        ProviderError::Other(e.to_string())
    }
}

// ============================================================
// Generic Output Types
// ============================================================

/// Binary or URL media from generation operations.
#[derive(Debug, Clone)]
pub enum MediaOutput {
    Bytes { data: Vec<u8>, extension: String },
    Url(String),
}

/// Handle returned by async task submission (video, async TTS).
#[derive(Debug, Clone, serde::Serialize)]
pub struct AsyncTaskHandle {
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

/// Result of polling an async task.
#[derive(Debug, Clone)]
pub enum AsyncTaskResult {
    Pending { status: String },
    Completed(MediaOutput),
    Failed { message: String },
}

// --- Voice ---

#[derive(Debug, Clone)]
pub struct VoiceInfoOutput {
    pub voice_id: String,
    pub voice_name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct VoiceDesignedOutput {
    pub voice_id: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct VoiceListResult {
    pub system: Vec<VoiceInfoOutput>,
    pub cloned: Vec<VoiceInfoOutput>,
    pub designed: Vec<VoiceDesignedOutput>,
}

#[derive(Debug, Clone)]
pub struct VoiceCloneResult {
    pub voice_id: String,
    pub demo_audio: Option<MediaOutput>,
}

// --- Chat ---

#[derive(Debug, Clone)]
pub struct ChatOutput {
    pub text: String,
    pub model: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
}

// --- Lyrics ---

#[derive(Debug, Clone)]
pub struct LyricsResult {
    pub title: Option<String>,
    pub style_tags: Option<String>,
    pub lyrics: Option<String>,
}

// --- Search ---

#[derive(Debug, Clone)]
pub struct SearchResultItem {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub date: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SearchOutput {
    pub results: Vec<SearchResultItem>,
    pub related: Vec<String>,
}

// --- Files ---

#[derive(Debug, Clone)]
pub struct FileInfoResult {
    pub file_id: Option<i64>,
    pub filename: Option<String>,
    pub bytes: Option<i64>,
    pub purpose: Option<String>,
    pub download_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileListResult {
    pub files: Vec<FileInfoResult>,
}

#[derive(Debug, Clone)]
pub struct FileUploadResult {
    pub file_id: i64,
}

// --- Usage ---

#[derive(Debug, Clone)]
pub struct UsageResult {
    pub fields: HashMap<String, serde_json::Value>,
}

// ============================================================
// Configuration (provider.toml)
// ============================================================

#[derive(Debug, Deserialize)]
struct ProviderToml {
    providers: HashMap<String, String>,
    #[serde(default)]
    provider_config: HashMap<String, ProviderSettings>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderSettings {
    pub api_key_env: String,
    pub api_host: Option<String>,
}

/// Read `provider.toml` from the given path. Returns `None` when the file
/// doesn't exist, which signals "use defaults".
fn load_provider_toml(path: &std::path::Path) -> Result<Option<ProviderToml>, ProviderError> {
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(path).map_err(|e| ProviderError::Config(format!(
        "cannot read {}: {e}",
        path.display()
    )))?;
    let config: ProviderToml = toml::from_str(&content).map_err(|e| ProviderError::Config(format!(
        "invalid {}: {e}",
        path.display()
    )))?;
    Ok(Some(config))
}

/// Return the provider name for a capability.
fn resolve_provider(toml: &Option<ProviderToml>, key: &str) -> String {
    toml.as_ref()
        .and_then(|t| t.providers.get(key).cloned())
        .unwrap_or_else(|| "minimax".to_string())
}

/// Return the `ProviderSettings` for a named provider.  When the provider is
/// "minimax" and no explicit config block exists, return the default settings.
fn resolve_settings(toml: &Option<ProviderToml>, name: &str) -> ProviderSettings {
    if let Some(settings) = toml.as_ref().and_then(|t| t.provider_config.get(name)) {
        return ProviderSettings {
            api_key_env: settings.api_key_env.clone(),
            api_host: settings.api_host.clone(),
        };
    }
    // Defaults for minimax
    ProviderSettings {
        api_key_env: "MINIMAX_API_KEY".to_string(),
        api_host: None,
    }
}

/// Build a `MiniMaxClient` from settings.
fn create_minimax_client(settings: &ProviderSettings) -> Result<crate::MiniMaxClient, ProviderError> {
    let api_key = std::env::var(&settings.api_key_env).map_err(|_| {
        ProviderError::Auth(format!(
            "environment variable {} not set",
            settings.api_key_env
        ))
    })?;
    let base_url = settings
        .api_host
        .clone()
        .or_else(|| std::env::var("MINIMAX_API_HOST").ok())
        .unwrap_or_else(|| crate::consts::DEFAULT_API_HOST.to_string());
    Ok(crate::MiniMaxClient::new(api_key, base_url))
}

// ============================================================
// Re-export the providers that are available
// ============================================================

mod minimax;
pub use minimax::MiniMaxProvider;

// ============================================================
// Factory
// ============================================================

use crate::tools::{chat, files, image, music, search, tts, usage, video};

/// Holds all capability trait objects, each independently selectable.
pub struct ProviderSet {
    pub tts: Arc<dyn tts::TtsProvider>,
    pub voice: Arc<dyn tts::VoiceProvider>,
    pub video: Arc<dyn video::VideoProvider>,
    pub image: Arc<dyn image::ImageProvider>,
    pub music: Arc<dyn music::MusicProvider>,
    pub chat: Arc<dyn chat::ChatProvider>,
    pub search: Arc<dyn search::SearchProvider>,
    pub files: Arc<dyn files::FileProvider>,
    pub usage: Arc<dyn usage::UsageProvider>,
}

/// Create the full `ProviderSet` by reading `provider.toml`.
///
/// Falls back to minimax + env-var key for every capability when the config
/// file is missing.
pub fn create_provider_set() -> Result<ProviderSet, ProviderError> {
    let toml = load_provider_toml(std::path::Path::new("provider.toml"))?;

    let minimax: Option<Arc<MiniMaxProvider>> = {
        let name = resolve_provider(&toml, "tts"); // any key works to trigger init
        if name == "minimax" {
            let settings = resolve_settings(&toml, "minimax");
            let client = create_minimax_client(&settings)?;
            Some(Arc::new(MiniMaxProvider::new(client)))
        } else {
            None
        }
    };

    // Helper: get Arc<MiniMaxProvider> for a given capability.
    // Returns an error if the provider name is not "minimax".
    fn get_minimax(
        p: &Option<Arc<MiniMaxProvider>>,
        toml: &Option<ProviderToml>,
        key: &str,
    ) -> Result<Arc<MiniMaxProvider>, ProviderError> {
        let name = resolve_provider(toml, key);
        if name != "minimax" {
            return Err(ProviderError::NotSupported(format!(
                "unknown provider '{name}' for {key}"
            )));
        }
        p.clone().ok_or_else(|| {
            ProviderError::Config(format!("minimax provider not initialised for {key}"))
        })
    }

    let m = &minimax;

    Ok(ProviderSet {
        tts: get_minimax(m, &toml, "tts")?,
        voice: get_minimax(m, &toml, "voice")?,
        video: get_minimax(m, &toml, "video")?,
        image: get_minimax(m, &toml, "image")?,
        music: get_minimax(m, &toml, "music")?,
        chat: get_minimax(m, &toml, "chat")?,
        search: get_minimax(m, &toml, "search")?,
        files: get_minimax(m, &toml, "files")?,
        usage: get_minimax(m, &toml, "usage")?,
    })
}
