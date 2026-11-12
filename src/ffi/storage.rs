use crate::ffi::types::{Ostfil, STORAGE_DIR};
use crate::utils::checksum::digest_file;
use crate::utils::cjson::{write_err, write_err_detail, write_success};
use crate::utils::errcode::{
    ErrCode, CHECKSUM_FAILED, COMPRESSION_FAILED, COPY_FAILED, CREATE_DIR_FAILED, FILE_NOT_FOUND,
    INVALID_JSON, INVALID_PATH, INVALID_UTF8, JSON_IN_NULL, NOT_A_FILE, READ_METADATA_FAILED,
    SERIALIZE_INFO_FAILED, WRITE_INFO_FAILED,
};
use crate::utils::info::save_info;
use crate::utils::pack::do_pack;
use serde::Deserialize;
use serde_json::json;
use std::ffi::{c_char, c_int, CStr};
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[unsafe(no_mangle)]
pub extern "C" fn post_storage(json_in: *const c_char, json_out: *mut c_char) -> c_int {
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
        file_path: String,
    }

    let req: InputData = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return write_err_detail(json_out, INVALID_JSON, e),
    };

    let file_path = Path::new(&req.file_path);
    if !file_path.exists() {
        return write_err(json_out, FILE_NOT_FOUND);
    }

    // 只接受普通文件：目录、设备文件等直接拒绝，避免 fs::copy 报出误导性的错误
    let metadata = match fs::metadata(file_path) {
        Ok(m) => m,
        Err(e) => return write_err_detail(json_out, READ_METADATA_FAILED, e),
    };
    if !metadata.is_file() {
        return write_err(json_out, NOT_A_FILE);
    }
    let size = metadata.len();

    // 生成 uuid
    let uuid = Uuid::new_v4().to_string();

    // 创建 uuid 目录
    let uuid_dir = Path::new(STORAGE_DIR).join(&uuid);
    if let Err(e) = fs::create_dir_all(&uuid_dir) {
        return fail(json_out, &uuid_dir, CREATE_DIR_FAILED, e);
    }

    // 文件扩展名
    let extra = file_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    // 拷贝原文件
    let original_path = uuid_dir.join("original");
    if let Err(e) = fs::copy(file_path, &original_path) {
        return fail(json_out, &uuid_dir, COPY_FAILED, e);
    }

    // 压缩 original -> original.z
    let compressed_path = uuid_dir.join("original.z");
    let (Some(original_str), Some(compressed_str)) =
        (original_path.to_str(), compressed_path.to_str())
    else {
        return fail_plain(json_out, &uuid_dir, INVALID_PATH);
    };
    if let Err(e) = do_pack(original_str, compressed_str) {
        return fail(json_out, &uuid_dir, COMPRESSION_FAILED, e);
    }

    let checksum = match digest_file(&original_path) {
        Ok(checksum) => checksum,
        Err(e) => return fail(json_out, &uuid_dir, CHECKSUM_FAILED, e),
    };
    let content_type = content_type_for(&extra);

    // 创建 Ostfil 结构体
    let ostfil = Ostfil {
        uuid: uuid.clone(),
        size,
        extra,
        end_time: chrono::Utc::now().timestamp() as u64,
        sha256: checksum.sha256,
        sampled: checksum.sampled,
        sample_bytes: checksum.sample_bytes,
        content_type,
    };

    let info = match serde_json::to_string_pretty(&ostfil) {
        Ok(s) => s,
        Err(e) => return fail(json_out, &uuid_dir, SERIALIZE_INFO_FAILED, e),
    };
    let Some(dir_str) = uuid_dir.to_str() else {
        return fail_plain(json_out, &uuid_dir, INVALID_PATH);
    };

    // 写入 info.json（使用 utils::info::save_info）
    if let Err(e) = save_info(dir_str, &info) {
        return fail(json_out, &uuid_dir, WRITE_INFO_FAILED, e);
    }

    // 返回结果
    write_success(json_out, "success", json!(ostfil))
}

/// 写入带详情的错误响应，并清理本次创建的目录，避免留下孤儿目录。
fn fail(json_out: *mut c_char, dir: &Path, err: ErrCode, detail: impl std::fmt::Display) -> c_int {
    let _ = fs::remove_dir_all(dir);
    write_err_detail(json_out, err, detail)
}

/// 写入固定错误响应（无详情），并清理本次创建的目录。
fn fail_plain(json_out: *mut c_char, dir: &Path, err: ErrCode) -> c_int {
    let _ = fs::remove_dir_all(dir);
    write_err(json_out, err)
}

/// 按扩展名返回基础 MIME 类型；未知类型返回 application/octet-stream。
pub fn content_type_for(extra: &str) -> String {
    match extra.to_ascii_lowercase().as_str() {
        "txt" | "md" | "log" => "text/plain",
        "json" => "application/json",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" | "mjs" => "application/javascript",
        "ts" => "application/typescript",
        "csv" => "text/csv",
        "xml" => "application/xml",
        "yaml" | "yml" => "application/yaml",
        "toml" => "application/toml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "gz" => "application/gzip",
        "tar" => "application/x-tar",
        "7z" => "application/x-7z-compressed",
        _ => "application/octet-stream",
    }
    .to_string()
}
