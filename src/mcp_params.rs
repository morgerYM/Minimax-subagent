use rmcp::schemars;

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct TextToAudioParams {
    #[schemars(description = "要转为语音的文本内容")]
    pub text: String,
    #[schemars(description = "音色 ID，默认 female-shaonv")]
    pub voice_id: Option<String>,
    #[schemars(description = "模型名称，默认 speech-2.8-hd")]
    pub model: Option<String>,
    #[schemars(description = "语速 0.5-2.0，默认 1.0")]
    pub speed: Option<f64>,
    #[schemars(description = "音量 0-10，默认 1.0")]
    pub vol: Option<f64>,
    #[schemars(description = "音调 -12 到 12，默认 0")]
    pub pitch: Option<i32>,
    #[schemars(description = "情感: happy/sad/angry/fearful/disgusted/surprised/neutral")]
    pub emotion: Option<String>,
    #[schemars(description = "采样率: 8000/16000/22050/24000/32000/44100，默认 32000")]
    pub sample_rate: Option<i32>,
    #[schemars(description = "比特率: 32000/64000/128000/256000，默认 128000")]
    pub bitrate: Option<i32>,
    #[schemars(description = "音频格式: mp3/pcm/flac，默认 mp3")]
    pub format: Option<String>,
    #[schemars(description = "输出目录（可选）。提供时保存文件到此目录")]
    pub output_directory: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct ListVoicesParams {
    #[schemars(description = "音色类型过滤: all/system/voice_cloning，默认 all")]
    pub voice_type: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct VoiceCloneParams {
    #[schemars(description = "新音色的 ID")]
    pub voice_id: String,
    #[schemars(description = "参考音频文件路径或 URL")]
    pub file: String,
    #[schemars(description = "试听文本（可选）")]
    pub text: Option<String>,
    #[schemars(description = "文件是否为 URL")]
    pub is_url: Option<bool>,
    #[schemars(description = "输出目录（可选）。提供时保存试听音频到此目录")]
    pub output_directory: Option<String>,
    #[schemars(description = "语言增强: auto/Chinese/English 等，默认 auto")]
    pub language_boost: Option<String>,
    #[schemars(description = "是否需要降噪，默认 false")]
    pub need_noise_reduction: Option<bool>,
    #[schemars(description = "是否需要音量归一化，默认 false")]
    pub need_volume_normalization: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct VoiceDesignParams {
    #[schemars(description = "描述想要创建的音色特征")]
    pub prompt: String,
    #[schemars(description = "用于生成试听音频的文本")]
    pub preview_text: String,
    #[schemars(description = "自定义 voice_id（可选）")]
    pub voice_id: Option<String>,
    #[schemars(description = "输出目录（可选）。提供时保存试听音频到此目录")]
    pub output_directory: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct GenerateVideoParams {
    #[schemars(description = "视频描述 prompt")]
    pub prompt: String,
    #[schemars(description = "模型名称，默认 MiniMax-Hailuo-2.3")]
    pub model: Option<String>,
    #[schemars(description = "首帧图片 URL（可选，用于图生视频）")]
    pub first_frame_image: Option<String>,
    #[schemars(description = "视频时长（秒），仅 Hailuo-02 支持 6 或 10")]
    pub duration: Option<i32>,
    #[schemars(description = "分辨率，仅 Hailuo-02 支持 768P/1080P")]
    pub resolution: Option<String>,
    #[schemars(description = "异步模式：true 立即返回 task_id，false 等待完成")]
    pub async_mode: Option<bool>,
    #[schemars(description = "输出目录（可选）。仅在 async_mode=false 时生效，保存视频到此目录")]
    pub output_directory: Option<String>,
    #[schemars(description = "尾帧图片 URL（可选，用于首尾帧视频，模型需为 MiniMax-Hailuo-02）")]
    pub last_frame_image: Option<String>,
    #[schemars(description = "是否启用 prompt 优化，默认 true")]
    pub prompt_optimizer: Option<bool>,
    #[schemars(description = "是否启用快速预处理，默认 false，仅适用于 Hailuo-2.3/Hailuo-02")]
    pub fast_pretreatment: Option<bool>,
    #[schemars(description = "是否添加水印，默认 false")]
    pub aigc_watermark: Option<bool>,
    #[schemars(description = "回调 URL（可选），用于接收异步状态通知")]
    pub callback_url: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct QueryVideoParams {
    #[schemars(description = "视频生成任务的 task_id")]
    pub task_id: String,
    #[schemars(description = "输出目录（可选）。提供时下载并保存视频到此目录")]
    pub output_directory: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct GenerateImageParams {
    #[schemars(description = "图像描述 prompt")]
    pub prompt: String,
    #[schemars(description = "模型名称，默认 image-01")]
    pub model: Option<String>,
    #[schemars(description = "宽高比: 1:1/16:9/4:3/3:2/2:3/3:4/9:16/21:9，默认 1:1")]
    pub aspect_ratio: Option<String>,
    #[schemars(description = "生成数量 1-9，默认 1")]
    pub n: Option<i32>,
    #[schemars(description = "是否启用 prompt 优化，默认 false")]
    pub prompt_optimizer: Option<bool>,
    #[schemars(description = "输出目录（可选）。提供时保存图片到此目录")]
    pub output_directory: Option<String>,
    #[schemars(description = "宽度 512-2048，8 的倍数，仅 image-01")]
    pub width: Option<i32>,
    #[schemars(description = "高度 512-2048，8 的倍数，仅 image-01")]
    pub height: Option<i32>,
    #[schemars(description = "响应格式: url/base64，默认 url")]
    pub response_format: Option<String>,
    #[schemars(description = "随机种子，用于复现结果")]
    pub seed: Option<i64>,
    #[schemars(description = "是否添加水印，默认 false")]
    pub aigc_watermark: Option<bool>,
    #[schemars(description = "风格类型（仅 image-01-live）: cartoon/vitality/chinese_traditional/watercolor")]
    pub style_type: Option<String>,
    #[schemars(description = "风格权重 0-1，默认 0.8")]
    pub style_weight: Option<f64>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct GenerateMusicParams {
    #[schemars(description = "音乐风格描述，10-300 字符")]
    pub prompt: String,
    #[schemars(description = "歌词，10-600 字符，支持 [Intro][Verse][Chorus][Bridge][Outro] 标签")]
    pub lyrics: String,
    #[schemars(description = "模型名称，默认 music-2.6")]
    pub model: Option<String>,
    #[schemars(description = "音频格式: mp3/wav/pcm，默认 mp3")]
    pub format: Option<String>,
    #[schemars(description = "输出目录（可选）。提供时保存音乐到此目录")]
    pub output_directory: Option<String>,
    #[schemars(description = "是否添加水印，默认 false")]
    pub aigc_watermark: Option<bool>,
    #[schemars(description = "是否启用歌词优化（仅 music-2.6），默认 false")]
    pub lyrics_optimizer: Option<bool>,
    #[schemars(description = "是否为纯音乐（仅 music-2.6），默认 false")]
    pub is_instrumental: Option<bool>,
    #[schemars(description = "是否启用流式输出，默认 false")]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct ChatParams {
    #[schemars(description = "用户消息")]
    pub prompt: String,
    #[schemars(description = "模型名称，默认 MiniMax-M3（支持 1M 上下文）。支持 coding-plan-vlm, coding-plan-search, MiniMax-M2.5 等")]
    pub model: Option<String>,
    #[schemars(description = "系统提示词")]
    pub system: Option<String>,
    #[schemars(description = "最大生成 token 数，默认 4096")]
    pub max_tokens: Option<i32>,
    #[schemars(description = "温度 0-1")]
    pub temperature: Option<f64>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct GenerateLyricsParams {
    #[schemars(description = "歌词风格描述")]
    pub prompt: String,
    #[schemars(description = "模式: write_full_song（写完整歌曲）/ edit（编辑续写），默认 write_full_song")]
    pub mode: Option<String>,
    #[schemars(description = "要编辑的现有歌词（mode=edit 时使用）")]
    pub lyrics: Option<String>,
    #[schemars(description = "歌曲标题（可选）")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct GenerateMusicCoverParams {
    #[schemars(description = "参考音频 URL")]
    pub audio_url: String,
    #[schemars(description = "翻唱风格描述")]
    pub prompt: Option<String>,
    #[schemars(description = "自定义歌词（可选，不传则自动从参考音频提取）")]
    pub lyrics: Option<String>,
    #[schemars(description = "输出目录（可选）。提供时保存音乐到此目录")]
    pub output_directory: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct WebSearchParams {
    #[schemars(description = "搜索查询词，建议 3-5 个关键词")]
    pub query: String,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct UnderstandImageParams {
    #[schemars(description = "对图片的提问或分析要求")]
    pub prompt: String,
    #[schemars(description = "图片来源，支持 HTTP/HTTPS URL 或本地文件路径（绝对或相对路径）")]
    pub image_source: String,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct GenerateAudioAsyncParams {
    #[schemars(description = "要转为语音的文本内容（最长 5 万字符）")]
    pub text: String,
    #[schemars(description = "音色 ID，默认 female-shaonv")]
    pub voice_id: Option<String>,
    #[schemars(description = "模型名称，默认 speech-2.8-hd")]
    pub model: Option<String>,
    #[schemars(description = "语速 0.5-2.0，默认 1.0")]
    pub speed: Option<f64>,
    #[schemars(description = "音量 0-10，默认 1.0")]
    pub vol: Option<f64>,
    #[schemars(description = "音调 -12 到 12，默认 0")]
    pub pitch: Option<i32>,
    #[schemars(description = "情感: happy/sad/angry/fearful/disgusted/surprised/calm/fluent/whisper")]
    pub emotion: Option<String>,
    #[schemars(description = "采样率: 8000/16000/22050/24000/32000/44100，默认 32000")]
    pub sample_rate: Option<i32>,
    #[schemars(description = "比特率: 32000/64000/128000/256000，默认 128000")]
    pub bitrate: Option<i32>,
    #[schemars(description = "音频格式: mp3/pcm/flac/wav/pcmu_raw/pcmu_wav/opus，默认 mp3")]
    pub format: Option<String>,
    #[schemars(description = "声道数: 1 或 2，默认 2")]
    pub channel: Option<i32>,
    #[schemars(description = "语言增强: auto/Chinese/English 等，默认 auto")]
    pub language_boost: Option<String>,
    #[schemars(description = "文本文件 ID（文件上传后获取），与 text 二选一，支持最长 100 万字符")]
    pub text_file_id: Option<i64>,
    #[schemars(description = "是否添加水印，默认 false")]
    pub aigc_watermark: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct QueryAudioTaskParams {
    #[schemars(description = "异步 TTS 任务的 task_id")]
    pub task_id: String,
    #[schemars(description = "输出目录（可选）。提供时下载并保存 mp3 到此目录")]
    pub output_directory: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct TextToAudioStreamParams {
    #[schemars(description = "要转为语音的文本内容（单次可传最长 10000 字符）")]
    pub text: String,
    #[schemars(description = "音色 ID，默认 female-shaonv")]
    pub voice_id: Option<String>,
    #[schemars(description = "模型名称，默认 speech-2.8-hd")]
    pub model: Option<String>,
    #[schemars(description = "语速 0.5-2.0，默认 1.0")]
    pub speed: Option<f64>,
    #[schemars(description = "音量 0-10，默认 1.0")]
    pub vol: Option<f64>,
    #[schemars(description = "音调 -12 到 12，默认 0")]
    pub pitch: Option<i32>,
    #[schemars(description = "情感: happy/sad/angry/fearful/disgusted/surprised/calm/fluent/whisper")]
    pub emotion: Option<String>,
    #[schemars(description = "采样率: 8000/16000/22050/24000/32000/44100，默认 32000")]
    pub sample_rate: Option<i32>,
    #[schemars(description = "比特率: 32000/64000/128000/256000，默认 128000")]
    pub bitrate: Option<i32>,
    #[schemars(description = "音频格式: mp3/pcm/flac/wav/pcmu_raw/pcmu_wav/opus，默认 mp3")]
    pub format: Option<String>,
    #[schemars(description = "声道数: 1 或 2，默认 1")]
    pub channel: Option<i32>,
    #[schemars(description = "语言增强: Chinese/English/auto 等，默认 auto")]
    pub language_boost: Option<String>,
    #[schemars(description = "是否启用连续推理模式（韵律更自然，延迟更高），默认 false，仅 speech-2.8-hd/turbo")]
    pub continuous_sound: Option<bool>,
    #[schemars(description = "输出目录（可选）。提供时保存音频到此目录")]
    pub output_directory: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct DeleteVoiceParams {
    #[schemars(description = "音色类型: system/voice_cloning/voice_generation")]
    pub voice_type: String,
    #[schemars(description = "要删除的音色 ID")]
    pub voice_id: String,
}

// ============================================================
// File Management
// ============================================================

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct ListFilesParams {
    #[schemars(description = "文件分类: voice_clone/prompt_audio/t2a_async_input")]
    pub purpose: String,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct RetrieveFileParams {
    #[schemars(description = "文件的唯一标识符")]
    pub file_id: i64,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct DeleteFileParams {
    #[schemars(description = "文件的唯一标识符")]
    pub file_id: i64,
    #[schemars(description = "文件使用目的: voice_clone/prompt_audio/t2a_async/t2a_async_input/video_generation")]
    pub purpose: String,
}

// ============================================================
// Video Agent Task
// ============================================================

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct GenerateVideoAgentParams {
    #[schemars(description = "视频模板的 ID")]
    pub template_id: String,
    #[schemars(description = "文本输入数组（JSON 格式），用于填充模板中的文本部分，如 [{\"value\": \"狮子\"}]")]
    pub text_inputs: Option<Vec<serde_json::Value>>,
    #[schemars(description = "媒体输入数组（JSON 格式），用于填充模板中的媒体部分，如 [{\"value\": \"https://...\"}]")]
    pub media_inputs: Option<Vec<serde_json::Value>>,
    #[schemars(description = "回调 URL（可选），用于接收任务状态更新通知")]
    pub callback_url: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct QueryVideoAgentParams {
    #[schemars(description = "视频Agent任务的 task_id")]
    pub task_id: String,
}
