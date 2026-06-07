//! MiniMax provider — wraps `MiniMaxClient` and implements all capability traits.

mod chat;
mod files;
mod image;
mod music;
mod search;
mod tts;
mod usage;
mod video;

#[derive(Clone)]
pub struct MiniMaxProvider {
    pub client: crate::MiniMaxClient,
}

impl MiniMaxProvider {
    pub fn new(client: crate::MiniMaxClient) -> Self {
        Self { client }
    }
}
