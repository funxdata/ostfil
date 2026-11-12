use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Ostfil {
    pub uuid: String,
    pub size: u64,
    pub extra: String,
    pub end_time: u64,
    #[serde(default)]
    pub sha256: String,
    #[serde(default)]
    pub sampled: bool,
    #[serde(default)]
    pub sample_bytes: u64,
    #[serde(default)]
    pub content_type: String,
}

pub const STORAGE_DIR: &str = "./storage";

/// 判断字符串能否安全地作为 `storage/` 下的目录名。
///
/// 只接受 36 位 UUID 形式（十六进制 + 固定位置的连字符），用于拦截
/// `../`、绝对路径等会越出存储根目录的输入。
pub fn is_safe_uuid(uuid: &str) -> bool {
    uuid.len() == 36
        && uuid.bytes().enumerate().all(|(i, b)| match i {
            8 | 13 | 18 | 23 => b == b'-',
            _ => b.is_ascii_hexdigit(),
        })
}
