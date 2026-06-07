pub mod tts;
pub mod video;
pub mod image;
pub mod music;
pub mod chat;
pub mod search;
pub mod usage;
pub mod files;
// subagent is declared directly in main.rs with #[path] because it depends
// on binary-only items (subagent_impl, to_mcp_err).
