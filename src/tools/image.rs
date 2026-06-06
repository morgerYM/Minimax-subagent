//! Image generation tool handlers.
//!
//! Provides image generation via MiniMax Image API.

use minimax_api::consts::*;
use minimax_api::types::*;
use minimax_api::utils;
use minimax_api::MiniMaxClient;

use crate::mcp_params::*;
use crate::to_mcp_err;

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

pub async fn handle_generate_image(
    client: &MiniMaxClient,
    params: GenerateImageParams,
) -> Result<CallToolResult, ErrorData> {
    let req = ImageGenerationRequest {
        model: params.model.unwrap_or_else(|| DEFAULT_IMAGE_MODEL.to_string()),
        prompt: params.prompt,
        aspect_ratio: params.aspect_ratio,
        n: params.n,
        prompt_optimizer: params.prompt_optimizer,
        width: params.width,
        height: params.height,
        response_format: params.response_format,
        seed: params.seed,
        aigc_watermark: params.aigc_watermark,
        subject_reference: None,
        style: params.style_type.map(|st| ImageStyle {
            style_type: st,
            style_weight: params.style_weight,
        }),
    };

    let resp = client.generate_image(&req).await.map_err(to_mcp_err)?;

    if params.output_directory.is_some() || params.output_file.is_some() {
        if let Some(data) = &resp.data {
            let mut saved = Vec::new();
            let total = data.image_urls.len();
            let user_file = params.output_file.as_deref();
            let user_dir = params.output_directory.as_deref();
            for (i, url) in data.image_urls.iter().enumerate() {
                // When user provides a single `output_file`, append `_{i}` to
                // the stem when n>1, preserving the extension (default jpg).
                // Otherwise, fall back to per-index auto-naming in user_dir.
                let per_file = if let Some(f) = user_file {
                    let p = std::path::Path::new(f);
                    let parent = p.parent();
                    let stem = p
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("image");
                    let ext = p
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("jpg");
                    let name = if total > 1 {
                        format!("{stem}_{i}.{ext}")
                    } else {
                        format!("{stem}.{ext}")
                    };
                    // Reattach parent dir so the full path is preserved.
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
                    &format!("{}_{}", i, req.prompt),
                    "jpg",
                )
                .map_err(to_mcp_err)?;
                if let Some(path) = path {
                    client
                        .download_to_path(url, &path)
                        .await
                        .map_err(to_mcp_err)?;
                    saved.push(path.display().to_string());
                }
            }
            if !saved.is_empty() {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Image saved to:\n{}",
                    saved.join("\n")
                ))]));
            }
        }
    }

    if let Some(data) = &resp.data {
        let urls = data.image_urls.join("\n");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Image generated! {} image(s):\n{}",
            data.image_urls.len(),
            urls
        ))]))
    } else {
        Ok(CallToolResult::success(vec![Content::text(
            "图像生成成功，但未返回数据。".to_string(),
        )]))
    }
}
