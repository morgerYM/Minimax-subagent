//! Image generation tool handlers.

use async_trait::async_trait;

use crate::mcp_params::GenerateImageParams;
use crate::providers::{MediaOutput, ProviderError};
use crate::utils;

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

// ============================================================
// Trait
// ============================================================

#[async_trait]
pub trait ImageProvider: Send + Sync {
    async fn generate_image(&self, params: &GenerateImageParams) -> Result<Vec<MediaOutput>, ProviderError>;
}

// ============================================================
// Handler
// ============================================================

fn to_mcp_err(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

pub async fn handle_generate_image(
    provider: &dyn ImageProvider,
    params: GenerateImageParams,
) -> Result<CallToolResult, ErrorData> {
    let outputs = provider.generate_image(&params).await.map_err(to_mcp_err)?;

    let user_file = params.output_file.as_deref();
    let user_dir = params.output_directory.as_deref();
    let total = outputs.len();

    if user_file.is_some() || user_dir.is_some() {
        let mut saved = Vec::new();
        for (i, output) in outputs.iter().enumerate() {
            match output {
                MediaOutput::Url(url) => {
                    let per_file = if let Some(f) = user_file {
                        let p = std::path::Path::new(f);
                        let parent = p.parent();
                        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
                        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("jpg");
                        let name = if total > 1 {
                            format!("{stem}_{i}.{ext}")
                        } else {
                            format!("{stem}.{ext}")
                        };
                        match parent {
                            Some(par) if !par.as_os_str().is_empty() => {
                                Some(par.join(name).to_string_lossy().to_string())
                            }
                            _ => Some(name),
                        }
                    } else {
                        None
                    };
                    let path = utils::resolve_output_file(
                        per_file.as_deref(),
                        user_dir,
                        "image",
                        &format!("{}_{}", i, params.prompt),
                        "jpg",
                    )
                    .map_err(to_mcp_err)?;
                    if let Some(path) = path {
                        // download URL to path
                        let bytes = reqwest::get(url).await.map_err(|e| {
                            ErrorData::internal_error(format!("download: {e}"), None)
                        })?.bytes().await.map_err(|e| {
                            ErrorData::internal_error(format!("download: {e}"), None)
                        })?;
                        tokio::fs::write(&path, &bytes).await.map_err(to_mcp_err)?;
                        saved.push(path.display().to_string());
                    }
                }
                MediaOutput::Bytes { data, extension } => {
                    if let Some(path) = utils::write_output_file(
                        None,
                        user_dir,
                        "image",
                        &format!("{}_{}", i, params.prompt),
                        extension,
                        data,
                    )
                    .await
                    .map_err(to_mcp_err)?
                    {
                        saved.push(path.display().to_string());
                    }
                }
            }
        }
        if !saved.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Image saved to:\n{}",
                saved.join("\n")
            ))]));
        }
    }

    let urls: Vec<String> = outputs
        .iter()
        .enumerate()
        .map(|(i, o)| match o {
            MediaOutput::Url(u) => format!("{}. {}", i + 1, u),
            MediaOutput::Bytes { data, .. } => {
                format!("{}. [{} bytes]", i + 1, data.len())
            }
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Image generated! {} image(s):\n{}",
        outputs.len(),
        urls.join("\n")
    ))]))
}
