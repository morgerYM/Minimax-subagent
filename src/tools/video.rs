//! Video generation tool handlers.
//!
//! Provides video generation and query via MiniMax Video API,
//! including async task submission and polling.

use minimax_api::consts::*;
use minimax_api::types::*;
use minimax_api::utils;
use minimax_api::MiniMaxClient;

use minimax_api::mcp_params::*;
use crate::to_mcp_err;

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

pub async fn handle_generate_video(
    client: &MiniMaxClient,
    params: GenerateVideoParams,
) -> Result<CallToolResult, ErrorData> {
    let subject_reference = params.subject_reference.map(|vals| {
        vals.into_iter()
            .filter_map(|v| {
                let reference_type = v.get("type")?.as_str()?.to_string();
                let image = v.get("image")?
                    .as_array()?
                    .iter()
                    .filter_map(|s| s.as_str().map(String::from))
                    .collect::<Vec<_>>();
                Some(SubjectReference {
                    reference_type,
                    image,
                })
            })
            .collect()
    });

    let req = VideoGenerationRequest {
        model: params.model.unwrap_or_else(|| DEFAULT_VIDEO_MODEL.to_string()),
        prompt: params.prompt,
        first_frame_image: params.first_frame_image,
        last_frame_image: params.last_frame_image,
        subject_reference,
        duration: params.duration,
        resolution: params.resolution,
        prompt_optimizer: params.prompt_optimizer,
        fast_pretreatment: params.fast_pretreatment,
        callback_url: params.callback_url,
        aigc_watermark: params.aigc_watermark,
    };

    let async_mode = params.async_mode.unwrap_or(true);

    if async_mode {
        let resp = client.create_video(&req).await.map_err(to_mcp_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Video task submitted!\ntask_id: {}\nUse query_video to poll progress.",
            resp.task_id
        ))]))
    } else {
        let bytes = client
            .generate_video_and_download(&req)
            .await
            .map_err(to_mcp_err)?;
        if params.output_directory.is_some() || params.output_file.is_some() {
            if let Some(path) = utils::write_output_file(
                params.output_file.as_deref(),
                params.output_directory.as_deref(),
                "video",
                &req.prompt,
                "mp4",
                &bytes,
            )
            .await
            .map_err(to_mcp_err)?
            {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Video generated! Saved to: {}",
                    path.display()
                ))]));
            }
        }
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Video generated! Size: {:.1} MB",
            bytes.len() as f64 / 1_048_576.0
        ))]))
    }
}

pub async fn handle_query_video(
    client: &MiniMaxClient,
    params: QueryVideoParams,
) -> Result<CallToolResult, ErrorData> {
    let resp = client.query_video(&params.task_id).await.map_err(to_mcp_err)?;

    match resp.status.as_str() {
        "Success" => {
            let file_id = resp.file_id.as_deref().unwrap_or("N/A");
            let download_url = client
                .get_file_download_url(file_id)
                .await
                .map_err(to_mcp_err)?;
            if params.output_directory.is_some() || params.output_file.is_some() {
                if let Some(path) = utils::resolve_output_file(
                    params.output_file.as_deref(),
                    params.output_directory.as_deref(),
                    "video",
                    &params.task_id,
                    "mp4",
                )
                .map_err(to_mcp_err)?
                {
                    client
                        .download_to_path(&download_url, &path)
                        .await
                        .map_err(to_mcp_err)?;
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "Video generated! Saved to: {}",
                        path.display()
                    ))]));
                }
            }
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Video generated!\nfile_id: {}\ndownload_url: {}",
                file_id, download_url
            ))]))
        }
        "Fail" => Ok(CallToolResult::success(vec![Content::text(
            "Video generation failed, please check prompt and retry.".to_string(),
        )])),
        status => Ok(CallToolResult::success(vec![Content::text(format!(
            "Video task status: {}\ntask_id: {}",
            status, params.task_id
        ))])),
    }
}

pub async fn handle_generate_video_agent(
    client: &MiniMaxClient,
    params: GenerateVideoAgentParams,
) -> Result<CallToolResult, ErrorData> {
    let req = VideoTemplateGenerationRequest {
        template_id: params.template_id,
        text_inputs: params.text_inputs.map(|vals| {
            vals.into_iter().map(|v| TextInput {
                value: v["value"].as_str().unwrap_or_default().to_string(),
            }).collect()
        }),
        media_inputs: params.media_inputs.map(|vals| {
            vals.into_iter().map(|v| MediaInput {
                value: v["value"].as_str().unwrap_or_default().to_string(),
            }).collect()
        }),
        callback_url: params.callback_url,
    };

    let resp = client.create_video_template(&req).await.map_err(to_mcp_err)?;

    let task_id = resp.task_id.as_deref().unwrap_or("N/A");
    let mut msg = format!(
        "Video Agent task submitted!\ntask_id: {}\nUse query_video_agent to poll progress.",
        task_id
    );
    if params.output_directory.is_some() || params.output_file.is_some() {
        msg.push_str(&format!(
            "\n(output_directory: {:?}, output_file: {:?} — echoed for reference; actual save happens in the query step.)",
            params.output_directory, params.output_file
        ));
    }

    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

pub async fn handle_query_video_agent(
    client: &MiniMaxClient,
    params: QueryVideoAgentParams,
) -> Result<CallToolResult, ErrorData> {
    let resp = client
        .query_video_template(&params.task_id)
        .await
        .map_err(to_mcp_err)?;

    match resp.status.as_str() {
        "Success" => {
            let url = resp.video_url.clone().unwrap_or_default();
            if (params.output_directory.is_some() || params.output_file.is_some())
                && !url.is_empty()
            {
                if let Some(path) = utils::resolve_output_file(
                    params.output_file.as_deref(),
                    params.output_directory.as_deref(),
                    "video_agent",
                    &params.task_id,
                    "mp4",
                )
                .map_err(to_mcp_err)?
                {
                    client
                        .download_to_path(&url, &path)
                        .await
                        .map_err(to_mcp_err)?;
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "Video Agent task succeeded!\ntask_id: {}\nSaved to: {}",
                        params.task_id,
                        path.display()
                    ))]));
                }
            }
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Video Agent task succeeded!\ntask_id: {}\nvideo_url: {}",
                params.task_id,
                if url.is_empty() { "N/A" } else { &url }
            ))]))
        }
        "Fail" => Ok(CallToolResult::success(vec![Content::text(
            "Video Agent task failed, please check parameters and retry.".to_string(),
        )])),
        status => Ok(CallToolResult::success(vec![Content::text(format!(
            "Video Agent task status: {}\ntask_id: {}",
            status, params.task_id
        ))])),
    }
}
