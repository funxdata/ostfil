use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// 小于等于该大小时计算完整文件摘要；超过时只计算首尾片段。
pub const FULL_HASH_LIMIT: u64 = 8 * 1024 * 1024;

/// 大文件首尾各取多少字节参与摘要。
pub const SAMPLE_CHUNK_BYTES: u64 = 4 * 1024 * 1024;

/// 文件校验信息，规则与 dytes 的 `FileChecksum` 一致。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChecksum {
    pub sha256: String,
    pub size: u64,
    pub sampled: bool,
    pub sample_bytes: u64,
}

/// 计算文件摘要。
///
/// - `size <= FULL_HASH_LIMIT`：整个文件的 SHA-256；
/// - `size > FULL_HASH_LIMIT`：输入为 `size`（u64 小端）+ 首 `SAMPLE_CHUNK_BYTES`
///   + 尾 `SAMPLE_CHUNK_BYTES` 的 SHA-256，此时 `sampled = true`。
///
/// 大文件不读全量内容，避免占满内存和 IO。
pub fn digest_file(path: &Path) -> Result<FileChecksum, String> {
    let size = fs::metadata(path)
        .map_err(|e| format!("failed to read size of {}: {}", path.display(), e))?
        .len();
    let mut file =
        File::open(path).map_err(|e| format!("failed to open {}: {}", path.display(), e))?;

    let sampled = size > FULL_HASH_LIMIT;
    let sample_bytes = if sampled { SAMPLE_CHUNK_BYTES } else { 0 };
    let mut hasher = Sha256::new();

    if sampled {
        hasher.update(size.to_le_bytes());
        let chunk = SAMPLE_CHUNK_BYTES as usize;

        let mut head = vec![0u8; chunk];
        file.read_exact(&mut head)
            .map_err(|e| format!("failed to read head of {}: {}", path.display(), e))?;
        hasher.update(&head);

        file.seek(SeekFrom::Start(size - SAMPLE_CHUNK_BYTES))
            .map_err(|e| format!("failed to seek {}: {}", path.display(), e))?;
        let mut tail = vec![0u8; chunk];
        file.read_exact(&mut tail)
            .map_err(|e| format!("failed to read tail of {}: {}", path.display(), e))?;
        hasher.update(&tail);
    } else {
        let mut buffer = vec![0u8; 64 * 1024];
        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
    }

    Ok(FileChecksum {
        sha256: hasher
            .finalize()
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect(),
        size,
        sampled,
        sample_bytes,
    })
}

/// 按同一规则重算并校验文件；大小或摘要任一不一致即返回错误。
pub fn verify_file(path: &Path, expected: &FileChecksum) -> Result<(), String> {
    let actual = digest_file(path)?;
    if &actual != expected {
        return Err(format!(
            "checksum mismatch for {}: expected sha256={} size={} sampled={}, got sha256={} size={} sampled={}",
            path.display(),
            expected.sha256,
            expected.size,
            expected.sampled,
            actual.sha256,
            actual.size,
            actual.sampled
        ));
    }
    Ok(())
}
