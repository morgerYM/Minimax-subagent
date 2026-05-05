pub mod client;
pub mod consts;
pub mod error;
pub mod types;

pub use client::MiniMaxClient;
pub use error::MiniMaxError;

// Re-export commonly used types
pub use types::{
    AudioData, AudioSetting, BaseResponse, FileUploadResponse, ImageGenerationRequest,
    ImageGenerationResponse, MusicAudioSetting, MusicGenerationRequest, MusicGenerationResponse,
    T2ARequest, T2AResponse, VideoGenerationRequest, VideoQueryResponse, VideoTaskResponse,
    VoiceCloneRequest, VoiceCloneResponse, VoiceDesignRequest, VoiceDesignResponse, VoiceInfo,
    VoiceListRequest, VoiceListResponse, VoiceSetting,
};
