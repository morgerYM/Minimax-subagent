use std::path::PathBuf;
use std::time::SystemTime;

use crate::error::MiniMaxError;

/// 处理 `~`，创建目录，返回规范化的 PathBuf。
pub fn resolve_and_create_dir(path: &str) -> Result<PathBuf, MiniMaxError> {
    let expanded = if path.starts_with('~') {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        path.replacen('~', &home, 1)
    } else {
        path.to_string()
    };
    let dir = PathBuf::from(&expanded);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Hex 字符串 → bytes。
pub fn decode_hex_audio(hex_str: &str) -> Result<Vec<u8>, MiniMaxError> {
    hex::decode(hex_str).map_err(|e| MiniMaxError::HexDecode(e.to_string()))
}

/// `{tool}_{text前10字符}_{epoch_millis}.{ext}`
/// 空格替换为 `_`，Unicode 安全截断。
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
