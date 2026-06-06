mod mcp_params;
mod tools;

use minimax_api::MiniMaxClient;

use mcp_params::*;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::handler::server::ServerHandler;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_handler, tool_router, ErrorData, ServiceExt};
use tracing_subscriber::EnvFilter;

fn to_mcp_err(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

#[derive(Clone)]
struct MiniMaxMcp {
    client: MiniMaxClient,
}

#[tool_router]
impl MiniMaxMcp {
    #[tool(description = "将文本转为语音。参数:音色(默认 female-shaonv)、语速、音量、音调、情感(9种,fluent/whisper 仅 speech-2.6-turbo/hd)、采样率、比特率、音频格式、输出目录。返回 hex 编码音频或保存路径。")]
    async fn text_to_audio(
        &self,
        Parameters(params): Parameters<TextToAudioParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::tts::handle_text_to_audio(&self.client, params).await
    }

    #[tool(description = "列出所有音色,支持过滤类型:system(系统)/ voice_cloning(克隆)/ voice_generation(AI 设计)/ all(默认)。")]
    async fn list_voices(
        &self,
        Parameters(params): Parameters<ListVoicesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::tts::handle_list_voices(&self.client, params).await
    }

    #[tool(description = "克隆新音色。自动上传参考音频(本地路径或 URL),指定新 voice_id。可选提供试听文本(text 传入时 model 自动设为 speech-2.8-hd)。支持降噪、音量归一化、语言增强。")]
    async fn voice_clone(
        &self,
        Parameters(params): Parameters<VoiceCloneParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::tts::handle_voice_clone(&self.client, params).await
    }

    #[tool(description = "通过文字 prompt 设计全新音色。需要较大账户余额,余额不足会返回 API error 1008。提供 preview_text 生成试听音频。")]
    async fn voice_design(
        &self,
        Parameters(params): Parameters<VoiceDesignParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::tts::handle_voice_design(&self.client, params).await
    }

    #[tool(description = "删除指定音色。必填:voice_type(system / voice_cloning / voice_generation)和 voice_id。")]
    async fn delete_voice(
        &self,
        Parameters(params): Parameters<DeleteVoiceParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::tts::handle_delete_voice(&self.client, params).await
    }

    #[tool(description = "使用 MiniMax 生成视频。默认异步模式，立即返回 task_id；设置 async_mode=false 等待完成")]
    async fn generate_video(
        &self,
        Parameters(params): Parameters<GenerateVideoParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::video::handle_generate_video(&self.client, params).await
    }

    #[tool(description = "查询视频生成任务的状态")]
    async fn query_video(
        &self,
        Parameters(params): Parameters<QueryVideoParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::video::handle_query_video(&self.client, params).await
    }

    #[tool(description = "使用 MiniMax 生成图像")]
    async fn generate_image(
        &self,
        Parameters(params): Parameters<GenerateImageParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::image::handle_generate_image(&self.client, params).await
    }

    #[tool(description = "查询 MiniMax API 账户的 Token 余额和使用量信息")]
    async fn query_usage(&self) -> Result<CallToolResult, ErrorData> {
        tools::usage::handle_query_usage(&self.client).await
    }

    #[tool(description = "生成音乐。必填:prompt(风格描述 10-300字符)、lyrics(歌词 10-600字符,支持 [Intro][Verse][Chorus][Bridge][Outro] 标签;is_instrumental=true 时可传空串)。可选:model、format、output_directory、is_instrumental、stream、aigc_watermark、lyrics_optimizer。")]
    async fn generate_music(
        &self,
        Parameters(params): Parameters<GenerateMusicParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::music::handle_generate_music(&self.client, params).await
    }

    #[tool(description = "使用 MiniMax Anthropic 兼容接口进行文本聊天，支持 coding-plan-vlm、coding-plan-search 等模型")]
    async fn chat(
        &self,
        Parameters(params): Parameters<ChatParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::chat::handle_chat(&self.client, params).await
    }

    #[tool(description = "使用 MiniMax 生成歌词，支持完整歌曲创作和歌词编辑/续写")]
    async fn generate_lyrics(
        &self,
        Parameters(params): Parameters<GenerateLyricsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::music::handle_generate_lyrics(&self.client, params).await
    }

    #[tool(description = "生成翻唱音乐。传参考音频 URL,内部自动调用预处理提取音频特征,再生成翻唱。可选自定义歌词(不传则从参考音频提取)和风格 prompt。")]
    async fn generate_music_cover(
        &self,
        Parameters(params): Parameters<GenerateMusicCoverParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::music::handle_generate_music_cover(&self.client, params).await
    }

    #[tool(description = "使用 MiniMax 进行网络搜索，返回搜索结果和相关搜索建议。搜索查询词建议 3-5 个关键词")]
    async fn web_search(
        &self,
        Parameters(params): Parameters<WebSearchParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::search::handle_web_search(&self.client, params).await
    }

    #[tool(description = "使用 MiniMax VLM 模型分析图片内容，支持 HTTP/HTTPS URL 和本地文件路径")]
    async fn understand_image(
        &self,
        Parameters(params): Parameters<UnderstandImageParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::search::handle_understand_image(&self.client, params).await
    }

    #[tool(description = "WebSocket 流式 TTS,低延迟。单次最大 10000 字符。continuous_sound 模式(韵律更自然,延迟更高)仅 speech-2.8-hd/turbo 支持。")]
    async fn text_to_audio_stream(
        &self,
        Parameters(params): Parameters<TextToAudioStreamParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::tts::handle_text_to_audio_stream(&self.client, params).await
    }

    #[tool(description = "异步 TTS(≤5万字符,通过 text_file_id 可达 100 万)。返回 task_id 后必须配 query_audio_task 轮询并下载 mp3。支持语速、音量、音调、情感、采样率、比特率、声道、语言增强、水印。")]
    async fn generate_audio_async(
        &self,
        Parameters(params): Parameters<GenerateAudioAsyncParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::tts::handle_generate_audio_async(&self.client, params).await
    }

    #[tool(description = "查询异步 TTS 任务状态，完成后自动下载并提取 mp3 文件")]
    async fn query_audio_task(
        &self,
        Parameters(params): Parameters<QueryAudioTaskParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::tts::handle_query_audio_task(&self.client, params).await
    }

    #[tool(description = "列出 MiniMax 平台上的文件")]
    async fn list_files(
        &self,
        Parameters(params): Parameters<ListFilesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::files::handle_list_files(&self.client, params).await
    }

    #[tool(description = "检索 MiniMax 平台上的文件信息，获取下载链接")]
    async fn retrieve_file(
        &self,
        Parameters(params): Parameters<RetrieveFileParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::files::handle_retrieve_file(&self.client, params).await
    }

    #[tool(description = "删除 MiniMax 平台上的文件")]
    async fn delete_file(
        &self,
        Parameters(params): Parameters<DeleteFileParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::files::handle_delete_file(&self.client, params).await
    }

    #[tool(description = "创建视频Agent任务，基于模板生成视频")]
    async fn generate_video_agent(
        &self,
        Parameters(params): Parameters<GenerateVideoAgentParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::video::handle_generate_video_agent(&self.client, params).await
    }

    #[tool(description = "查询视频Agent任务状态")]
    async fn query_video_agent(
        &self,
        Parameters(params): Parameters<QueryVideoAgentParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::video::handle_query_video_agent(&self.client, params).await
    }
}

#[tool_handler(
    name = "minimax-mcp",
    version = "0.1.0",
    instructions = "MiniMax API MCP server — 提供视频生成、语音合成、图像生成、音乐生成等能力。需要设置 MINIMAX_API_KEY 环境变量。"
)]
impl ServerHandler for MiniMaxMcp {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
