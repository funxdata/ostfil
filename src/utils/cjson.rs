use crate::utils::errcode::ErrCode;
use serde_json::{json, Value};
use std::ffi::{c_char, c_int};

/// 写 JSON 到 C buffer
///
/// # Safety contract
///
/// `json_out` 必须为空指针，或指向至少 65536 字节的可写内存；函数最多写入
/// 65535 字节 + NUL。调用方（FFI 导出函数）负责保证该前提。
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn write_json_out(json_out: *mut c_char, value: Value) -> c_int {
    let out_str = value.to_string();
    let bytes = out_str.as_bytes();

    unsafe {
        if json_out.is_null() {
            return -1;
        }

        if bytes.len() >= 65536 {
            return -2;
        }

        std::ptr::copy_nonoverlapping(bytes.as_ptr(), json_out as *mut u8, bytes.len());

        *json_out.add(bytes.len()) = 0;
    }

    0
}

/// 返回错误 JSON
pub fn write_error(json_out: *mut c_char, code: i32, msg: &str) -> c_int {
    write_json_out(
        json_out,
        json!({
            "code": code,
            "data": {},
            "msg": msg
        }),
    )
}

/// 返回固定错误（无动态详情）
pub fn write_err(json_out: *mut c_char, err: ErrCode) -> c_int {
    write_error(json_out, err.code, err.msg)
}

/// 返回错误，并在固定文案后追加详情
pub fn write_err_detail(
    json_out: *mut c_char,
    err: ErrCode,
    detail: impl std::fmt::Display,
) -> c_int {
    write_error(json_out, err.code, &err.with_detail(detail))
}

/// 返回成功 JSON
pub fn write_success(json_out: *mut c_char, msg: &str, data: Value) -> c_int {
    write_json_out(
        json_out,
        json!({
            "code": 200,
            "data": data,
            "msg": msg
        }),
    )
}
