//! UsageProvider impl for MiniMaxProvider.

use async_trait::async_trait;

use crate::providers::*;

use super::MiniMaxProvider;

#[async_trait]
impl crate::tools::usage::UsageProvider for MiniMaxProvider {
    async fn query_usage(&self) -> Result<UsageResult, ProviderError> {
        let resp = self.client.get_token_plan_remains().await?;
        Ok(UsageResult {
            fields: resp.extra,
        })
    }
}
