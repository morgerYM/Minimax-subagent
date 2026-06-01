// Base
pub const DEFAULT_API_HOST: &str = "https://api.minimaxi.com";
pub const ENV_API_KEY: &str = "MINIMAX_API_KEY";
pub const ENV_API_HOST: &str = "MINIMAX_API_HOST";

// TTS models
pub const DEFAULT_TTS_MODEL: &str = "speech-2.8-hd";
pub const DEFAULT_VOICE_ID: &str = "female-shaonv";
pub const DEFAULT_SPEED: f64 = 1.0;
pub const DEFAULT_VOLUME: f64 = 1.0;
pub const DEFAULT_PITCH: i32 = 0;
pub const DEFAULT_EMOTION: &str = "happy";
pub const DEFAULT_SAMPLE_RATE: i32 = 32000;
pub const DEFAULT_BITRATE: i32 = 128000;
pub const DEFAULT_FORMAT: &str = "mp3";
pub const DEFAULT_CHANNEL: i32 = 1;
pub const DEFAULT_CHANNEL_ASYNC: i32 = 2;
pub const DEFAULT_LANGUAGE_BOOST: &str = "auto";

// Video models
pub const DEFAULT_VIDEO_MODEL: &str = "MiniMax-Hailuo-2.3";

// Image models
pub const DEFAULT_IMAGE_MODEL: &str = "image-01";

// Music models
pub const DEFAULT_MUSIC_MODEL: &str = "music-2.6";

// Chat models
pub const DEFAULT_CHAT_MODEL: &str = "MiniMax-M3";

// Polling for async tasks
pub const MAX_POLL_RETRIES: i32 = 30;
pub const POLL_INTERVAL_SECS: u64 = 20;
// Longer timeout for Hailuo-02 models
pub const MAX_POLL_RETRIES_HAILUO_02: i32 = 60;

// Async TTS polling (faster than video, shorter interval)
pub const ASYNC_TTS_POLL_INTERVAL_SECS: u64 = 5;
pub const ASYNC_TTS_MAX_POLL_RETRIES: i32 = 120;
