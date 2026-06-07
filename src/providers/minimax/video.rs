//! VideoProvider impl for MiniMaxProvider.

use async_trait::async_trait;

use crate::consts::*;
use crate::mcp_params::*;
use crate::providers::*;
use crate::types::*;

use super::MiniMaxProvider;

#[async_trait]
impl crate::tools::video::VideoProvider for MiniMaxProvider {
    async fn create_video(&self, params: &GenerateVideoParams) -> Result<AsyncTaskHandle, ProviderError> {
        let subject_reference = params.subject_reference.as_ref().map(|vals| {
            vals.iter()
                .filter_map(|v| {
                    let reference_type = v.get("type")?.as_str()?.to_string();
                    let image = v.get("image")?
                        .as_array()?
                        .iter()
                        .filter_map(|s| s.as_str().map(String::from))
                        .collect();
                    Some(SubjectReference { reference_type, image })
                })
                .collect()
        });

        let req = VideoGenerationRequest {
            model: params.model.clone().unwrap_or_else(|| DEFAULT_VIDEO_MODEL.to_string()),
            prompt: params.prompt.clone(),
            first_frame_image: params.first_frame_image.clone(),
            last_frame_image: params.last_frame_image.clone(),
            subject_reference,
            duration: params.duration,
            resolution: params.resolution.clone(),
            prompt_optimizer: params.prompt_optimizer,
            fast_pretreatment: params.fast_pretreatment,
            callback_url: params.callback_url.clone(),
            aigc_watermark: params.aigc_watermark,
        };

        let resp = self.client.create_video(&req).await?;
        Ok(AsyncTaskHandle {
            task_id: resp.task_id,
            extra: None,
        })
    }

    async fn query_video(&self, task_id: &str) -> Result<AsyncTaskResult, ProviderError> {
        let resp = self.client.query_video(task_id).await?;

        match resp.status.as_str() {
            "Success" => {
                let file_id = resp.file_id.as_deref().unwrap_or("N/A");
                let download_url = self.client.get_file_download_url(file_id).await?;
                Ok(AsyncTaskResult::Completed(MediaOutput::Url(download_url)))
            }
            "Fail" => Ok(AsyncTaskResult::Failed {
                message: "Video generation failed".into(),
            }),
            status => Ok(AsyncTaskResult::Pending {
                status: status.to_string(),
            }),
        }
    }

    async fn generate_video_and_wait(&self, params: &GenerateVideoParams) -> Result<MediaOutput, ProviderError> {
        let bytes = self.client.generate_video_and_download(&build_video_req(params)).await?;
        Ok(MediaOutput::Bytes { data: bytes, extension: "mp4".to_string() })
    }

    async fn create_video_agent(&self, params: &GenerateVideoAgentParams) -> Result<AsyncTaskHandle, ProviderError> {
        let req = VideoTemplateGenerationRequest {
            template_id: params.template_id.clone(),
            text_inputs: params.text_inputs.as_ref().map(|vals| {
                vals.iter().map(|v| TextInput {
                    value: v["value"].as_str().unwrap_or_default().to_string(),
                }).collect()
            }),
            media_inputs: params.media_inputs.as_ref().map(|vals| {
                vals.iter().map(|v| MediaInput {
                    value: v["value"].as_str().unwrap_or_default().to_string(),
                }).collect()
            }),
            callback_url: params.callback_url.clone(),
        };

        let resp = self.client.create_video_template(&req).await?;
        Ok(AsyncTaskHandle {
            task_id: resp.task_id.unwrap_or_else(|| "N/A".to_string()),
            extra: None,
        })
    }

    async fn query_video_agent(&self, task_id: &str) -> Result<AsyncTaskResult, ProviderError> {
        let resp = self.client.query_video_template(task_id).await?;

        match resp.status.as_str() {
            "Success" => {
                let url = resp.video_url.unwrap_or_default();
                if url.is_empty() {
                    Ok(AsyncTaskResult::Completed(MediaOutput::Url("N/A".to_string())))
                } else {
                    Ok(AsyncTaskResult::Completed(MediaOutput::Url(url)))
                }
            }
            "Fail" => Ok(AsyncTaskResult::Failed {
                message: "Video Agent task failed".into(),
            }),
            status => Ok(AsyncTaskResult::Pending {
                status: status.to_string(),
            }),
        }
    }
}

fn build_video_req(params: &GenerateVideoParams) -> VideoGenerationRequest {
    let subject_reference = params.subject_reference.as_ref().map(|vals| {
        vals.iter()
            .filter_map(|v| {
                let reference_type = v.get("type")?.as_str()?.to_string();
                let image = v.get("image")?
                    .as_array()?
                    .iter()
                    .filter_map(|s| s.as_str().map(String::from))
                    .collect();
                Some(SubjectReference { reference_type, image })
            })
            .collect()
    });

    VideoGenerationRequest {
        model: params.model.clone().unwrap_or_else(|| DEFAULT_VIDEO_MODEL.to_string()),
        prompt: params.prompt.clone(),
        first_frame_image: params.first_frame_image.clone(),
        last_frame_image: params.last_frame_image.clone(),
        subject_reference,
        duration: params.duration,
        resolution: params.resolution.clone(),
        prompt_optimizer: params.prompt_optimizer,
        fast_pretreatment: params.fast_pretreatment,
        callback_url: params.callback_url.clone(),
        aigc_watermark: params.aigc_watermark,
    }
}
