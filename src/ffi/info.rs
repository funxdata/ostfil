use crate::ffi::types::{is_safe_uuid, Ostfil, STORAGE_DIR};
use crate::utils::cjson::{write_error, write_success};
use crate::utils::info::get_info;
use crate::utils::pack::un_pack;
use chrono;
use serde::Deserialize;
use serde_json::json;
use std::ffi::{c_char, c_int, CStr};
use std::path::Path;

#[unsafe(no_mangle)]
pub extern "C" fn post_get_fil(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    if json_in.is_null() {
        return write_error(json_out, 400100, "json_in is null");
    }

    let body = unsafe {
        match CStr::from_ptr(json_in).to_str() {
            Ok(s) => s,
            Err(_) => return write_error(json_out, 400101, "invalid utf-8"),
        }
    };

    #[derive(Deserialize)]
    struct InputData {
        uuid: String,
    }

    let req: InputData = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return write_error(json_out, 400102, &format!("invalid json: {}", e)),
    };

    if !is_safe_uuid(&req.uuid) {
        return write_error(json_out, 400105, "invalid uuid");
    }

    let uuid_dir = Path::new(STORAGE_DIR).join(&req.uuid);
    if !uuid_dir.exists() {
        return write_error(json_out, 400104, "uuid directory does not exist");
    }

    let Some(dir_str) = uuid_dir.to_str() else {
        return write_error(json_out, 500106, "storage path is not valid utf-8");
    };

    // 读取 info.json（使用 utils::info::get_info）
    let content = match get_info(dir_str) {
        Ok(c) => c,
        Err(e) => {
            return write_error(
                json_out,
                500200,
                &format!("failed to read info.json: {}", e),
            )
        }
    };

    let mut ostfil: Ostfil = match serde_json::from_str(&content) {
        Ok(f) => f,
        Err(e) => return write_error(json_out, 500201, &format!("invalid json: {}", e)),
    };

    // 刷新最后使用时间，并记录是否需要在读取后写回
    let mut dirty = false;
    let now = chrono::Utc::now().timestamp() as u64;
    if now > ostfil.end_time {
        ostfil.end_time = now;
        dirty = true;
    }

    // 检查 original 是否存在，如果不存在，则解压
    let original_path = uuid_dir.join("original");
    let compressed_path = {
        let current = uuid_dir.join("original.z");
        // 兼容旧布局：早期版本的压缩文件名为 original.z.z
        if current.exists() {
            current
        } else {
            uuid_dir.join("original.z.z")
        }
    };
    if !original_path.exists() && compressed_path.exists() {
        let (Some(compressed_str), Some(original_str)) =
            (compressed_path.to_str(), original_path.to_str())
        else {
            return write_error(json_out, 500106, "storage path is not valid utf-8");
        };
        if let Err(e) = un_pack(compressed_str, original_str) {
            return write_error(json_out, 500202, &format!("decompression failed: {}", e));
        }
    }

    // 兼容旧 info.json：补齐 sha256 / content_type
    if ostfil.content_type.is_empty() {
        ostfil.content_type = crate::ffi::storage::content_type_for(&ostfil.extra);
        dirty = true;
    }
    if ostfil.sha256.is_empty() && original_path.exists() {
        if let Ok(checksum) = crate::utils::checksum::digest_file(&original_path) {
            ostfil.sha256 = checksum.sha256;
            ostfil.sampled = checksum.sampled;
            ostfil.sample_bytes = checksum.sample_bytes;
            dirty = true;
        }
    }

    // 把最后使用时间与补算字段写回；写回失败不影响本次读取结果
    if dirty {
        if let Ok(info) = serde_json::to_string_pretty(&ostfil) {
            let _ = crate::utils::info::save_info(dir_str, &info);
        }
    }

    // 返回更新后的 Ostfil
    write_success(json_out, "success", json!(ostfil))
}
