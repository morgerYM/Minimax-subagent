//! File management tool handlers.

use async_trait::async_trait;

use crate::mcp_params::{DeleteFileParams, ListFilesParams, RetrieveFileParams};
use crate::providers::{FileListResult, FileInfoResult, FileUploadResult, ProviderError};

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

// ============================================================
// Trait
// ============================================================

#[async_trait]
pub trait FileProvider: Send + Sync {
    async fn list_files(&self, purpose: &str) -> Result<FileListResult, ProviderError>;
    async fn retrieve_file(&self, file_id: i64) -> Result<FileInfoResult, ProviderError>;
    async fn delete_file(&self, file_id: i64, purpose: &str) -> Result<(), ProviderError>;
    async fn upload_file(&self, file_path: &std::path::Path, purpose: &str) -> Result<FileUploadResult, ProviderError>;
}

// ============================================================
// Handlers
// ============================================================

fn to_mcp_err(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

pub async fn handle_list_files(
    provider: &dyn FileProvider,
    params: ListFilesParams,
) -> Result<CallToolResult, ErrorData> {
    let resp = provider.list_files(&params.purpose).await.map_err(to_mcp_err)?;

    if resp.files.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(format!(
            "No files under purpose '{}'.",
            params.purpose
        ))]));
    }

    let mut lines = vec![format!(
        "Files under purpose '{}' ({} total):",
        params.purpose,
        resp.files.len()
    )];
    for f in &resp.files {
        let name = f.filename.as_deref().unwrap_or("unknown");
        let fid = f.file_id.map_or("N/A".to_string(), |id| id.to_string());
        let size = f.bytes.map_or("N/A".to_string(), |b| format!("{} bytes", b));
        lines.push(format!("  file_id: {}, Name: {}, Size: {}", fid, name, size));
    }

    Ok(CallToolResult::success(vec![Content::text(lines.join("\n"))]))
}

pub async fn handle_retrieve_file(
    provider: &dyn FileProvider,
    params: RetrieveFileParams,
) -> Result<CallToolResult, ErrorData> {
    let f = provider.retrieve_file(params.file_id).await.map_err(to_mcp_err)?;

    let mut lines = vec![format!("File info (file_id: {}):", params.file_id)];
    if let Some(name) = &f.filename {
        lines.push(format!("  Name: {}", name));
    }
    if let Some(size) = f.bytes {
        lines.push(format!("  Size: {} bytes", size));
    }
    if let Some(purpose) = &f.purpose {
        lines.push(format!("  Purpose: {}", purpose));
    }
    if let Some(url) = &f.download_url {
        lines.push(format!("  Download URL: {}", url));
    }
    Ok(CallToolResult::success(vec![Content::text(lines.join("\n"))]))
}

pub async fn handle_delete_file(
    provider: &dyn FileProvider,
    params: DeleteFileParams,
) -> Result<CallToolResult, ErrorData> {
    provider
        .delete_file(params.file_id, &params.purpose)
        .await
        .map_err(to_mcp_err)?;

    Ok(CallToolResult::success(vec![Content::text(format!(
        "File deleted: file_id={}, purpose={}",
        params.file_id, params.purpose
    ))]))
}
