use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

use reqwest::Client;
use tracing::info;

use crate::error::MiniMaxError;

/// Resolve `~` in path, create directory if needed, return normalized PathBuf.
pub fn resolve_and_create_dir(path: &str) -> Result<PathBuf, MiniMaxError> {
    let expanded = if path.starts_with('~') {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        path.replacen('~', &home, 1)
    } else {
        path.to_string()
    };
    let dir = PathBuf::from(&expanded);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Convert hex string to bytes.
pub fn decode_hex_audio(hex_str: &str) -> Result<Vec<u8>, MiniMaxError> {
    hex::decode(hex_str).map_err(|e| MiniMaxError::HexDecode(e.to_string()))
}

/// Format: `{tool}_{first_10_chars_of_text}_{epoch_millis}.{ext}`
/// Spaces become `_`, Unicode-safe truncation.
pub fn build_filename(tool: &str, text: &str, ext: &str) -> String {
    let prefix: String = text
        .chars()
        .take(10)
        .map(|c| if c == ' ' { '_' } else { c })
        .collect();
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{}_{}_{}.{}", tool, prefix, ts, ext)
}

/// Process image source (URL or local path):
/// - Strip `@` prefix
/// - HTTP/HTTPS: download and convert to base64 data URL
/// - Local file: read and convert to base64 data URL
pub async fn process_image_url(image_source: &str) -> String {
    let trimmed = image_source.trim_start_matches('@');

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        // Download HTTP/HTTPS URL and convert to base64
        match download_image(trimmed).await {
            Ok(data_url) => data_url,
            Err(e) => {
                tracing::warn!("Failed to download image, using original URL: {}", e);
                trimmed.to_string()
            }
        }
    } else if trimmed.starts_with("data:") {
        // Already a data URL, pass through
        trimmed.to_string()
    } else {
        // Local file to base64 data URL
        let path = PathBuf::from(trimmed);
        if path.exists() {
            match fs::read(&path) {
                Ok(data) => {
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("png");
                    let mime = match ext {
                        "jpg" | "jpeg" => "image/jpeg",
                        "png" => "image/png",
                        "webp" => "image/webp",
                        "gif" => "image/gif",
                        _ => "image/png",
                    };
                    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
                    format!("data:{};base64,{}", mime, encoded)
                }
                Err(_) => trimmed.to_string(),
            }
        } else {
            trimmed.to_string()
        }
    }
}

async fn download_image(url: &str) -> Result<String, MiniMaxError> {
    info!("Downloading image from: {}", url);
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| MiniMaxError::Api {
            code: -1,
            message: format!("Failed to create HTTP client: {}", e),
            trace_id: None,
        })?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| MiniMaxError::Api {
            code: -1,
            message: format!("Failed to download image: {}", e),
            trace_id: None,
        })?;

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg");

    let image_format = if content_type.contains("png") {
        "png"
    } else if content_type.contains("webp") {
        "webp"
    } else if content_type.contains("gif") {
        "gif"
    } else {
        "jpeg"
    };

    let bytes = response
        .bytes()
        .await
        .map_err(|e| MiniMaxError::Api {
            code: -1,
            message: format!("Failed to read image bytes: {}", e),
            trace_id: None,
        })?;

    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    Ok(format!("data:image/{};base64,{}", image_format, encoded))
}

/// Save hex audio to $TMPDIR/minimax/ and play via afplay.
/// Silently ignores playback errors.
pub fn save_and_play_audio(hex: &str, tool: &str) -> Result<PathBuf, MiniMaxError> {
    let bytes = decode_hex_audio(hex)?;
    let filename = build_filename(tool, "audio", "mp3");
    let filepath = PathBuf::from(&filename);

    std::fs::write(&filepath, &bytes)?;

    // Blocking playback — waits for audio to finish before returning.
    let status = Command::new("afplay").arg(&filepath).status();
    if status.is_err() {
        eprintln!("[Warning] afplay playback failed");
    }

    Ok(filepath)
}

/// Save raw bytes to $TMPDIR/minimax/ and play via afplay.
pub fn save_and_play_audio_bytes(bytes: &[u8], prefix: &str) -> Result<PathBuf, MiniMaxError> {
    let filename = build_filename(prefix, "audio", "mp3");
    let filepath = PathBuf::from(&filename);

    std::fs::write(&filepath, bytes)?;

    let status = Command::new("afplay").arg(&filepath).status();
    if status.is_err() {
        eprintln!("[Warning] afplay playback failed");
    }

    Ok(filepath)
}

/// Download image to current directory and open with `open`.
pub async fn download_and_open_image(url: &str, tool: &str) -> Result<PathBuf, MiniMaxError> {
    use std::time::Duration;

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| MiniMaxError::Api {
            code: -1,
            message: format!("Failed to create HTTP client: {}", e),
            trace_id: None,
        })?;
    let bytes = client
        .get(url)
        .send()
        .await
        .map_err(|e| MiniMaxError::Api {
            code: -1,
            message: format!("Failed to download image: {}", e),
            trace_id: None,
        })?
        .bytes()
        .await
        .map_err(|e| MiniMaxError::Api {
            code: -1,
            message: format!("Failed to read image bytes: {}", e),
            trace_id: None,
        })?;

    let ext = if url.contains(".png") {
        "png"
    } else {
        "jpg"
    };
    let filename = build_filename(tool, "image", ext);
    let filepath = PathBuf::from(&filename);

    tokio::fs::write(&filepath, &bytes).await?;

    // Non-blocking open — does not wait for the app to launch.
    let filepath_clone = filepath.clone();
    tokio::spawn(async move {
        if Command::new("open").arg(&filepath_clone).spawn().is_err() {
            eprintln!("[Warning] open failed");
        }
    });

    Ok(filepath)
}
