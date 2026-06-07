//! ImageProvider impl for MiniMaxProvider.

use async_trait::async_trait;

use crate::consts::*;
use crate::mcp_params::GenerateImageParams;
use crate::providers::*;
use crate::types::{ImageGenerationRequest, ImageStyle};

use super::MiniMaxProvider;

#[async_trait]
impl crate::tools::image::ImageProvider for MiniMaxProvider {
    async fn generate_image(&self, params: &GenerateImageParams) -> Result<Vec<MediaOutput>, ProviderError> {
        let req = ImageGenerationRequest {
            model: params.model.clone().unwrap_or_else(|| DEFAULT_IMAGE_MODEL.to_string()),
            prompt: params.prompt.clone(),
            aspect_ratio: params.aspect_ratio.clone(),
            n: params.n,
            prompt_optimizer: params.prompt_optimizer,
            width: params.width,
            height: params.height,
            response_format: params.response_format.clone(),
            seed: params.seed,
            aigc_watermark: params.aigc_watermark,
            subject_reference: None,
            style: params.style_type.clone().map(|st| ImageStyle {
                style_type: st,
                style_weight: params.style_weight,
            }),
        };

        let resp = self.client.generate_image(&req).await?;

        match resp.data {
            Some(data) => {
                let outputs: Vec<MediaOutput> = data.image_urls.into_iter()
                    .map(|url| MediaOutput::Url(url))
                    .collect();
                Ok(outputs)
            }
            None => Ok(vec![]),
        }
    }
}
