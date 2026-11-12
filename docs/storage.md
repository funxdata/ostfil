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
    ├── original.z.z  # gzip 压缩产物（当前实现）
    └── info.json     # 元数据（Ostfil 的 pretty JSON）
```

| 路径 | 产生者 | 说明 |
| --- | --- | --- |
| `<uuid>/original` | `post_storage` | `fs::copy` 源文件，保留原始扩展名之外的字节内容 |
| `<uuid>/original.z.z` | `post_storage` | gzip（最佳压缩级别）后的副本 |
| `<uuid>/info.json` | `post_storage` | `serde_json::to_string_pretty` 输出的 `Ostfil` |

`<uuid>` 为 v4 UUID 的字符串形式（36 字符，带连字符）。

## info.json

内容即 `Ostfil` 结构体，字段含义见 [接口说明](./api.md#ostfil-数据结构)：

```json
{
  "uuid": "3f0c1a52-...-9d0e",
  "size": 1024,
  "extra": "txt",
  "end_time": 1762531200,
  "md5": "9e107d9d372bb6826bd81d3542a419d6",
  "content_type": "text/plain"
}
```

`content_type` 带 `#[serde(default)]`，因此缺少该字段的旧 `info.json` 仍能反序列化，并在读取时按 `extra` 补齐。

## 写入流程（post_storage）

1. 校验 `json_in`、UTF-8、JSON、`file_path` 是否存在。
2. 生成 UUID，`create_dir_all(./storage/<uuid>)` → 失败 `500100`。
3. `extra` 取 `Path::extension()`，无扩展名为空串；`size` 取 `fs::metadata().len()`，失败按 `0`。
4. 拷贝源文件到 `original` → 失败 `500101`。
5. `do_pack(original, original.z)` → 失败 `500102`。
6. `compute_md5(original)` → 失败 `500104`；按 `extra` 推断 `content_type`。
7. 组装 `Ostfil`，`end_time` 取当前 UTC 秒级时间戳。
8. 写 `info.json` → 失败 `500103`。
9. 返回 `Ostfil`。

`original` 在压缩后**不会删除**，请求结束后目录内同时保留原文与压缩副本。

## 读取流程（post_get_fil）

1. 校验入参；`./storage/<uuid>` 不存在 → `400104`。
2. 读 `info.json` → 读失败 `500200`，解析失败 `500201`。
3. 内存中把 `end_time` 刷新为当前时间，**不回写**文件。
4. 若 `original` 不存在且 `original.z` 存在，尝试解压（失败 `500202`）。
5. 兼容旧记录：`content_type` 为空则按 `extra` 推断；`md5` 为空且 `original` 存在则补算。
6. 返回 `Ostfil`。

## 删除流程（post_delete_file）

`./storage/<uuid>` 不存在 → `400104`；否则 `remove_dir_all` 整目录 → 失败 `500203`。

## 当前实现的已知问题

以下行为由代码现状决定，宿主侧不应依赖，后续修复时以代码为准：

1. **压缩文件名多一层 `.z`**：`src/utils/pack.rs` 的 `do_pack(path, save_path)` 内部执行 `format!("{}.z", save_path)`，而 `storage.rs` 传入的 `save_path` 已是 `original.z`，因此实际产物是 `original.z.z`，`original.z` 并不存在。
2. **解压分支不会生效**：`info.rs` 在 `original` 缺失时判断并调用 `un_pack(original.z, original)`；但 `un_pack(path, extra)` 内部又拼成 `<path>.z` 与 `<path>.<extra>`，与调用方语义不匹配。加之 `original` 从不删除，该分支在正常流程下不可达。
3. **磁盘占用翻倍**：原文与压缩副本同时保留，`size` 仍记录原始大小。
4. **无并发控制**：同一 `uuid` 的并发读写、删除没有锁，依赖宿主串行化。
5. **不做路径校验**：`file_path` 与 `uuid` 均原样拼接，`uuid` 不做格式检查；调用方需保证 `uuid` 来自本库返回且不含路径分隔符。
