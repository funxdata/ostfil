//! 统一错误码与文案。
//!
//! FFI 响应里的 `code` / `msg` 全部集中在此，调用方只引用常量，
//! 避免错误码和文案散落在各业务模块中。
//!
//! 分段约定：`4001xx` 输入与请求，`5001xx` 写入阶段，`5002xx` 读取与删除阶段。

/// 一个错误码及其固定文案。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrCode {
    pub code: i32,
    pub msg: &'static str,
}

impl ErrCode {
    /// 在固定文案后追加详情，格式为 `msg: detail`。
    pub fn with_detail(&self, detail: impl std::fmt::Display) -> String {
        format!("{}: {}", self.msg, detail)
    }
}

// ---- 4001xx 输入与请求 ----

pub const JSON_IN_NULL: ErrCode = ErrCode {
    code: 400100,
    msg: "json_in is null",
};

pub const INVALID_UTF8: ErrCode = ErrCode {
    code: 400101,
    msg: "invalid utf-8",
};

pub const INVALID_JSON: ErrCode = ErrCode {
    code: 400102,
    msg: "invalid json",
};

pub const FILE_NOT_FOUND: ErrCode = ErrCode {
    code: 400103,
    msg: "file does not exist",
};

pub const UUID_NOT_FOUND: ErrCode = ErrCode {
    code: 400104,
    msg: "uuid directory does not exist",
};

pub const INVALID_UUID: ErrCode = ErrCode {
    code: 400105,
    msg: "invalid uuid",
};

pub const NOT_A_FILE: ErrCode = ErrCode {
    code: 400106,
    msg: "file_path is not a regular file",
};

// ---- 5001xx 写入阶段 ----

pub const CREATE_DIR_FAILED: ErrCode = ErrCode {
    code: 500100,
    msg: "failed to create dir",
};

pub const COPY_FAILED: ErrCode = ErrCode {
    code: 500101,
    msg: "failed to copy file",
};

pub const COMPRESSION_FAILED: ErrCode = ErrCode {
    code: 500102,
    msg: "compression failed",
};

pub const WRITE_INFO_FAILED: ErrCode = ErrCode {
    code: 500103,
    msg: "failed to write info.json",
};

pub const SERIALIZE_INFO_FAILED: ErrCode = ErrCode {
    code: 500103,
    msg: "failed to serialize info",
};

pub const CHECKSUM_FAILED: ErrCode = ErrCode {
    code: 500104,
    msg: "failed to compute checksum",
};

pub const READ_METADATA_FAILED: ErrCode = ErrCode {
    code: 500105,
    msg: "failed to read metadata",
};

pub const INVALID_PATH: ErrCode = ErrCode {
    code: 500106,
    msg: "storage path is not valid utf-8",
};

// ---- 5002xx 读取与删除阶段 ----

pub const READ_INFO_FAILED: ErrCode = ErrCode {
    code: 500200,
    msg: "failed to read info.json",
};

pub const INVALID_INFO_JSON: ErrCode = ErrCode {
    code: 500201,
    msg: "invalid json",
};

pub const DECOMPRESSION_FAILED: ErrCode = ErrCode {
    code: 500202,
    msg: "decompression failed",
};

pub const DELETE_FAILED: ErrCode = ErrCode {
    code: 500203,
    msg: "delete failed",
};
