//! ChatProvider impl for MiniMaxProvider.

use async_trait::async_trait;

use crate::mcp_params::ChatParams;
use crate::providers::*;
use crate::types::{ChatMessage, ChatRequest};

use super::MiniMaxProvider;

#[async_trait]
impl crate::tools::chat::ChatProvider for MiniMaxProvider {
    async fn chat(&self, params: &ChatParams) -> Result<ChatOutput, ProviderError> {
        let model = params.model.clone()
            .unwrap_or_else(|| crate::consts::DEFAULT_CHAT_MODEL.to_string());

        let req = ChatRequest {
            model,
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: params.prompt.clone(),
            }],
            system: params.system.clone(),
            max_tokens: params.max_tokens.or(Some(4096)),
            temperature: params.temperature,
            top_p: None,
            stream: false,
        };

        let resp = self.client.chat(&req).await?;

        let text: Vec<String> = resp.content.iter()
            .filter(|b| b.block_type == "text")
            .filter_map(|b| b.text.as_deref())
            .map(String::from)
            .collect();

        let result = if text.is_empty() {
            "聊天完成，但无文本输出。".to_string()
        } else {
            text.join("\n")
        };

        Ok(ChatOutput {
            text: result,
            model: resp.model,
            input_tokens: resp.usage.as_ref().and_then(|u| u.input_tokens),
            output_tokens: resp.usage.as_ref().and_then(|u| u.output_tokens),
        })
    }
}
