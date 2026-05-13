use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use sha2::{Digest, Sha256};
use tokio::{fs, io::AsyncWriteExt, sync::RwLock};

use crate::common::error::{AppError, AppResult};

use super::{
    model::{FileRecord, PendingUpload},
    repo::FileRepository,
};

pub const FILE_SIZE_LIMIT: i64 = 100 * 1024 * 1024; // 100MB max
pub const UPLOAD_TIMEOUT_SECS: u64 = 600; // 10 minutes
pub const CLEANUP_INTERVAL_MINUTES: u64 = 30;

pub struct UploadResult {
    pub file: FileRecord,
    pub message_id: i64,
    pub file_content: String,
    pub recipient_user_ids: Vec<i64>,
}

#[derive(Clone)]
pub struct FileService<R: FileRepository> {
    repo: R,
    upload_dir: PathBuf,
    pending_uploads: Arc<RwLock<HashMap<String, PendingUpload>>>,
}

impl<R> FileService<R>
where
    R: FileRepository,
{
    pub fn new(repo: R, upload_dir: PathBuf) -> Self {
        Self {
            repo,
            upload_dir,
            pending_uploads: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn upload_dir(&self) -> &PathBuf {
        &self.upload_dir
    }

    pub async fn init_upload(
        &self,
        user_id: i64,
        session_id: i64,
        file_name: &str,
        file_size: i64,
        file_type: &str,
        total_chunks: u32,
    ) -> AppResult<String> {
        if file_size <= 0 {
            return Err(AppError::BadRequest(
                "file size must be positive".to_string(),
            ));
        }
        if file_size > FILE_SIZE_LIMIT {
            return Err(AppError::BadRequest(format!(
                "file size exceeds limit of {} bytes",
                FILE_SIZE_LIMIT
            )));
        }
        if file_name.trim().is_empty() {
            return Err(AppError::BadRequest(
                "file name cannot be blank".to_string(),
            ));
        }
        if total_chunks == 0 {
            return Err(AppError::BadRequest(
                "total_chunks must be positive".to_string(),
            ));
        }

        // Verify session membership
        if !self.repo.is_session_member(session_id, user_id).await? {
            return Err(AppError::Forbidden(
                "you are not a member of this session".to_string(),
            ));
        }

        let upload_id = uuid::Uuid::new_v4().to_string();

        // Create temp directory for chunks
        let temp_dir = self.upload_dir.join("tmp").join(&upload_id);
        fs::create_dir_all(&temp_dir)
            .await
            .map_err(|e| AppError::internal(anyhow::anyhow!("failed to create temp dir: {}", e)))?;

        let pending = PendingUpload {
            session_id,
            sender_id: user_id,
            file_name: file_name.to_string(),
            file_size,
            file_type: file_type.to_string(),
            total_chunks,
            received_chunks: vec![false; total_chunks as usize],
            created_at: Instant::now(),
        };

        self.pending_uploads
            .write()
            .await
            .insert(upload_id.clone(), pending);

        Ok(upload_id)
    }

    pub async fn save_chunk(
        &self,
        upload_id: &str,
        chunk_index: u32,
        data: &[u8],
    ) -> AppResult<()> {
        let pending = {
            let pending_map = self.pending_uploads.read().await;
            pending_map.get(upload_id).cloned()
        };

        let pending = pending
            .ok_or_else(|| AppError::BadRequest("upload_id not found or expired".to_string()))?;

        // Clean up stale uploads
        if pending.created_at.elapsed() > Duration::from_secs(UPLOAD_TIMEOUT_SECS) {
            self.pending_uploads.write().await.remove(upload_id);
            let _ = fs::remove_dir_all(self.upload_dir.join("tmp").join(upload_id)).await;
            return Err(AppError::BadRequest("upload session expired".to_string()));
        }

        if chunk_index >= pending.total_chunks {
            return Err(AppError::BadRequest("chunk index out of range".to_string()));
        }

        // Write chunk data to disk
        let chunk_path = self
            .upload_dir
            .join("tmp")
            .join(upload_id)
            .join(format!("chunk_{:06}", chunk_index));

        let mut file = fs::File::create(&chunk_path)
            .await
            .map_err(|e| AppError::internal(anyhow::anyhow!("failed to write chunk: {}", e)))?;
        file.write_all(data).await.map_err(|e| {
            AppError::internal(anyhow::anyhow!("failed to write chunk data: {}", e))
        })?;
        file.flush()
            .await
            .map_err(|e| AppError::internal(anyhow::anyhow!("failed to flush chunk: {}", e)))?;

        // Mark chunk as received
        let mut pending_map = self.pending_uploads.write().await;
        if let Some(p) = pending_map.get_mut(upload_id) {
            p.received_chunks[chunk_index as usize] = true;
        }

        Ok(())
    }

    pub async fn complete_upload(
        &self,
        user_id: i64,
        upload_id: &str,
        expected_hash: &str,
    ) -> AppResult<UploadResult> {
        let pending = {
            let mut pending_map = self.pending_uploads.write().await;
            pending_map.remove(upload_id)
        };

        let pending = pending
            .ok_or_else(|| AppError::BadRequest("upload_id not found or expired".to_string()))?;

        // Verify all chunks received
        let all_received = pending.received_chunks.iter().all(|&r| r);
        if !all_received {
            let missing: Vec<String> = pending
                .received_chunks
                .iter()
                .enumerate()
                .filter(|(_, r)| !**r)
                .map(|(i, _)| i.to_string())
                .collect();
            let _ = fs::remove_dir_all(self.upload_dir.join("tmp").join(upload_id)).await;
            return Err(AppError::BadRequest(format!(
                "missing chunks: {}",
                missing.join(", ")
            )));
        }

        // Reassemble chunks into a single file
        let temp_dir = self.upload_dir.join("tmp").join(upload_id);
        let reassembled_path = temp_dir.join("reassembled");

        {
            let mut output = fs::File::create(&reassembled_path).await.map_err(|e| {
                AppError::internal(anyhow::anyhow!("failed to create reassembled file: {}", e))
            })?;

            for i in 0..pending.total_chunks {
                let chunk_path = temp_dir.join(format!("chunk_{:06}", i));
                let chunk_data = fs::read(&chunk_path).await.map_err(|e| {
                    AppError::internal(anyhow::anyhow!("failed to read chunk {}: {}", i, e))
                })?;
                output.write_all(&chunk_data).await.map_err(|e| {
                    AppError::internal(anyhow::anyhow!("failed to write reassembled data: {}", e))
                })?;
            }

            output.flush().await.map_err(|e| {
                AppError::internal(anyhow::anyhow!("failed to flush reassembled file: {}", e))
            })?;
        }

        // Verify SHA256 hash
        let actual_hash = compute_file_hash(&reassembled_path).await?;
        if !actual_hash.eq_ignore_ascii_case(expected_hash) {
            let _ = fs::remove_dir_all(&temp_dir).await;
            return Err(AppError::BadRequest(format!(
                "hash mismatch: expected {}, got {}",
                expected_hash, actual_hash
            )));
        }

        // Move file to final location
        let final_filename = format!(
            "{}_{}",
            chrono::Utc::now().format("%Y%m%d%H%M%S"),
            sanitize_filename(&pending.file_name)
        );
        let final_dir = self.upload_dir.join("final");
        fs::create_dir_all(&final_dir).await.map_err(|e| {
            AppError::internal(anyhow::anyhow!("failed to create final dir: {}", e))
        })?;

        let final_path = final_dir.join(&final_filename);
        fs::rename(&reassembled_path, &final_path)
            .await
            .map_err(|e| AppError::internal(anyhow::anyhow!("failed to move file: {}", e)))?;

        // Remove temp directory
        let _ = fs::remove_dir_all(&temp_dir).await;

        // Create file record in database
        let storage_path = format!("final/{}", final_filename);
        let file_record = self
            .repo
            .create_file(
                pending.session_id,
                pending.sender_id,
                &pending.file_name,
                pending.file_size,
                &pending.file_type,
                &actual_hash,
                &storage_path,
            )
            .await?;

        // Get recipient list
        let recipient_ids = self
            .repo
            .get_session_member_ids(pending.session_id, user_id)
            .await?;

        // Create file message with JSON content containing metadata
        let file_content = serde_json::json!({
            "file_id": file_record.file_id,
            "file_name": pending.file_name,
            "file_size": pending.file_size,
            "file_type": pending.file_type,
        })
        .to_string();

        let message_id = self
            .repo
            .create_file_message(pending.session_id, user_id, &file_content)
            .await?;

        self.repo
            .update_session_last_message(pending.session_id, message_id)
            .await?;

        Ok(UploadResult {
            file: file_record,
            message_id,
            file_content,
            recipient_user_ids: recipient_ids,
        })
    }

    #[allow(dead_code)]
    pub async fn get_file_record(&self, file_id: i64) -> AppResult<Option<FileRecord>> {
        self.repo.get_file(file_id).await
    }

    pub async fn verify_file_access(&self, user_id: i64, file_id: i64) -> AppResult<FileRecord> {
        let file = self
            .repo
            .get_file(file_id)
            .await?
            .ok_or_else(|| AppError::NotFound("file not found".to_string()))?;

        if !self
            .repo
            .is_session_member(file.session_id, user_id)
            .await?
        {
            return Err(AppError::Forbidden(
                "you do not have access to this file".to_string(),
            ));
        }

        Ok(file)
    }

    /// Clean up expired files from disk and database.
    /// Returns the number of files cleaned.
    pub async fn cleanup_expired(&self) -> AppResult<usize> {
        let expired = self.repo.list_expired_files().await?;
        let count = expired.len();

        for file in &expired {
            let full_path = self.upload_dir.join(&file.storage_path);
            if let Err(e) = fs::remove_file(&full_path).await {
                tracing::warn!(
                    file_id = file.file_id,
                    path = %full_path.display(),
                    error = %e,
                    "failed to remove expired file from disk"
                );
            }

            if let Err(e) = self.repo.delete_file_record(file.file_id).await {
                tracing::warn!(
                    file_id = file.file_id,
                    error = %e,
                    "failed to delete expired file record"
                );
            }
        }

        Ok(count)
    }

    /// Start a background task that periodically cleans up expired files.
    pub fn start_cleanup_task(self)
    where
        Self: 'static + Send,
    {
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(Duration::from_secs(CLEANUP_INTERVAL_MINUTES * 60));
            interval.tick().await; // skip first immediate tick

            loop {
                interval.tick().await;
                match self.cleanup_expired().await {
                    Ok(count) => {
                        if count > 0 {
                            tracing::info!("cleaned up {} expired file(s)", count);
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "file cleanup task failed");
                    }
                }
            }
        });
    }
}

async fn compute_file_hash(path: &std::path::Path) -> AppResult<String> {
    use tokio::io::AsyncReadExt;

    let mut file = fs::File::open(path).await.map_err(|e| {
        AppError::internal(anyhow::anyhow!("failed to open file for hashing: {}", e))
    })?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer).await.map_err(|e| {
            AppError::internal(anyhow::anyhow!("failed to read file for hashing: {}", e))
        })?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn sanitize_filename(name: &str) -> String {
    let mut sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // Limit length
    if sanitized.len() > 128 {
        if let Some(ext) = std::path::Path::new(&sanitized)
            .extension()
            .and_then(|e| e.to_str())
        {
            let stem_len = 128 - ext.len() - 1;
            if stem_len > 0 {
                let stem: String = sanitized.chars().take(stem_len).collect();
                sanitized = format!("{}.{}", stem, ext);
            } else {
                sanitized = sanitized.chars().take(128).collect();
            }
        } else {
            sanitized = sanitized.chars().take(128).collect();
        }
    }

    sanitized
}
