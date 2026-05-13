use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub file_id: i64,
    pub session_id: i64,
    pub sender_id: i64,
    pub file_name: String,
    pub file_size: i64,
    pub file_type: String,
    pub file_hash: String,
    pub storage_path: String,
    pub created_at: String,
    pub expires_at: String,
}

/// In-memory state tracking a pending chunked upload
#[derive(Clone)]
pub struct PendingUpload {
    pub session_id: i64,
    pub sender_id: i64,
    pub file_name: String,
    pub file_size: i64,
    pub file_type: String,
    pub total_chunks: u32,
    pub received_chunks: Vec<bool>,
    pub received_bytes: i64,
    pub created_at: std::time::Instant,
}
