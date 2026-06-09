pub mod utils;
use crate::utils::cjson::{write_error, write_success};
use crate::utils::info::{get_info, save_info};
use crate::utils::pack::{do_pack, un_pack};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::ffi::{c_char, c_int, CStr};
use std::fs;
use std::path::Path;
use uuid::Uuid;

/// 存储目录
const STORAGE_DIR: &str = "./storage";

#[derive(Debug, Serialize, Deserialize)]
pub struct Ostfil {
    pub uuid: String,
    pub size: u64,
    pub extra: String,
    pub end_time: u64,
    pub md5: String,
}

/// 保存文件
#[unsafe(no_mangle)]
pub extern "C" fn save_fil(json_in: *const c_char, json_out: *mut c_char) -> c_int {
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
        file_path: String,
    }

    let req: InputData = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return write_error(json_out, 400102, &format!("invalid json: {}", e)),
    };

    let file_path = Path::new(&req.file_path);
    if !file_path.exists() {
        return write_error(json_out, 400103, "file does not exist");
    }

    // 生成 uuid
    let uuid = Uuid::new_v4().to_string();

    // 创建 uuid 目录
    let uuid_dir = Path::new(STORAGE_DIR).join(&uuid);
    if let Err(e) = fs::create_dir_all(&uuid_dir) {
        return write_error(json_out, 500100, &format!("failed to create dir: {}", e));
    }

    // 文件扩展名
    let extra = file_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    // 文件大小
    let size = match fs::metadata(file_path) {
        Ok(meta) => meta.len(),
        Err(_) => 0,
    };

    // 拷贝原文件
    let original_path = uuid_dir.join("original");
    if let Err(e) = fs::copy(file_path, &original_path) {
        return write_error(json_out, 500101, &format!("failed to copy file: {}", e));
    }

    // 压缩 original -> original.z
    let compressed_path = uuid_dir.join("original.z");
    if let Err(e) = do_pack(
        original_path.to_str().unwrap(),
        compressed_path.to_str().unwrap(),
    ) {
        return write_error(json_out, 500102, &format!("compression failed: {}", e));
    }

    // 创建 Ostfil 结构体
    let ostfil = Ostfil {
        uuid: uuid.clone(),
        size,
        extra,
        end_time: chrono::Utc::now().timestamp() as u64,
        md5: "".to_string(),
    };

    // 写入 info.json（使用 utils::info::save_info）
    if let Err(e) = save_info(
        uuid_dir.to_str().unwrap(),
        &serde_json::to_string_pretty(&ostfil).unwrap(),
    ) {
        return write_error(
            json_out,
            500103,
            &format!("failed to write info.json: {}", e),
        );
    }

    // 返回结果
    write_success(json_out, "success", json!(ostfil))
}

/// 获取文件信息
#[unsafe(no_mangle)]
pub extern "C" fn get_fil(json_in: *const c_char, json_out: *mut c_char) -> c_int {
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
