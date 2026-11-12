# 代码约定

> 适用于本仓库 `src/`、`tests/` 下的 Rust 代码。改动代码前先读这一页。

## 1. 测试只放 `tests/`

- `src/` 内不允许出现 `#[cfg(test)]` 模块、`#[test]` 函数或 `mod tests`。
- 所有测试写在仓库根目录的 `tests/` 下，作为集成测试：
  - `tests/*.rs` 通过 `ostfil::post_storage` 等公开入口调用；
  - 需要验证内部逻辑时，把可测逻辑抽成 `pub` 的小函数（如 `utils::pack`、`utils::md` 已是公开模块），再从 `tests/` 调用；
  - 不要为了测试把内部实现改成 `pub(crate)` 后在 `src/` 里塞测试模块。
- 运行：`cargo test`。

理由：本仓库产出 `cdylib`，测试代码不应进入发布产物；集成测试按外部调用方式验证，能同时覆盖 FFI 契约。

## 2. `mod.rs` 只做模块声明

`src/**/mod.rs` 中只允许：

- `mod xxx;` / `pub mod xxx;`
- `pub use ...;`（需要重导出时）
- 模块级文档注释（`//!`）

不允许函数、结构体、常量、`impl` 块等任何实现代码。实现在同目录的其他文件中。

```rust
// src/utils/mod.rs
pub mod cjson;
pub mod info;
pub mod pack;
pub mod md;
```

理由：`mod.rs` 只承担模块组织，实现放在独立文件里，便于定位、review 和后续拆分。

## 3. 尽量不要改动 FFI 接口

以下内容视为对外契约（见 [FFI 接口](../docs/ffi.md)、[错误码](../docs/errors.md)），改动需谨慎：

- 导出函数名：`post_storage` / `post_get_fil` / `post_delete_file`；
- 参数与返回类型：`(const char *json_in, char *json_out) -> int`；
- 请求与响应 JSON 的字段名、字段类型；
- 错误码与 `msg` 文本。

约定：

- 新增能力优先**新增**函数/字段，而不是修改已有签名或语义；
- 确需修改时，同一次提交内同步更新 `docs/ffi.md`、`docs/errors.md`，并在提交信息中写明对宿主的兼容性影响；
- 同步更新 `ts/ffi.ts` 的符号表与 `ts/*.ts` 调用。

理由：宿主通过 C ABI 直接加载动态库，签名或字段变动会让旧宿主静默失败甚至崩溃。

## 4. 补充

- FFI 边界不得使用 `unwrap()` / `expect()`，避免 panic 越过 `extern "C"` 终止宿主进程（见 `docs/issues.md` I-12）。
- 新增错误码沿用现有分段：`4001xx` 输入/请求，`5001xx` 写入阶段，`5002xx` 读取/删除阶段。
- `storage/` 布局与 `info.json` 字段属于数据格式，改动需保持旧数据可读，或在读取侧做兼容（参考 `Ostfil.content_type` 的 `#[serde(default)]`）。
