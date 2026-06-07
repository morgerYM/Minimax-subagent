//! FileProvider impl for MiniMaxProvider.

use async_trait::async_trait;

use crate::providers::*;

use super::MiniMaxProvider;

#[async_trait]
impl crate::tools::files::FileProvider for MiniMaxProvider {
    async fn list_files(&self, purpose: &str) -> Result<FileListResult, ProviderError> {
        let resp = self.client.list_files(purpose).await?;
        Ok(FileListResult {
            files: resp.files.into_iter().map(|f| FileInfoResult {
                file_id: f.file_id,
                filename: f.filename,
                bytes: f.bytes,
                purpose: f.purpose,
                download_url: f.download_url,
            }).collect(),
        })
    }

    async fn retrieve_file(&self, file_id: i64) -> Result<FileInfoResult, ProviderError> {
        let resp = self.client.retrieve_file_info(file_id).await?;
        match resp.file {
            Some(f) => Ok(FileInfoResult {
                file_id: f.file_id,
                filename: f.filename,
                bytes: f.bytes,
                purpose: f.purpose,
                download_url: f.download_url,
            }),
            None => Err(ProviderError::NotFound(format!("file_id={file_id}"))),
        }
    }

    async fn delete_file(&self, file_id: i64, purpose: &str) -> Result<(), ProviderError> {
        self.client.delete_file(file_id, purpose).await?;
        Ok(())
    }

    async fn upload_file(&self, file_path: &std::path::Path, purpose: &str) -> Result<FileUploadResult, ProviderError> {
        let resp = self.client.upload_file(file_path, purpose).await?;
        let file_id = resp.file.ok_or_else(|| ProviderError::Api("upload failed".into()))?.file_id;
        Ok(FileUploadResult { file_id })
    }
}
