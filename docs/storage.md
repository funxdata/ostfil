# 存储布局

## 根目录

存储根目录在 `src/ffi/types.rs` 中固定为常量：

```rust
pub const STORAGE_DIR: &str = "./storage";
```

它是**相对路径**，实际位置取决于宿主进程的工作目录。目录本身不会在库初始化时创建，而是在 `post_storage` 中按需 `create_dir_all`。

## 单个文件的结构

```
./storage/
└── <uuid>/
    ├── original      # 源文件副本（未删除）
    ├── original.z    # gzip 压缩产物
    └── info.json     # 元数据（Ostfil 的 pretty JSON）
```

| 路径 | 产生者 | 说明 |
| --- | --- | --- |
| `<uuid>/original` | `post_storage` | `fs::copy` 源文件 |
| `<uuid>/original.z` | `post_storage` | gzip（最佳压缩级别）后的副本 |
| `<uuid>/info.json` | `post_storage` | `serde_json::to_string_pretty` 输出的 `Ostfil` |

`<uuid>` 为 v4 UUID 的字符串形式（36 字符，带连字符）。读取与删除都会先校验其格式（`400105`），因此不存在 `../` 或绝对路径逃逸。

早期版本（≤ 1.0.4）的压缩文件名是 `original.z.z`；`post_get_fil` 在 `original.z` 不存在时会回退使用 `original.z.z`，旧数据无需迁移。

## info.json

内容即 `Ostfil` 结构体，字段含义见 [FFI 接口](./ffi.md#ostfil-数据结构)：

```json
{
  "uuid": "3f0c1a52-...-9d0e",
  "size": 1024,
  "extra": "txt",
  "end_time": 1762531200,
  "sha256": "2dc4378f5479b2f6ac42eda016d41bdbc2b51638a550800592d18d9052fdcdf9",
  "sampled": false,
  "sample_bytes": 0,
  "content_type": "text/plain"
}
```

`content_type` 带 `#[serde(default)]`，因此缺少该字段的旧 `info.json` 仍能反序列化，并在读取时按 `extra` 补齐。

## 写入流程（post_storage）

1. 校验 `json_in`、UTF-8、JSON、`file_path` 是否存在（`400103`）。
2. 读取元数据并要求是普通文件：元数据读取失败 `500105`，目录/设备等 `400106`；`size` 取 `metadata.len()`。
3. 生成 UUID，`create_dir_all(./storage/<uuid>)` → 失败 `500100`。
4. `extra` 取 `Path::extension()`，无扩展名为空串。
5. 拷贝源文件到 `original` → 失败 `500101`。
6. `do_pack(original, original.z)` → 失败 `500102`。
7. `digest_file(original)`（≤ 8 MiB 全量，> 8 MiB 取大小 + 首尾各 4 MiB）→ 失败 `500104`；按 `extra` 推断 `content_type`。
8. 组装 `Ostfil`，`end_time` 取当前 UTC 秒级时间戳。
9. 写 `info.json` → 失败 `500103`。
10. 返回 `Ostfil`。

第 3 步之后任意一步失败都会先删除本次创建的 `storage/<uuid>/` 再返回错误，不留孤儿目录。

`original` 在压缩后**不会删除**，请求结束后目录内同时保留原文与压缩副本（见 [已知问题](./issues.md) I-04）。

## 读取流程（post_get_fil）

1. 校验入参，`uuid` 格式非法 → `400105`；`./storage/<uuid>` 不存在 → `400104`。
2. 读 `info.json` → 读失败 `500200`，解析失败 `500201`。
3. 内存中把 `end_time` 刷新为当前时间，**不回写**文件（见 [已知问题](./issues.md) I-08）。
4. 若 `original` 不存在，取 `original.z`（不存在则回退旧布局 `original.z.z`）解压还原，失败 `500202`。
5. 兼容旧记录：`content_type` 为空则按 `extra` 推断；`sha256` 为空且 `original` 存在则补算（同时补 `sampled` / `sample_bytes`）。
6. 返回 `Ostfil`。

## 删除流程（post_delete_file）

校验 `uuid` 格式（`400105`），随后直接 `remove_dir_all`：目录不存在（`NotFound`）→ `400104`，其他失败 → `500203`。不再先 `exists` 再删除，避免两步之间的竞态。

## 已修复的历史问题

以下问题在 2026-11-12 的修复中已解决，记录在此便于对照旧数据：

1. 压缩产物曾多一层 `.z`（实际为 `original.z.z`），现已改为 `original.z`；
2. `un_pack` 的参数语义曾与调用方不一致，导致解压必然失败，现已统一为 `(src_path, dst_path)`；
3. `uuid` 与 `file_path` 曾不做校验，现已分别返回 `400105`、`400106`。

仍在讨论的行为（是否删除 `original`、`end_time` 语义、存储根目录配置、并发锁与原子写等）见 [已知问题](./issues.md)。
