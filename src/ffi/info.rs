use crate::ffi::types::{Ostfil, STORAGE_DIR};
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

    let uuid_dir = Path::new(STORAGE_DIR).join(&req.uuid);
    if !uuid_dir.exists() {
        return write_error(json_out, 400104, "uuid directory does not exist");
    }

    // 读取 info.json（使用 utils::info::get_info）
    let content = match get_info(uuid_dir.to_str().unwrap()) {
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

    // 更新 end_time
    ostfil.end_time = chrono::Utc::now().timestamp() as u64;

    // 检查 original 是否存在，如果不存在，则解压
    let original_path = uuid_dir.join("original");
    let compressed_path = uuid_dir.join("original.z");
    if !original_path.exists() && compressed_path.exists() {
        if let Err(e) = un_pack(
            compressed_path.to_str().unwrap(),
            original_path.to_str().unwrap(),
        ) {
            return write_error(json_out, 500202, &format!("decompression failed: {}", e));
        }
    }

    // 返回更新后的 Ostfil
    write_success(json_out, "success", json!(ostfil))
}
