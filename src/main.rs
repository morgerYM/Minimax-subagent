use minimax_api::consts::*;
use minimax_api::types::*;
use minimax_api::MiniMaxClient;
use minimax_api::utils;

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
    #[schemars(description = "模型名称，默认 speech-2.8-hd")]
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
    #[schemars(description = "输出目录（可选）。提供时保存文件到此目录")]
    output_directory: Option<String>,
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
    #[schemars(description = "输出目录（可选）。提供时保存试听音频到此目录")]
    output_directory: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct VoiceDesignParams {
    #[schemars(description = "描述想要创建的音色特征")]
    prompt: String,
    #[schemars(description = "用于生成试听音频的文本")]
    preview_text: String,
    #[schemars(description = "自定义 voice_id（可选）")]
    voice_id: Option<String>,
    #[schemars(description = "输出目录（可选）。提供时保存试听音频到此目录")]
    output_directory: Option<String>,
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
    #[schemars(description = "输出目录（可选）。仅在 async_mode=false 时生效，保存视频到此目录")]
    output_directory: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct QueryVideoParams {
    #[schemars(description = "视频生成任务的 task_id")]
    task_id: String,
    #[schemars(description = "输出目录（可选）。提供时下载并保存视频到此目录")]
    output_directory: Option<String>,
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
    #[schemars(description = "输出目录（可选）。提供时保存图片到此目录")]
    output_directory: Option<String>,
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
    #[schemars(description = "输出目录（可选）。提供时保存音乐到此目录")]
    output_directory: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct ChatParams {
    #[schemars(description = "用户消息")]
    prompt: String,
    #[schemars(description = "模型名称，默认 MiniMax-M2.7。支持 coding-plan-vlm, coding-plan-search, MiniMax-M2.5 等")]
    model: Option<String>,
    #[schemars(description = "系统提示词")]
    system: Option<String>,
    #[schemars(description = "最大生成 token 数，默认 4096")]
    max_tokens: Option<i32>,
    #[schemars(description = "温度 0-1")]
    temperature: Option<f64>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct GenerateLyricsParams {
    #[schemars(description = "歌词风格描述")]
    prompt: String,
    #[schemars(description = "模式: write_full_song（写完整歌曲）/ edit（编辑续写），默认 write_full_song")]
    mode: Option<String>,
    #[schemars(description = "要编辑的现有歌词（mode=edit 时使用）")]
    lyrics: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct GenerateMusicCoverParams {
    #[schemars(description = "参考音频 URL")]
    audio_url: String,
    #[schemars(description = "翻唱风格描述")]
    prompt: Option<String>,
    #[schemars(description = "自定义歌词（可选，不传则自动从参考音频提取）")]
    lyrics: Option<String>,
    #[schemars(description = "输出目录（可选）。提供时保存音乐到此目录")]
    output_directory: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct WebSearchParams {
    #[schemars(description = "搜索查询词，建议 3-5 个关键词")]
    query: String,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct UnderstandImageParams {
    #[schemars(description = "对图片的提问或分析要求")]
    prompt: String,
    #[schemars(description = "图片来源，支持 HTTP/HTTPS URL 或本地文件路径（绝对或相对路径）")]
    image_source: String,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct GenerateAudioAsyncParams {
    #[schemars(description = "要转为语音的文本内容（最长 5 万字符）")]
    text: String,
    #[schemars(description = "音色 ID，默认 female-shaonv")]
    voice_id: Option<String>,
    #[schemars(description = "模型名称，默认 speech-2.8-hd")]
    model: Option<String>,
    #[schemars(description = "语速 0.5-2.0，默认 1.0")]
    speed: Option<f64>,
    #[schemars(description = "音量 0-10，默认 1.0")]
    vol: Option<f64>,
    #[schemars(description = "音调 -12 到 12，默认 0")]
    pitch: Option<i32>,
    #[schemars(description = "情感: happy/sad/angry/fearful/disgusted/surprised/calm/fluent/whisper")]
    emotion: Option<String>,
    #[schemars(description = "采样率: 8000/16000/22050/24000/32000/44100，默认 32000")]
    sample_rate: Option<i32>,
    #[schemars(description = "比特率: 32000/64000/128000/256000，默认 128000")]
    bitrate: Option<i32>,
    #[schemars(description = "音频格式: mp3/pcm/flac/wav/pcmu_raw/pcmu_wav/opus，默认 mp3")]
    format: Option<String>,
    #[schemars(description = "声道数: 1 或 2，默认 2")]
    channel: Option<i32>,
    #[schemars(description = "语言增强: auto/Chinese/English 等，默认 auto")]
    language_boost: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
struct QueryAudioTaskParams {
    #[schemars(description = "异步 TTS 任务的 task_id")]
    task_id: String,
    #[schemars(description = "输出目录（可选）。提供时下载并保存 mp3 到此目录")]
    output_directory: Option<String>,
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

        if let Some(dir) = &params.output_directory {
            if let Some(data) = &resp.data {
                if let Some(audio_hex) = &data.audio {
                    let path = utils::resolve_and_create_dir(dir).map_err(to_mcp_err)?;
                    let ext = &req.audio_setting.format;
                    let filename = utils::build_filename("text_to_audio", &req.text, ext);
                    let filepath = path.join(filename);
                    let bytes = utils::decode_hex_audio(audio_hex).map_err(to_mcp_err)?;
                    tokio::fs::write(&filepath, &bytes).await.map_err(to_mcp_err)?;
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "保存到: {}",
                        filepath.display()
                    ))]));
                }
            }
        }

        let json = serde_json::to_string_pretty(&resp).map_err(to_mcp_err)?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
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

        if let Some(dir) = &params.output_directory {
            if let Some(demo_url) = &resp.demo_audio {
                let path = utils::resolve_and_create_dir(dir).map_err(to_mcp_err)?;
                let text = req.text.as_deref().unwrap_or("voice");
                let filename = utils::build_filename("voice_clone", text, "wav");
                let filepath = path.join(filename);
                self.client
                    .download_to_path(demo_url, &filepath)
                    .await
                    .map_err(to_mcp_err)?;
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "音色克隆成功！\nvoice_id: {}\n保存到: {}",
                    req.voice_id,
                    filepath.display()
                ))]));
            }
        }

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

        if let Some(dir) = &params.output_directory {
            if let Some(audio_hex) = &resp.trial_audio {
                let path = utils::resolve_and_create_dir(dir).map_err(to_mcp_err)?;
                let filename =
                    utils::build_filename("voice_design", &req.preview_text, "mp3");
                let filepath = path.join(filename);
                let bytes = utils::decode_hex_audio(audio_hex).map_err(to_mcp_err)?;
                tokio::fs::write(&filepath, &bytes).await.map_err(to_mcp_err)?;
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "音色设计成功！\nvoice_id: {}\n保存到: {}",
                    resp.voice_id.unwrap_or_else(|| "未知".to_string()),
                    filepath.display()
                ))]));
            }
        }

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
            let bytes = self
                .client
                .generate_video_and_download(&req)
                .await
                .map_err(to_mcp_err)?;
            if let Some(dir) = &params.output_directory {
                let path = utils::resolve_and_create_dir(dir).map_err(to_mcp_err)?;
                let filename = utils::build_filename("video", &req.prompt, "mp4");
                let filepath = path.join(filename);
                tokio::fs::write(&filepath, &bytes).await.map_err(to_mcp_err)?;
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "视频生成完成！保存到: {}",
                    filepath.display()
                ))]));
            }
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
                    .map_err(to_mcp_err)?;
                if let Some(dir) = &params.output_directory {
                    let path = utils::resolve_and_create_dir(dir).map_err(to_mcp_err)?;
                    let filename = utils::build_filename("video", &params.task_id, "mp4");
                    let filepath = path.join(filename);
                    self.client
                        .download_to_path(&download_url, &filepath)
                        .await
                        .map_err(to_mcp_err)?;
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "视频生成成功！保存到: {}",
                        filepath.display()
                    ))]));
                }
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
                    self.client
                        .download_to_path(url, &filepath)
                        .await
                        .map_err(to_mcp_err)?;
                    saved.push(filepath.display().to_string());
                }
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "图像保存到:\n{}",
                    saved.join("\n")
                ))]));
            }
        }

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

    #[tool(description = "查询 MiniMax API 账户的 Token 余额和使用量信息")]
    async fn query_usage(&self) -> Result<CallToolResult, ErrorData> {
        let resp = self
            .client
            .get_token_plan_remains()
            .await
            .map_err(to_mcp_err)?;

        let mut lines: Vec<String> = Vec::new();
        // Sort keys for stable output
        let mut keys: Vec<&String> = resp.extra.keys().collect();
        keys.sort();
        for key in keys {
            if let Some(val) = resp.extra.get(key) {
                lines.push(format!("{}: {}", key, val));
            }
        }

        Ok(CallToolResult::success(vec![Content::text(if lines.is_empty() {
            format!(
                "查询成功。\nstatus: {}",
                resp.base_resp.status_msg
            )
        } else {
            format!(
                "账户用量信息:\n{}",
                lines.join("\n")
            )
        })]))
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
            audio_url: None,
            cover_feature_id: None,
            timbre: None,
        };

        let resp = self.client.generate_music(&req).await.map_err(to_mcp_err)?;

        if let Some(dir) = &params.output_directory {
            if let Some(data) = &resp.data {
                if let Some(audio_hex) = &data.audio {
                    let path = utils::resolve_and_create_dir(dir).map_err(to_mcp_err)?;
                    let ext = &req.audio_setting.format;
                    let filename = utils::build_filename("music", &req.prompt, ext);
                    let filepath = path.join(filename);
                    let bytes = utils::decode_hex_audio(audio_hex).map_err(to_mcp_err)?;
                    tokio::fs::write(&filepath, &bytes).await.map_err(to_mcp_err)?;
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "音乐保存到: {}",
                        filepath.display()
                    ))]));
                }
            }
        }

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

    #[tool(description = "使用 MiniMax Anthropic 兼容接口进行文本聊天，支持 coding-plan-vlm、coding-plan-search 等模型")]
    async fn chat(
        &self,
        Parameters(params): Parameters<ChatParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let model = params
            .model
            .unwrap_or_else(|| DEFAULT_CHAT_MODEL.to_string());
        let req = ChatRequest {
            model,
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: params.prompt,
            }],
            system: params.system,
            max_tokens: params.max_tokens.or(Some(4096)),
            temperature: params.temperature,
            top_p: None,
            stream: false,
        };

        let resp = self.client.chat(&req).await.map_err(to_mcp_err)?;

        let text: Vec<String> = resp
            .content
            .iter()
            .filter(|b| b.block_type == "text")
            .filter_map(|b| b.text.as_deref())
            .map(String::from)
            .collect();

        if text.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "聊天完成，但无文本输出。".to_string(),
            )]));
        }

        let mut result = text.join("\n");
        if let Some(usage) = &resp.usage {
            result.push_str(&format!(
                "\n\n[model: {}, input: {} tokens, output: {} tokens]",
                resp.model,
                usage.input_tokens.unwrap_or(0),
                usage.output_tokens.unwrap_or(0)
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "使用 MiniMax 生成歌词，支持完整歌曲创作和歌词编辑/续写")]
    async fn generate_lyrics(
        &self,
        Parameters(params): Parameters<GenerateLyricsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mode = params.mode.unwrap_or_else(|| "write_full_song".to_string());
        let req = LyricsGenerationRequest {
            mode,
            prompt: params.prompt,
            lyrics: params.lyrics,
        };

        let resp = self.client.generate_lyrics(&req).await.map_err(to_mcp_err)?;

        let mut lines = Vec::new();
        if let Some(title) = &resp.song_title {
            lines.push(format!("歌名: {}", title));
        }
        if let Some(tags) = &resp.style_tags {
            lines.push(format!("风格标签: {}", tags));
        }
        if let Some(lyrics) = &resp.lyrics {
            lines.push(format!("歌词:\n{}", lyrics));
        }

        Ok(CallToolResult::success(vec![Content::text(
            if lines.is_empty() {
                "歌词生成成功。".to_string()
            } else {
                lines.join("\n")
            },
        )]))
    }

    #[tool(description = "使用 MiniMax 生成翻唱音乐：上传参考音频，可自定义歌词和风格")]
    async fn generate_music_cover(
        &self,
        Parameters(params): Parameters<GenerateMusicCoverParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let pre_resp = self
            .client
            .preprocess_music_cover(&params.audio_url)
            .await
            .map_err(to_mcp_err)?;

        let cover_feature_id = pre_resp
            .cover_feature_id
            .ok_or_else(|| ErrorData::internal_error("预处理失败，未获取到 cover_feature_id", None))?;

        let req = MusicGenerationRequest {
            model: "music-cover".to_string(),
            prompt: params.prompt.unwrap_or_default(),
            lyrics: params.lyrics.unwrap_or_default(),
            audio_setting: MusicAudioSetting {
                sample_rate: DEFAULT_SAMPLE_RATE,
                bitrate: DEFAULT_BITRATE,
                format: DEFAULT_FORMAT.to_string(),
            },
            output_format: None,
            audio_url: None,
            cover_feature_id: Some(cover_feature_id),
            timbre: None,
        };

        let resp = self.client.generate_music(&req).await.map_err(to_mcp_err)?;

        if let Some(dir) = &params.output_directory {
            if let Some(data) = &resp.data {
                if let Some(audio_hex) = &data.audio {
                    let path = utils::resolve_and_create_dir(dir).map_err(to_mcp_err)?;
                    let filename = utils::build_filename("music_cover", &req.prompt, "mp3");
                    let filepath = path.join(filename);
                    let bytes = utils::decode_hex_audio(audio_hex).map_err(to_mcp_err)?;
                    tokio::fs::write(&filepath, &bytes).await.map_err(to_mcp_err)?;
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "翻唱生成完成！保存到: {}",
                        filepath.display()
                    ))]));
                }
            }
        }

        if let Some(data) = &resp.data {
            if let Some(audio) = &data.audio {
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "翻唱生成成功！数据长度: {} 字符",
                    audio.len()
                ))]))
            } else {
                Ok(CallToolResult::success(vec![Content::text(
                    "翻唱生成成功，但未返回音频数据。".to_string(),
                )]))
            }
        } else {
            Ok(CallToolResult::success(vec![Content::text(
                "翻唱生成成功。".to_string(),
            )]))
        }
    }

    #[tool(description = "使用 MiniMax 进行网络搜索，返回搜索结果和相关搜索建议。搜索查询词建议 3-5 个关键词")]
    async fn web_search(
        &self,
        Parameters(params): Parameters<WebSearchParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let req = SearchRequest { q: params.query };
        let resp = self.client.search(&req).await.map_err(to_mcp_err)?;

        let mut lines = Vec::new();
        lines.push(format!("搜索结果 (共 {} 条):", resp.organic.len()));
        lines.push(String::new());

        for (i, result) in resp.organic.iter().enumerate() {
            lines.push(format!("{}. {}", i + 1, result.title));
            lines.push(format!("   URL: {}", result.link));
            lines.push(format!("   {}", result.snippet));
            if let Some(date) = &result.date {
                lines.push(format!("   日期: {}", date));
            }
            lines.push(String::new());
        }

        if !resp.related_searches.is_empty() {
            lines.push("相关搜索:".to_string());
            for rs in &resp.related_searches {
                lines.push(format!("  - {}", rs.query));
            }
        }

        Ok(CallToolResult::success(vec![Content::text(lines.join("\n"))]))
    }

    #[tool(description = "使用 MiniMax VLM 模型分析图片内容，支持 HTTP/HTTPS URL 和本地文件路径")]
    async fn understand_image(
        &self,
        Parameters(params): Parameters<UnderstandImageParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let processed = utils::process_image_url(&params.image_source).await;
        let req = VlmRequest {
            prompt: params.prompt,
            image_url: processed,
        };
        let resp = self.client.vlm(&req).await.map_err(to_mcp_err)?;

        let content = resp.content.unwrap_or_else(|| "未返回内容".to_string());
        Ok(CallToolResult::success(vec![Content::text(content)]))
    }

    #[tool(description = "使用 MiniMax 异步文本转语音（支持最长 5 万字符），立即返回 task_id")]
    async fn generate_audio_async(
        &self,
        Parameters(params): Parameters<GenerateAudioAsyncParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let req = T2AAsyncRequest {
            model: params.model.unwrap_or_else(|| DEFAULT_TTS_MODEL.to_string()),
            text: params.text,
            voice_setting: VoiceSetting {
                voice_id: params
                    .voice_id
                    .unwrap_or_else(|| DEFAULT_VOICE_ID.to_string()),
                speed: params.speed,
                vol: params.vol,
                pitch: params.pitch,
                emotion: params.emotion,
            },
            audio_setting: Some(AsyncAudioSetting {
                audio_sample_rate: params.sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE),
                bitrate: params.bitrate.unwrap_or(DEFAULT_BITRATE),
                format: params.format.unwrap_or_else(|| DEFAULT_FORMAT.to_string()),
                channel: params.channel.unwrap_or(2),
            }),
            language_boost: Some(
                params
                    .language_boost
                    .unwrap_or_else(|| DEFAULT_LANGUAGE_BOOST.to_string()),
            ),
        };

        let resp = self.client.create_async_tts(&req).await.map_err(to_mcp_err)?;

        let json = serde_json::to_string_pretty(&serde_json::json!({
            "task_id": resp.task_id,
            "file_id": resp.file_id,
            "usage_characters": resp.usage_characters,
            "message": format!("异步 TTS 任务已提交。使用 query_audio_task --task_id {} 查询进度并下载结果。", resp.task_id),
        }))
        .map_err(to_mcp_err)?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "查询异步 TTS 任务状态，完成后自动下载并提取 mp3 文件")]
    async fn query_audio_task(
        &self,
        Parameters(params): Parameters<QueryAudioTaskParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let task_id: i64 = params
            .task_id
            .parse()
            .map_err(|e| ErrorData::internal_error(format!("无效的 task_id: {e}"), None))?;

        // Check status first to detect failures early
        let status = self.client.query_async_tts(task_id).await.map_err(to_mcp_err)?;

        match status.status.as_str() {
            "Failed" | "Expired" => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "任务状态: {}\ntask_id: {}",
                    status.status, task_id
                ))]));
            }
            _ => {}
        }

        // Poll file retrieve until download_url is available, then download + extract
        // Default file_id to task_id (they're the same for text-based requests)
        let file_id = status.file_id.unwrap_or(task_id);

        let download_url = self
            .client
            .poll_file_download_url(file_id, ASYNC_TTS_MAX_POLL_RETRIES, ASYNC_TTS_POLL_INTERVAL_SECS)
            .await
            .map_err(to_mcp_err)?;

        // Download tar and extract mp3
        let tar_bytes = self
            .client
            .download_bytes(&download_url)
            .await
            .map_err(to_mcp_err)?;

        let mut tar = tar::Archive::new(std::io::Cursor::new(&tar_bytes));
        let mut mp3_bytes: Option<Vec<u8>> = None;

        for entry in tar.entries().map_err(to_mcp_err)? {
            let mut entry = entry.map_err(to_mcp_err)?;
            let path = entry.path().map_err(to_mcp_err)?;
            let name = path.to_string_lossy();
            if name.ends_with(".mp3") {
                let mut out = Vec::new();
                std::io::copy(&mut entry, &mut out).map_err(to_mcp_err)?;
                mp3_bytes = Some(out);
                break;
            }
        }

        let mp3 = mp3_bytes.ok_or_else(|| {
            ErrorData::internal_error("tar 包中未找到 mp3 文件", None)
        })?;

        if let Some(dir) = &params.output_directory {
            let path = utils::resolve_and_create_dir(dir).map_err(to_mcp_err)?;
            let filename = utils::build_filename("async_tts", &task_id.to_string(), "mp3");
            let filepath = path.join(filename);
            tokio::fs::write(&filepath, &mp3).await.map_err(to_mcp_err)?;
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "异步 TTS 完成！保存到: {} ({} bytes)",
                filepath.display(),
                mp3.len()
            ))]));
        }

        Ok(CallToolResult::success(vec![Content::text(format!(
            "异步 TTS 完成！音频大小: {} bytes ({} KB)",
            mp3.len(),
            mp3.len() / 1024
        ))]))
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
