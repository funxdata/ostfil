use crate::ffi::types::{is_safe_uuid, STORAGE_DIR};
use crate::utils::cjson::{write_err, write_err_detail, write_success};
use crate::utils::errcode::{
    DELETE_FAILED, INVALID_JSON, INVALID_UTF8, INVALID_UUID, JSON_IN_NULL, UUID_NOT_FOUND,
};
use serde::Deserialize;
use serde_json::json;
use std::ffi::{c_char, c_int, CStr};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

#[unsafe(no_mangle)]
pub extern "C" fn post_delete_file(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    if json_in.is_null() {
        return write_err(json_out, JSON_IN_NULL);
    }

    let body = unsafe {
        match CStr::from_ptr(json_in).to_str() {
            Ok(s) => s,
            Err(_) => return write_err(json_out, INVALID_UTF8),
        }
    };

    #[derive(Deserialize)]
    struct InputData {
        uuid: String,
    }

    let req: InputData = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return write_err_detail(json_out, INVALID_JSON, e),
    };

    if !is_safe_uuid(&req.uuid) {
        return write_err(json_out, INVALID_UUID);
    }

    let uuid_dir = Path::new(STORAGE_DIR).join(&req.uuid);

    // 直接删除，按错误类型区分「不存在」，避免 exists + remove 之间的竞态
    match fs::remove_dir_all(&uuid_dir) {
        Ok(()) => write_success(json_out, "success", json!({})),
        Err(e) if e.kind() == ErrorKind::NotFound => write_err(json_out, UUID_NOT_FOUND),
        Err(e) => write_err_detail(json_out, DELETE_FAILED, e),
    }
}
