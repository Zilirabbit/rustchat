use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct InitUploadRequest {
    pub session_id: i64,
    pub file_name: String,
    pub file_size: i64,
    pub file_type: String,
    pub total_chunks: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct InitUploadResponse {
    pub upload_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CompleteUploadRequest {
    pub file_hash: String,
}
