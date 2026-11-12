# ostfil 文档

`ostfil` 是一个用 Rust 实现的本地文件管理动态库（`cdylib`），通过 C ABI 暴露 JSON 接口，供宿主程序（如 Piksel 系统包）加载调用。

## 模块结构

| 路径 | 职责 |
| --- | --- |
| `src/lib.rs` | 库入口，重导出三个 FFI 函数 |
| `src/ffi/storage.rs` | 落盘：拷贝原文件、gzip 压缩、计算 MD5/大小/类型、写 `info.json` |
| `src/ffi/info.rs` | 读取 `info.json`、按需解压、刷新 `end_time`、补齐旧记录字段 |
| `src/ffi/delete.rs` | 删除整个 `storage/<uuid>/` 目录 |
| `src/ffi/types.rs` | `Ostfil` 元数据结构体、`STORAGE_DIR`（`./storage`） |
| `src/utils/cjson.rs` | 向 C 缓冲区写 JSON，统一成功/错误信封 |
| `src/utils/info.rs` | `info.json` 读写 |
| `src/utils/md.rs` | 文件 MD5 |
| `src/utils/pack.rs` | gzip 压缩 / 解压 |

## 文档索引

- [接口说明](./api.md) — 三个 C ABI 函数、请求/响应结构、调用示例
- [存储布局](./storage.md) — `storage/<uuid>/` 目录结构、`info.json` 字段、读写流程
- [错误码](./errors.md) — 业务错误码与函数返回码
- [构建与发布](./build.md) — 构建产物、平台导出、tag 发布流程

## 快速开始

```bash
cargo build --release
```

产物：

| 平台 | 文件 |
| --- | --- |
| Windows | `target/release/ostfil.dll` |
| Linux | `target/release/libostfil.so` |
| macOS | `target/release/libostfil.dylib` |

存储根目录在代码中固定为相对路径 `./storage`（`STORAGE_DIR`），因此宿主进程的工作目录决定了实际落盘位置。

## 相关文件

- `dytes.json` — 包元信息（`name`/`uuid`/`version`）与平台导出映射
- `.github/workflows/dytes-release.yml` — 打 `v*` tag 后按平台构建并用 dytes 发布
- `skills/` — 早期基于 C 结构体接口的设计说明，与当前 JSON 接口不一致，仅供参考
