use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};

use sha2::{Digest, Sha256};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
};

use crate::common::error::{AppError, AppResult};

use super::{
    model::{FileRecord, PendingUpload},
    repo::FileRepository,
};

pub const FILE_SIZE_LIMIT: i64 = 100 * 1024 * 1024; // 100MB max
pub const UPLOAD_TIMEOUT_SECS: u64 = 600; // 10 minutes
pub const CLEANUP_INTERVAL_MINUTES: u64 = 30;
const MAX_FILE_NAME_LEN: usize = 256;
const MAX_FILE_TYPE_LEN: usize = 128;
const MAX_TOTAL_CHUNKS: u32 = 2048;

#[derive(Debug)]
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

    pub async fn prepare_storage_dirs(&self) -> AppResult<usize> {
        fs::create_dir_all(self.upload_dir.join("tmp"))
            .await
            .map_err(|e| {
                AppError::internal(anyhow::anyhow!("failed to create upload tmp dir: {}", e))
            })?;
        fs::create_dir_all(self.upload_dir.join("final"))
            .await
            .map_err(|e| {
                AppError::internal(anyhow::anyhow!("failed to create upload final dir: {}", e))
            })?;

        self.cleanup_stale_tmp_dirs().await
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
        validate_upload_request(file_name, file_size, file_type, total_chunks)?;

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
            received_bytes: 0,
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
        user_id: i64,
        upload_id: &str,
        chunk_index: u32,
        data: &[u8],
    ) -> AppResult<()> {
        if data.is_empty() {
            return Err(AppError::BadRequest(
                "chunk data cannot be empty".to_string(),
            ));
        }

        let mut pending_map = self.pending_uploads.write().await;
        let pending = pending_map
            .get_mut(upload_id)
            .ok_or_else(|| AppError::BadRequest("upload_id not found or expired".to_string()))?;

        if pending.created_at.elapsed() > Duration::from_secs(UPLOAD_TIMEOUT_SECS) {
            pending_map.remove(upload_id);
            let _ = fs::remove_dir_all(self.pending_dir(upload_id)).await;
            return Err(AppError::BadRequest("upload session expired".to_string()));
        }

        if pending.sender_id != user_id {
            return Err(AppError::Forbidden(
                "you are not the owner of this upload".to_string(),
            ));
        }

        if chunk_index >= pending.total_chunks {
            return Err(AppError::BadRequest("chunk index out of range".to_string()));
        }

        if pending.received_chunks[chunk_index as usize] {
            return Err(AppError::BadRequest("chunk already uploaded".to_string()));
        }

        let chunk_size = i64::try_from(data.len())
            .map_err(|_| AppError::BadRequest("chunk size exceeds supported range".to_string()))?;
        if pending.received_bytes + chunk_size > pending.file_size {
            return Err(AppError::BadRequest(
                "received bytes exceed declared file size".to_string(),
            ));
        }

        let chunk_path = self
            .pending_dir(upload_id)
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

        pending.received_chunks[chunk_index as usize] = true;
        pending.received_bytes += chunk_size;

        Ok(())
    }

    pub async fn complete_upload(
        &self,
        user_id: i64,
        upload_id: &str,
        expected_hash: &str,
    ) -> AppResult<UploadResult> {
        validate_sha256_hash(expected_hash)?;

        let pending = {
            let mut pending_map = self.pending_uploads.write().await;
            let pending = pending_map.get(upload_id).cloned().ok_or_else(|| {
                AppError::BadRequest("upload_id not found or expired".to_string())
            })?;

            if pending.sender_id != user_id {
                return Err(AppError::Forbidden(
                    "you are not the owner of this upload".to_string(),
                ));
            }

            pending_map.remove(upload_id)
        };

        let pending = pending
            .ok_or_else(|| AppError::BadRequest("upload_id not found or expired".to_string()))?;

        let temp_dir = self.pending_dir(upload_id);

        if pending.created_at.elapsed() > Duration::from_secs(UPLOAD_TIMEOUT_SECS) {
            let _ = fs::remove_dir_all(&temp_dir).await;
            return Err(AppError::BadRequest("upload session expired".to_string()));
        }

        let all_received = pending.received_chunks.iter().all(|&r| r);
        if !all_received {
            let missing: Vec<String> = pending
                .received_chunks
                .iter()
                .enumerate()
                .filter(|(_, r)| !**r)
                .map(|(i, _)| i.to_string())
                .collect();
            let _ = fs::remove_dir_all(&temp_dir).await;
            return Err(AppError::BadRequest(format!(
                "missing chunks: {}",
                missing.join(", ")
            )));
        }

        if pending.received_bytes != pending.file_size {
            let _ = fs::remove_dir_all(&temp_dir).await;
            return Err(AppError::BadRequest(format!(
                "declared file size mismatch: expected {}, got {}",
                pending.file_size, pending.received_bytes
            )));
        }

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

        let actual_size = fs::metadata(&reassembled_path)
            .await
            .map_err(|e| {
                AppError::internal(anyhow::anyhow!("failed to stat reassembled file: {}", e))
            })?
            .len();
        let actual_size = i64::try_from(actual_size).map_err(|_| {
            AppError::BadRequest("reassembled file size exceeds supported range".to_string())
        })?;
        if actual_size != pending.file_size {
            let _ = fs::remove_dir_all(&temp_dir).await;
            return Err(AppError::BadRequest(format!(
                "actual file size mismatch: expected {}, got {}",
                pending.file_size, actual_size
            )));
        }

        let actual_hash = compute_file_hash(&reassembled_path).await?;
        if !actual_hash.eq_ignore_ascii_case(expected_hash) {
            let _ = fs::remove_dir_all(&temp_dir).await;
            return Err(AppError::BadRequest(format!(
                "hash mismatch: expected {}, got {}",
                expected_hash, actual_hash
            )));
        }

        let final_filename = format!(
            "{}_{}_{}",
            chrono::Utc::now().format("%Y%m%d%H%M%S"),
            uuid::Uuid::new_v4(),
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

        let _ = fs::remove_dir_all(&temp_dir).await;

        let storage_path = format!("final/{}", final_filename);
        let upload_result = self
            .repo
            .create_file_upload_message(
                pending.session_id,
                pending.sender_id,
                &pending.file_name,
                pending.file_size,
                &pending.file_type,
                &actual_hash,
                &storage_path,
            )
            .await;

        let (file_record, message_id, file_content) = match upload_result {
            Ok(result) => result,
            Err(error) => {
                let _ = fs::remove_file(&final_path).await;
                return Err(error);
            }
        };

        let recipient_ids = self
            .repo
            .get_session_member_ids(pending.session_id, user_id)
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
        let mut count = expired.len();

        for file in &expired {
            let full_path = self.upload_dir.join(&file.storage_path);
            if let Err(e) = fs::remove_file(&full_path).await
                && e.kind() != std::io::ErrorKind::NotFound
            {
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

        count += self.cleanup_stale_tmp_dirs().await?;

        Ok(count)
    }

    async fn cleanup_stale_tmp_dirs(&self) -> AppResult<usize> {
        let tmp_dir = self.upload_dir.join("tmp");
        fs::create_dir_all(&tmp_dir).await.map_err(|e| {
            AppError::internal(anyhow::anyhow!("failed to create upload tmp dir: {}", e))
        })?;

        let mut cleaned = 0;
        let mut entries = fs::read_dir(&tmp_dir).await.map_err(|e| {
            AppError::internal(anyhow::anyhow!("failed to read upload tmp dir: {}", e))
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            AppError::internal(anyhow::anyhow!("failed to read upload tmp entry: {}", e))
        })? {
            let metadata = match entry.metadata().await {
                Ok(metadata) => metadata,
                Err(error) => {
                    tracing::warn!(error = %error, "failed to read upload tmp entry metadata");
                    continue;
                }
            };

            if !metadata.is_dir() {
                continue;
            }

            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let is_stale = SystemTime::now()
                .duration_since(modified)
                .map(|age| age > Duration::from_secs(UPLOAD_TIMEOUT_SECS))
                .unwrap_or(true);

            if is_stale {
                let path = entry.path();
                match fs::remove_dir_all(&path).await {
                    Ok(()) => cleaned += 1,
                    Err(error) => {
                        tracing::warn!(
                            path = %path.display(),
                            error = %error,
                            "failed to remove stale upload tmp dir"
                        );
                    }
                }
            }
        }

        Ok(cleaned)
    }

    fn pending_dir(&self, upload_id: &str) -> PathBuf {
        self.upload_dir.join("tmp").join(upload_id)
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

async fn compute_file_hash(path: &Path) -> AppResult<String> {
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

fn validate_upload_request(
    file_name: &str,
    file_size: i64,
    file_type: &str,
    total_chunks: u32,
) -> AppResult<()> {
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
    if file_name.chars().count() > MAX_FILE_NAME_LEN {
        return Err(AppError::BadRequest(format!(
            "file name must be at most {MAX_FILE_NAME_LEN} characters"
        )));
    }
    if file_type.trim().is_empty() {
        return Err(AppError::BadRequest(
            "file type cannot be blank".to_string(),
        ));
    }
    if file_type.chars().count() > MAX_FILE_TYPE_LEN {
        return Err(AppError::BadRequest(format!(
            "file type must be at most {MAX_FILE_TYPE_LEN} characters"
        )));
    }
    if total_chunks == 0 {
        return Err(AppError::BadRequest(
            "total_chunks must be positive".to_string(),
        ));
    }
    if total_chunks > MAX_TOTAL_CHUNKS {
        return Err(AppError::BadRequest(format!(
            "total_chunks must be at most {MAX_TOTAL_CHUNKS}"
        )));
    }

    Ok(())
}

fn validate_sha256_hash(hash: &str) -> AppResult<()> {
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(AppError::BadRequest(
            "file_hash must be a 64 character sha256 hex string".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        sync::Mutex,
        time::{SystemTime, UNIX_EPOCH},
    };

    use async_trait::async_trait;

    use super::*;

    #[derive(Default)]
    struct FakeFileRepository {
        members: Mutex<HashSet<(i64, i64)>>,
        records: Mutex<HashMap<i64, FileRecord>>,
        recipient_ids: Mutex<Vec<i64>>,
        deleted_file_ids: Mutex<Vec<i64>>,
        expired_files: Mutex<Vec<FileRecord>>,
        next_file_id: Mutex<i64>,
        next_message_id: Mutex<i64>,
        fail_create: Mutex<bool>,
    }

    #[async_trait]
    impl FileRepository for FakeFileRepository {
        async fn create_file_upload_message(
            &self,
            session_id: i64,
            sender_id: i64,
            file_name: &str,
            file_size: i64,
            file_type: &str,
            file_hash: &str,
            storage_path: &str,
        ) -> AppResult<(FileRecord, i64, String)> {
            if *self.fail_create.lock().unwrap() {
                return Err(AppError::internal(anyhow::anyhow!("forced create failure")));
            }

            let mut next_file_id = self.next_file_id.lock().unwrap();
            *next_file_id += 1;
            let file_id = *next_file_id;
            let mut next_message_id = self.next_message_id.lock().unwrap();
            *next_message_id += 1;
            let message_id = *next_message_id;
            let record = FileRecord {
                file_id,
                session_id,
                sender_id,
                file_name: file_name.to_string(),
                file_size,
                file_type: file_type.to_string(),
                file_hash: file_hash.to_string(),
                storage_path: storage_path.to_string(),
                created_at: "2026-05-13 12:00:00+00".to_string(),
                expires_at: "2026-05-14 12:00:00+00".to_string(),
            };
            let file_content = serde_json::json!({
                "file_id": record.file_id,
                "file_name": record.file_name,
                "file_size": record.file_size,
                "file_type": record.file_type,
            })
            .to_string();

            self.records.lock().unwrap().insert(file_id, record.clone());
            Ok((record, message_id, file_content))
        }

        async fn get_file(&self, file_id: i64) -> AppResult<Option<FileRecord>> {
            Ok(self.records.lock().unwrap().get(&file_id).cloned())
        }

        async fn list_expired_files(&self) -> AppResult<Vec<FileRecord>> {
            Ok(self.expired_files.lock().unwrap().clone())
        }

        async fn delete_file_record(&self, file_id: i64) -> AppResult<()> {
            self.deleted_file_ids.lock().unwrap().push(file_id);
            Ok(())
        }

        async fn get_session_member_ids(
            &self,
            _session_id: i64,
            _exclude_user_id: i64,
        ) -> AppResult<Vec<i64>> {
            Ok(self.recipient_ids.lock().unwrap().clone())
        }

        async fn is_session_member(&self, session_id: i64, user_id: i64) -> AppResult<bool> {
            Ok(self
                .members
                .lock()
                .unwrap()
                .contains(&(session_id, user_id)))
        }
    }

    fn service() -> (FileService<FakeFileRepository>, PathBuf) {
        let repo = FakeFileRepository::default();
        repo.members.lock().unwrap().insert((10, 1));
        repo.members.lock().unwrap().insert((10, 2));
        repo.recipient_ids.lock().unwrap().push(2);
        let upload_dir = std::env::temp_dir().join(format!(
            "rustchat-upload-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        (FileService::new(repo, upload_dir.clone()), upload_dir)
    }

    fn sha256_hex(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    #[tokio::test]
    async fn init_upload_rejects_non_member_and_invalid_metadata() {
        let (service, upload_dir) = service();

        let non_member = service
            .init_upload(99, 10, "report.pdf", 4, "application/pdf", 1)
            .await
            .unwrap_err();
        assert_eq!(non_member.status_code(), axum::http::StatusCode::FORBIDDEN);

        let empty_file = service
            .init_upload(1, 10, "report.pdf", 0, "application/pdf", 1)
            .await
            .unwrap_err();
        assert_eq!(
            empty_file.status_code(),
            axum::http::StatusCode::BAD_REQUEST
        );

        let long_name = "x".repeat(MAX_FILE_NAME_LEN + 1);
        let invalid_name = service
            .init_upload(1, 10, &long_name, 1, "application/pdf", 1)
            .await
            .unwrap_err();
        assert_eq!(
            invalid_name.status_code(),
            axum::http::StatusCode::BAD_REQUEST
        );

        let _ = fs::remove_dir_all(upload_dir).await;
    }

    #[tokio::test]
    async fn chunks_are_owned_bounded_and_not_repeatable() {
        let (service, upload_dir) = service();
        service.prepare_storage_dirs().await.unwrap();
        let upload_id = service
            .init_upload(1, 10, "report.pdf", 6, "application/pdf", 2)
            .await
            .unwrap();

        let wrong_owner = service
            .save_chunk(2, &upload_id, 0, b"abc")
            .await
            .unwrap_err();
        assert_eq!(wrong_owner.status_code(), axum::http::StatusCode::FORBIDDEN);

        service.save_chunk(1, &upload_id, 0, b"abc").await.unwrap();

        let duplicate = service
            .save_chunk(1, &upload_id, 0, b"abc")
            .await
            .unwrap_err();
        assert_eq!(duplicate.status_code(), axum::http::StatusCode::BAD_REQUEST);

        let out_of_range = service
            .save_chunk(1, &upload_id, 2, b"abc")
            .await
            .unwrap_err();
        assert_eq!(
            out_of_range.status_code(),
            axum::http::StatusCode::BAD_REQUEST
        );

        let too_many_bytes = service
            .save_chunk(1, &upload_id, 1, b"abcd")
            .await
            .unwrap_err();
        assert_eq!(
            too_many_bytes.status_code(),
            axum::http::StatusCode::BAD_REQUEST
        );

        let _ = fs::remove_dir_all(upload_dir).await;
    }

    #[tokio::test]
    async fn complete_upload_rejects_missing_chunks_and_removes_tmp_dir() {
        let (service, upload_dir) = service();
        service.prepare_storage_dirs().await.unwrap();
        let upload_id = service
            .init_upload(1, 10, "report.pdf", 6, "application/pdf", 2)
            .await
            .unwrap();
        service.save_chunk(1, &upload_id, 0, b"abc").await.unwrap();

        let error = service
            .complete_upload(1, &upload_id, &sha256_hex(b"abc"))
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), axum::http::StatusCode::BAD_REQUEST);
        assert!(!upload_dir.join("tmp").join(&upload_id).exists());
        assert!(service.repo.records.lock().unwrap().is_empty());

        let _ = fs::remove_dir_all(upload_dir).await;
    }

    #[tokio::test]
    async fn complete_upload_rejects_size_and_hash_mismatch_without_writing_repo() {
        let (service, upload_dir) = service();
        service.prepare_storage_dirs().await.unwrap();
        let upload_id = service
            .init_upload(1, 10, "report.pdf", 6, "application/pdf", 1)
            .await
            .unwrap();
        service.save_chunk(1, &upload_id, 0, b"abc").await.unwrap();

        let size_error = service
            .complete_upload(1, &upload_id, &sha256_hex(b"abc"))
            .await
            .unwrap_err();
        assert_eq!(
            size_error.status_code(),
            axum::http::StatusCode::BAD_REQUEST
        );
        assert!(service.repo.records.lock().unwrap().is_empty());

        let upload_id = service
            .init_upload(1, 10, "report.pdf", 3, "application/pdf", 1)
            .await
            .unwrap();
        service.save_chunk(1, &upload_id, 0, b"abc").await.unwrap();
        let hash_error = service
            .complete_upload(
                1,
                &upload_id,
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .await
            .unwrap_err();
        assert_eq!(
            hash_error.status_code(),
            axum::http::StatusCode::BAD_REQUEST
        );
        assert!(service.repo.records.lock().unwrap().is_empty());

        let _ = fs::remove_dir_all(upload_dir).await;
    }

    #[tokio::test]
    async fn complete_upload_persists_file_message_and_cleans_tmp_dir() {
        let (service, upload_dir) = service();
        service.prepare_storage_dirs().await.unwrap();
        let upload_id = service
            .init_upload(1, 10, "report.pdf", 6, "application/pdf", 2)
            .await
            .unwrap();
        service.save_chunk(1, &upload_id, 0, b"abc").await.unwrap();
        service.save_chunk(1, &upload_id, 1, b"def").await.unwrap();

        let result = service
            .complete_upload(1, &upload_id, &sha256_hex(b"abcdef"))
            .await
            .unwrap();

        assert_eq!(result.file.file_name, "report.pdf");
        assert_eq!(result.file.file_size, 6);
        assert_eq!(result.message_id, 1);
        assert_eq!(result.recipient_user_ids, vec![2]);
        assert!(result.file_content.contains("\"file_id\":1"));
        assert!(upload_dir.join(&result.file.storage_path).exists());
        assert!(!upload_dir.join("tmp").join(&upload_id).exists());

        let _ = fs::remove_dir_all(upload_dir).await;
    }

    #[tokio::test]
    async fn complete_upload_removes_final_file_when_repo_write_fails() {
        let (service, upload_dir) = service();
        *service.repo.fail_create.lock().unwrap() = true;
        service.prepare_storage_dirs().await.unwrap();
        let upload_id = service
            .init_upload(1, 10, "report.pdf", 3, "application/pdf", 1)
            .await
            .unwrap();
        service.save_chunk(1, &upload_id, 0, b"abc").await.unwrap();

        let error = service
            .complete_upload(1, &upload_id, &sha256_hex(b"abc"))
            .await
            .unwrap_err();

        assert_eq!(
            error.status_code(),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        let mut final_entries = fs::read_dir(upload_dir.join("final")).await.unwrap();
        assert!(final_entries.next_entry().await.unwrap().is_none());

        let _ = fs::remove_dir_all(upload_dir).await;
    }
}
