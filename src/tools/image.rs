//! Image generation tool handlers.
//!
//! Provides image generation via MiniMax Image API.

use minimax_api::consts::*;
use minimax_api::types::*;
use minimax_api::utils;
use minimax_api::MiniMaxClient;

use minimax_api::mcp_params::*;
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

    if let Some(dir) = &params.output_directory {
        if let Some(data) = &resp.data {
            let path = utils::resolve_and_create_dir(dir).map_err(to_mcp_err)?;
            let mut saved = Vec::new();
            for (i, url) in data.image_urls.iter().enumerate() {
                let filename = utils::build_filename(
                    "image",
                    &format!("{}_{}", i, req.prompt),
                    "jpg",
                );
                let filepath = path.join(&filename);
                client
                    .download_to_path(url, &filepath)
                    .await
                    .map_err(to_mcp_err)?;
                saved.push(filepath.display().to_string());
            }
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Image saved to:\n{}",
                saved.join("\n")
            ))]));
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
