use minimax_api::consts::*;
use minimax_api::types::*;
use minimax_api::MiniMaxClient;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{CallToolResult, Content};
use rmcp::{schemars, tool, tool_handler, tool_router, ErrorData, ServiceExt};
use tracing_subscriber::EnvFilter;

fn to_mcp_err(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

#[derive(Clone)]
struct MiniMaxMcp {
    client: MiniMaxClient,
}

// ============================================================
// MCP tool parameter structs
// ============================================================

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct TextToAudioParams {
    #[schemars(description = "要转为语音的文本内容")]
    text: String,
    #[schemars(description = "音色 ID，默认 female-shaonv")]
    voice_id: Option<String>,
    #[schemars(description = "模型名称，默认 speech-2.6-hd")]
    model: Option<String>,
    #[schemars(description = "语速 0.5-2.0，默认 1.0")]
    speed: Option<f64>,
    #[schemars(description = "音量 0-10，默认 1.0")]
    vol: Option<f64>,
    #[schemars(description = "音调 -12 到 12，默认 0")]
    pitch: Option<i32>,
    #[schemars(description = "情感: happy/sad/angry/fearful/disgusted/surprised/neutral")]
    emotion: Option<String>,
    #[schemars(description = "采样率: 8000/16000/22050/24000/32000/44100，默认 32000")]
    sample_rate: Option<i32>,
    #[schemars(description = "比特率: 32000/64000/128000/256000，默认 128000")]
    bitrate: Option<i32>,
    #[schemars(description = "音频格式: mp3/pcm/flac，默认 mp3")]
    format: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct ListVoicesParams {
    #[schemars(description = "音色类型过滤: all/system/voice_cloning，默认 all")]
    voice_type: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct VoiceCloneParams {
    #[schemars(description = "新音色的 ID")]
    voice_id: String,
    #[schemars(description = "参考音频文件路径或 URL")]
    file: String,
    #[schemars(description = "试听文本（可选）")]
    text: Option<String>,
    #[schemars(description = "文件是否为 URL")]
    is_url: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct VoiceDesignParams {
    #[schemars(description = "描述想要创建的音色特征")]
    prompt: String,
    #[schemars(description = "用于生成试听音频的文本")]
    preview_text: String,
    #[schemars(description = "自定义 voice_id（可选）")]
    voice_id: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct GenerateVideoParams {
    #[schemars(description = "视频描述 prompt")]
    prompt: String,
    #[schemars(description = "模型名称，默认 MiniMax-Hailuo-2.3")]
    model: Option<String>,
    #[schemars(description = "首帧图片 URL（可选，用于图生视频）")]
    first_frame_image: Option<String>,
    #[schemars(description = "视频时长（秒），仅 Hailuo-02 支持 6 或 10")]
    duration: Option<i32>,
    #[schemars(description = "分辨率，仅 Hailuo-02 支持 768P/1080P")]
    resolution: Option<String>,
    #[schemars(description = "异步模式：true 立即返回 task_id，false 等待完成")]
    async_mode: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct QueryVideoParams {
    #[schemars(description = "视频生成任务的 task_id")]
    task_id: String,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct GenerateImageParams {
    #[schemars(description = "图像描述 prompt")]
    prompt: String,
    #[schemars(description = "模型名称，默认 image-01")]
    model: Option<String>,
    #[schemars(description = "宽高比: 1:1/16:9/4:3/3:2/2:3/3:4/9:16/21:9，默认 1:1")]
    aspect_ratio: Option<String>,
    #[schemars(description = "生成数量 1-9，默认 1")]
    n: Option<i32>,
    #[schemars(description = "是否启用 prompt 优化，默认 true")]
    prompt_optimizer: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct GenerateMusicParams {
    #[schemars(description = "音乐风格描述，10-300 字符")]
    prompt: String,
    #[schemars(description = "歌词，10-600 字符，支持 [Intro][Verse][Chorus][Bridge][Outro] 标签")]
    lyrics: String,
    #[schemars(description = "模型名称，默认 music-2.6")]
    model: Option<String>,
    #[schemars(description = "音频格式: mp3/wav/pcm，默认 mp3")]
    format: Option<String>,
}

// ============================================================
// MCP tool implementations
// ============================================================

#[tool_router]
impl MiniMaxMcp {
    #[tool(description = "使用 MiniMax 将文本转为语音，返回音频数据或下载链接")]
    async fn text_to_audio(
        &self,
        Parameters(params): Parameters<TextToAudioParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let req = T2ARequest {
            model: params.model.unwrap_or_else(|| DEFAULT_TTS_MODEL.to_string()),
            text: params.text,
            stream: Some(false),
            voice_setting: VoiceSetting {
                voice_id: params
                    .voice_id
                    .unwrap_or_else(|| DEFAULT_VOICE_ID.to_string()),
                speed: params.speed,
                vol: params.vol,
                pitch: params.pitch,
                emotion: params.emotion,
            },
            audio_setting: AudioSetting {
                sample_rate: params.sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE),
                bitrate: params.bitrate.unwrap_or(DEFAULT_BITRATE),
                format: params.format.unwrap_or_else(|| DEFAULT_FORMAT.to_string()),
                channel: DEFAULT_CHANNEL,
            },
            language_boost: Some(DEFAULT_LANGUAGE_BOOST.to_string()),
            output_format: None,
        };

        let resp = self.client.text_to_audio(&req).await.map_err(to_mcp_err)?;

        let text = if let Some(data) = &resp.data {
            if let Some(audio) = &data.audio {
                format!("音频生成成功。数据长度: {} 字符", audio.len())
            } else {
                "音频生成成功，但未返回数据。".to_string()
            }
        } else {
            format!("音频生成成功。base_resp: {:?}", resp.base_resp)
        };

        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "列出 MiniMax 所有可用的音色（系统音色 + 克隆音色）")]
    async fn list_voices(
        &self,
        Parameters(params): Parameters<ListVoicesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let resp = self.client.list_voices(params.voice_type.as_deref()).await.map_err(to_mcp_err)?;

        let mut lines = Vec::new();
        lines.push("=== 系统音色 ===".to_string());
        for v in &resp.system_voice {
            lines.push(format!("  {} — {}", v.voice_id, v.voice_name));
        }
        lines.push("=== 克隆音色 ===".to_string());
        for v in &resp.voice_cloning {
            lines.push(format!("  {} — {}", v.voice_id, v.voice_name));
        }
        lines.push(format!(
            "共 {} 个系统音色 + {} 个克隆音色",
            resp.system_voice.len(),
            resp.voice_cloning.len()
        ));

        Ok(CallToolResult::success(vec![Content::text(lines.join("\n"))]))
    }

    #[tool(description = "克隆一个新的音色：上传参考音频，获取新的 voice_id")]
    async fn voice_clone(
        &self,
        Parameters(params): Parameters<VoiceCloneParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let file_id = if params.is_url.unwrap_or(false) {
            // Download from URL and upload
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let tmp = std::env::temp_dir().join(format!("minimax_voice_clone_{ts}"));
            self.client
                .download_to_path(&params.file, &tmp)
                .await
                .map_err(to_mcp_err)?;
            let upload = self
                .client
                .upload_file(&tmp, "voice_clone")
                .await
                .map_err(to_mcp_err)?;
            let _ = std::fs::remove_file(&tmp);
            upload
                .file
                .ok_or_else(|| ErrorData::internal_error("upload failed", None))?
                .file_id
        } else {
            let upload = self
                .client
                .upload_file(std::path::Path::new(&params.file), "voice_clone")
                .await
                .map_err(to_mcp_err)?;
            upload
                .file
                .ok_or_else(|| ErrorData::internal_error("upload failed", None))?
                .file_id
        };

        let req = VoiceCloneRequest {
            file_id,
            voice_id: params.voice_id,
            text: params.text,
            model: None,
        };

        let resp = self.client.voice_clone(&req).await.map_err(to_mcp_err)?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "音色克隆成功！\nvoice_id: {}\ndemo_audio: {}",
            req.voice_id,
            resp.demo_audio.unwrap_or_else(|| "无".to_string())
        ))]))
    }

    #[tool(description = "通过文字描述设计一个全新的音色")]
    async fn voice_design(
        &self,
        Parameters(params): Parameters<VoiceDesignParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let req = VoiceDesignRequest {
            prompt: params.prompt,
            preview_text: params.preview_text,
            voice_id: params.voice_id,
        };

        let resp = self.client.voice_design(&req).await.map_err(to_mcp_err)?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "音色设计成功！\nvoice_id: {}\ntrial_audio 长度: {} 字符",
            resp.voice_id.unwrap_or_else(|| "未知".to_string()),
            resp.trial_audio
                .as_ref()
                .map(|a| a.len())
                .unwrap_or(0)
        ))]))
    }

    #[tool(description = "使用 MiniMax 生成视频。默认异步模式，立即返回 task_id；设置 async_mode=false 等待完成")]
    async fn generate_video(
        &self,
        Parameters(params): Parameters<GenerateVideoParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let req = VideoGenerationRequest {
            model: params.model.unwrap_or_else(|| DEFAULT_VIDEO_MODEL.to_string()),
            prompt: params.prompt,
            first_frame_image: params.first_frame_image,
            duration: params.duration,
            resolution: params.resolution,
        };

        let async_mode = params.async_mode.unwrap_or(true);

        if async_mode {
            let resp = self.client.create_video(&req).await.map_err(to_mcp_err)?;
            Ok(CallToolResult::success(vec![Content::text(format!(
                "视频任务已提交！\ntask_id: {}\n使用 query_video 查询进度。",
                resp.task_id
            ))]))
        } else {
            // blocking mode — poll until done
            let bytes = self.client.generate_video_and_download(&req).await.map_err(to_mcp_err)?;
            Ok(CallToolResult::success(vec![Content::text(format!(
                "视频生成完成！大小: {:.1} MB",
                bytes.len() as f64 / 1_048_576.0
            ))]))
        }
    }

    #[tool(description = "查询视频生成任务的状态")]
    async fn query_video(
        &self,
        Parameters(params): Parameters<QueryVideoParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let resp = self.client.query_video(&params.task_id).await.map_err(to_mcp_err)?;

        match resp.status.as_str() {
            "Success" => {
                let file_id = resp.file_id.as_deref().unwrap_or("N/A");
                let download_url = self
                    .client
                    .get_file_download_url(file_id)
                    .await
                    .unwrap_or_else(|_| "获取失败".to_string());
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "视频生成成功！\nfile_id: {}\ndownload_url: {}",
                    file_id, download_url
                ))]))
            }
            "Fail" => Ok(CallToolResult::success(vec![Content::text(
                "视频生成失败，请检查 prompt 后重试。".to_string(),
            )])),
            status => Ok(CallToolResult::success(vec![Content::text(format!(
                "视频任务状态: {}\ntask_id: {}",
                status, params.task_id
            ))])),
        }
    }

    #[tool(description = "使用 MiniMax 生成图像")]
    async fn generate_image(
        &self,
        Parameters(params): Parameters<GenerateImageParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let req = ImageGenerationRequest {
            model: params.model.unwrap_or_else(|| DEFAULT_IMAGE_MODEL.to_string()),
            prompt: params.prompt,
            aspect_ratio: params.aspect_ratio,
            n: params.n,
            prompt_optimizer: params.prompt_optimizer,
        };

        let resp = self.client.generate_image(&req).await.map_err(to_mcp_err)?;

        if let Some(data) = &resp.data {
            let urls = data.image_urls.join("\n");
            Ok(CallToolResult::success(vec![Content::text(format!(
                "图像生成成功！共 {} 张:\n{}",
                data.image_urls.len(),
                urls
            ))]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(
                "图像生成成功，但未返回数据。".to_string(),
            )]))
        }
    }

    #[tool(description = "使用 MiniMax 生成音乐")]
    async fn generate_music(
        &self,
        Parameters(params): Parameters<GenerateMusicParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let req = MusicGenerationRequest {
            model: params.model.unwrap_or_else(|| DEFAULT_MUSIC_MODEL.to_string()),
            prompt: params.prompt,
            lyrics: params.lyrics,
            audio_setting: MusicAudioSetting {
                sample_rate: DEFAULT_SAMPLE_RATE,
                bitrate: DEFAULT_BITRATE,
                format: params.format.unwrap_or_else(|| DEFAULT_FORMAT.to_string()),
            },
            output_format: None,
        };

        let resp = self.client.generate_music(&req).await.map_err(to_mcp_err)?;

        if let Some(data) = &resp.data {
            if let Some(audio) = &data.audio {
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "音乐生成成功！数据长度: {} 字符",
                    audio.len()
                ))]))
            } else {
                Ok(CallToolResult::success(vec![Content::text(
                    "音乐生成成功，但未返回音频数据。".to_string(),
                )]))
            }
        } else {
            Ok(CallToolResult::success(vec![Content::text(
                "音乐生成成功。".to_string(),
            )]))
        }
    }
}

#[tool_handler(
    name = "minimax-mcp",
    version = "0.1.0",
    instructions = "MiniMax API MCP server — 提供视频生成、语音合成、图像生成、音乐生成等能力。需要设置 MINIMAX_API_KEY 环境变量。"
)]
impl ServerHandler for MiniMaxMcp {}

// ============================================================
// Entry point
// ============================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Write logs to stderr to keep stdout clean for MCP protocol.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let client = MiniMaxClient::from_env()?;
    let server = MiniMaxMcp { client };
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
