//! File management tool handlers.
//!
//! Provides file list, retrieve, and delete via MiniMax Files API.

use minimax_api::MiniMaxClient;

use minimax_api::mcp_params::*;
use crate::to_mcp_err;

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData;

pub async fn handle_list_files(
    client: &MiniMaxClient,
    params: ListFilesParams,
) -> Result<CallToolResult, ErrorData> {
    let resp = client.list_files(&params.purpose).await.map_err(to_mcp_err)?;

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
    client: &MiniMaxClient,
    params: RetrieveFileParams,
) -> Result<CallToolResult, ErrorData> {
    let resp = client
        .retrieve_file_info(params.file_id)
        .await
        .map_err(to_mcp_err)?;

    match resp.file {
        Some(f) => {
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
        None => Ok(CallToolResult::success(vec![Content::text(format!(
            "File not found: file_id={}.",
            params.file_id
        ))])),
    }
}

pub async fn handle_delete_file(
    client: &MiniMaxClient,
    params: DeleteFileParams,
) -> Result<CallToolResult, ErrorData> {
    client
        .delete_file(params.file_id, &params.purpose)
        .await
        .map_err(to_mcp_err)?;

    Ok(CallToolResult::success(vec![Content::text(format!(
        "File deleted: file_id={}, purpose={}",
        params.file_id, params.purpose
    ))]))
}
