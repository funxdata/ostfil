
use std::fs;
use std::path::Path;

/// 获取文件信息
pub fn get_info(path: &str) -> Result<String, String> {
    let file_path = Path::new(path).join("info.json");

    fs::read_to_string(&file_path)
        .map_err(|e| format!("failed to read {}: {}", file_path.display(), e))
}

/// 保存文件信息
pub fn save_info(path: &str, info: &str) -> Result<(), String> {
    let file_path = Path::new(path).join("info.json");

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {}", parent.display(), e))?;
    }

    fs::write(&file_path, info)
        .map_err(|e| format!("failed to write {}: {}", file_path.display(), e))?;

    Ok(())
}

