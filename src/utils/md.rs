use std::fs;
use std::path::Path;

/// 计算文件 MD5。
pub fn compute_md5(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
    Ok(format!("{:x}", md5::compute(&bytes)))
}
