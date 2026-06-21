use crate::ffi::types::STORAGE_DIR;
use crate::utils::cjson::{write_error, write_success};
use serde::Deserialize;
use serde_json::json;
use std::ffi::{c_char, c_int, CStr};
use std::fs;
use std::path::Path;

#[unsafe(no_mangle)]
pub extern "C" fn post_delete_file(json_in: *const c_char, json_out: *mut c_char) -> c_int {
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

    // ✅ 删除目录
    if let Err(e) = fs::remove_dir_all(&uuid_dir) {
        return write_error(json_out, 500203, &format!("delete failed: {}", e));
    }

    write_success(json_out, "success", json!({}))
}
