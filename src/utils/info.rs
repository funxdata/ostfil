use std::fs;
use std::path::Path;
use uuid::Uuid;

/// 获取文件信息
pub fn get_info(path: &str) -> Result<String, String> {
    let file_path = Path::new(path).join("info.json");

    fs::read_to_string(&file_path)
        .map_err(|e| format!("failed to read {}: {}", file_path.display(), e))
}

/// 保存文件信息。
///
/// 先写临时文件再用 `rename` 替换，避免写入中途失败留下截断的 `info.json`。
pub fn save_info(path: &str, info: &str) -> Result<(), String> {
    let dir = Path::new(path);
    let file_path = dir.join("info.json");
    let tmp_path = dir.join(format!(".info.json.{}.tmp", Uuid::new_v4()));

    fs::create_dir_all(dir).map_err(|e| format!("failed to create {}: {}", dir.display(), e))?;

    fs::write(&tmp_path, info)
        .map_err(|e| format!("failed to write {}: {}", tmp_path.display(), e))?;

    fs::rename(&tmp_path, &file_path).map_err(|e| {
        let _ = fs::remove_file(&tmp_path);
        format!("failed to replace {}: {}", file_path.display(), e)
    })?;

    Ok(())
}
