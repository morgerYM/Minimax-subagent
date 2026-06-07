//! SearchProvider impl for MiniMaxProvider.

use async_trait::async_trait;

use crate::mcp_params::UnderstandImageParams;
use crate::providers::*;
use crate::types::{SearchRequest, VlmRequest};
use crate::utils;

use super::MiniMaxProvider;

#[async_trait]
impl crate::tools::search::SearchProvider for MiniMaxProvider {
    async fn web_search(&self, query: &str) -> Result<SearchOutput, ProviderError> {
        let req = SearchRequest { q: query.to_string() };
        let resp = self.client.search(&req).await?;

        Ok(SearchOutput {
            results: resp.organic.into_iter().map(|r| SearchResultItem {
                title: r.title,
                url: r.link,
                snippet: r.snippet,
                date: r.date,
            }).collect(),
            related: resp.related_searches.into_iter().map(|r| r.query).collect(),
        })
    }

    async fn understand_image(&self, params: &UnderstandImageParams) -> Result<String, ProviderError> {
        let processed = utils::process_image_url(&params.image_source).await;
        let req = VlmRequest {
            prompt: params.prompt.clone(),
            image_url: processed,
        };
        let resp = self.client.vlm(&req).await?;
        Ok(resp.content.unwrap_or_else(|| "No content returned".to_string()))
    }
}
