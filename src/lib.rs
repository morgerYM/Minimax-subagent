pub mod client;
pub mod consts;
pub mod error;
pub mod types;
pub mod utils;
pub mod ws_client;

pub use client::MiniMaxClient;
pub use error::MiniMaxError;

// Re-export commonly used types
pub use types::{
    AsyncAudioSetting, AudioData, AudioSetting, BaseResponse, ChatMessage, ChatRequest,
    ChatResponse, ClonePrompt, DeleteVoiceRequest, FileDeleteResponse, FileListResponse,
    FileObject, FileRetrieveResponse, FileUploadResponse, ImageGenerationRequest, ImageGenerationResponse, ImageStyle,
    LyricsGenerationRequest, LyricsGenerationResponse, MusicAudioSetting,
    MusicCoverPreprocessRequest, MusicCoverPreprocessResponse, MusicGenerationRequest,
    MusicGenerationResponse, PronunciationDict, SubjectReference, T2AAsyncRequest, T2ARequest,
    T2AResponse, TimbreWeights, TokenPlanResponse, VideoGenerationRequest, VideoQueryResponse,
    VideoTaskResponse, VoiceCloneRequest, VoiceCloneResponse, VoiceDesignRequest,
    VoiceDesignResponse, VoiceGenerationInfo, VoiceInfo, VoiceListRequest, VoiceListResponse,
    VoiceModify, VoiceSetting,
};
