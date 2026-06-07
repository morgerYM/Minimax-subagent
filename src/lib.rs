pub mod client;
pub mod consts;
pub mod error;
pub mod mcp_params;
pub mod providers;
pub mod subagent;
pub mod tools;
pub mod types;
pub mod utils;
pub mod ws_client;

pub use client::MiniMaxClient;
pub use error::MiniMaxError;

// Re-export commonly used types
pub use types::{
    AgentChatRequest, AgentChatResponse, AgentContent, AgentContentBlock, AgentMessage,
    AgentResponseBlock, AsyncAudioSetting, AudioData, AudioSetting, BaseResponse, ChatMessage,
    ChatRequest, ChatResponse, ClonePrompt, DeleteVoiceRequest, FileDeleteResponse,
    FileListResponse, FileObject, FileRetrieveResponse, FileUploadResponse,
    ImageGenerationRequest, ImageGenerationResponse, ImageStyle, LyricsGenerationRequest,
    LyricsGenerationResponse, MusicAudioSetting, MusicCoverPreprocessRequest,
    MusicCoverPreprocessResponse, MusicGenerationRequest, MusicGenerationResponse,
    PronunciationDict, SubjectReference, T2AAsyncRequest, T2ARequest, T2AResponse,
    TimbreWeights, TokenPlanResponse, ToolSpec, VideoGenerationRequest, VideoQueryResponse,
    VideoTaskResponse, VoiceCloneRequest, VoiceCloneResponse, VoiceDesignRequest,
    VoiceDesignResponse, VoiceGenerationInfo, VoiceInfo, VoiceListRequest, VoiceListResponse,
    VoiceModify, VoiceSetting,
};
