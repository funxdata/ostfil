# ostfil 已知问题

> 记录对 `1.0.4` 源码做静态审计时发现的缺陷：现象、影响与修复方向。条目编号固定，修复后请更新状态而不是删除条目。最近更新：2026-11-12。

## 汇总

| 编号 | 问题 | 严重程度 | 状态 |
| --- | --- | --- | --- |
| I-01 | `uuid` 未校验，可越出 `storage/` 删除或读取任意目录 | 高 | 未修复 |
| I-02 | 压缩产物实际为 `original.z.z` | 中 | 未修复 |
| I-03 | `un_pack` 参数语义错误，解压功能不可用 | 中 | 未修复 |
| I-04 | `original` 压缩后不删除，磁盘占用翻倍 | 中 | 未修复 |
| I-05 | `post_storage` 失败不回滚，且错误响应不带 `uuid` | 中 | 未修复 |
| I-06 | 函数返回值不表示业务成败 | 中 | 未修复 |
| I-07 | MD5 全量读入内存，大文件可能 OOM | 中 | 未修复 |
| I-08 | `end_time` 更新不持久化，字段名与语义不符 | 中 | 未修复 |
| I-09 | 存储根目录依赖进程工作目录，无配置入口 | 中 | 未修复 |
| I-10 | 无并发控制，非原子写入 | 中 | 未修复 |
| I-11 | 没有任何测试 | 中 | 未修复 |
| I-12 | FFI 边界存在 `unwrap()`，panic 会终止宿主进程 | 中 | 未修复 |
| I-13 | `file_path` 不校验是否为普通文件 | 低 | 未修复 |
| I-14 | `size` 吞掉错误，与 0 字节文件不可区分 | 低 | 未修复 |
| I-15 | 冗余依赖 `libc`、`once_cell` | 低 | 未修复 |
| I-16 | 输出缓冲区上限 65536 不可协商 | 低 | 未修复 |
| I-17 | `content_type` 覆盖范围有限 | 低 | 未修复 |
| I-18 | 发布流程存在不可达分支，未传 `DYTES_S3_ENDPOINT` | 低 | 未修复 |

---

## I-01 `uuid` 未校验，可越出 `storage/` 删除或读取任意目录

**严重程度**：高 ｜ **状态**：未修复 ｜ **位置**：`src/ffi/delete.rs:32`、`src/ffi/info.rs:34`

### 现象

两个函数都直接用 `Path::new(STORAGE_DIR).join(&req.uuid)` 拼路径，`uuid` 不做任何格式检查：

```rust
let uuid_dir = Path::new(STORAGE_DIR).join(&req.uuid);
```

`Path::join` 在参数为绝对路径时会**整体替换**左边，因此：

```json
{"uuid":"D:/some/dir"}      // Windows：直接指向 D:/some/dir
{"uuid":"../../important"}  // 相对路径同样可逃逸
```

`post_delete_file` 随后执行 `fs::remove_dir_all(&uuid_dir)`。

### 影响

宿主若把外部输入（前端、配置、网络）直接透传给 `uuid`，等价于暴露了一个「删除任意目录」的接口；`post_get_fil` 则能读取任意目录下的 `info.json`。属于可被滥用为破坏性操作的安全问题，而不只是健壮性问题。

### 修复方向

在 FFI 入口把 `uuid` 解析为 `Uuid` 类型（`uuid` 依赖已在），或用白名单校验 36 字符、十六进制与连字符；同时断言拼接结果仍以 `STORAGE_DIR` 为前缀。校验失败返回 `4001xx` 新错误码。

---

## I-02 压缩产物实际为 `original.z.z`

**严重程度**：中 ｜ **状态**：未修复 ｜ **位置**：`src/utils/pack.rs:9`、`src/ffi/storage.rs:70-74`

### 现象

`do_pack(path, save_path)` 内部再次追加 `.z`：

```rust
let output_file_path = format!("{}.z", save_path);
```

而调用方传入的 `save_path` 已经是 `original.z`，最终产物是 `original.z.z`，注释与实际行为不符。

### 影响

- `original.z` 永远不会存在；
- `post_get_fil` 中判断 `compressed_path.exists()`（即 `original.z`）恒为 `false`，与 I-03 一起导致解压链路完全失效；
- 依赖 `original.z` 命名的外部脚本/清理工具会找不到文件。

### 修复方向

统一「目标路径」语义：`do_pack(src, dst)` 直接写 `dst`，后缀由调用方决定；或让调用方传 `original` 并由函数补 `.z`。修复后同步更新 `docs/storage.md`。

---

## I-03 `un_pack` 参数语义错误，解压功能不可用

**严重程度**：中 ｜ **状态**：未修复 ｜ **位置**：`src/utils/pack.rs:25-30`、`src/ffi/info.rs:63-68`

### 现象

`un_pack(path, extra)` 内部把两个参数都当**后缀**使用：

```rust
let input_file_path = format!("{}.z", path);
let output_file_path = format!("{}.{}", path, extra);
```

调用方（`info.rs`）却传入完整路径：

```rust
un_pack(compressed_path.to_str().unwrap(), original_path.to_str().unwrap())
```

实际执行时：读取 `.../original.z.z`（恰好是 I-02 的真实产物），写入 `.../original.z../storage/<uuid>/original` —— 目标路径上级目录不存在，`File::create` 失败，返回 `500202`。

### 影响

`original` 被删除或丢失后，无法通过压缩副本恢复，且错误信息只说「decompression failed」。正常流程因 `original` 从不删除（I-04）而不会触发，问题被掩盖。

### 修复方向

同 I-02 一起重构 `do_pack` / `un_pack` 为 `(src_path, dst_path)` 语义，并补一条「删除 `original` 后 `post_get_fil` 能还原」的测试。

---

## I-04 `original` 压缩后不删除，磁盘占用翻倍

**严重程度**：中 ｜ **状态**：未修复 ｜ **位置**：`src/ffi/storage.rs:69-76`

### 现象

压缩完成后没有 `fs::remove_file(&original_path)`，目录里同时保留原文与 `.z` 副本。`skills/ostfil.md` 的原始设计是「压缩保存为 `original.z`」，未提保留原文。

### 影响

每个文件占用约 2 倍空间（压缩率高的文本接近 2 倍，已压缩格式更高），`size` 仍记录原文大小，宿主据此做的容量统计会低估实际占用。

### 修复方向

压缩成功后删除 `original`，并依赖 I-02/I-03 修好的解压链路还原；或在 `info.json` 增加 `size_compressed` 字段并保留原文（需明确产品语义）。

---

## I-05 `post_storage` 失败不回滚，且错误响应不带 `uuid`

**严重程度**：中 ｜ **状态**：未修复 ｜ **位置**：`src/ffi/storage.rs:45-103`

### 现象

创建目录之后的任一失败（拷贝、压缩、MD5、写 `info.json`）都直接 `return write_error(...)`，不清理已创建的 `storage/<uuid>/`。错误信封固定为 `{code, data:{}, msg}`，不携带 `uuid`。

### 影响

- 留下没有 `info.json` 的孤儿目录，`post_get_fil` 对它会返回 `500200`；
- 调用方拿不到 `uuid`，无法主动清理；
- 反复失败会持续累积垃圾目录。

### 修复方向

失败路径统一走清理（删除本次创建的目录）；把 `uuid` 放进错误响应 `data`，或在错误 `msg` 中附带。

---

## I-06 函数返回值不表示业务成败

**严重程度**：中 ｜ **状态**：未修复 ｜ **位置**：`src/utils/cjson.rs:26-45`

### 现象

`write_error` 与 `write_success` 都 `return write_json_out(...)`，只要缓冲区可写就返回 `0`。因此业务失败时三个函数同样返回 `0`，`-1`/`-2` 只表示缓冲区问题。

### 影响

C 调用方按惯例「0 即成功」，极易把错误当成功；文档已注明，但契约本身反直觉。

### 修复方向

业务错误返回 `code` 本身或统一非 0（如 `1`），并在 `docs/ffi.md` 的返回值表中同步；若要兼容现有调用方，可先加一个 `ostfil_last_error` 之类的查询接口。

---

## I-07 MD5 全量读入内存，大文件可能 OOM

**严重程度**：中 ｜ **状态**：未修复 ｜ **位置**：`src/utils/md.rs:6`

### 现象

```rust
let bytes = fs::read(path)?;
```

一次性把整个文件读进 `Vec<u8>` 再计算 MD5，没有大小上限。压缩路径是流式的，只有 MD5 不是。

### 影响

`post_storage` 处理数 GB 文件时，进程内存峰值约等于文件大小；在 32 位宿主或内存受限容器中会分配失败甚至被 OOM killer 终止。

### 修复方向

改为 `BufReader` + `md5::Context` 分块更新（64 KiB 一次），不改变结果。

---

## I-08 `end_time` 更新不持久化，字段名与语义不符

**严重程度**：中 ｜ **状态**：未修复 ｜ **位置**：`src/ffi/info.rs:57`

### 现象

`post_get_fil` 把 `end_time` 改成当前时间后直接返回，**没有写回** `info.json`；`post_storage` 写入的则是入库时间。字段名 `end_time` 在 `skills/ostfil.md` 中描述为「最后一次打开时间」。

### 影响

宿主看到「最后访问时间」只对本次调用有效，重启或重新读取后又变回入库时间；若宿主依赖该字段做清理策略，行为不可预期。

### 修复方向

明确语义后二选一：改名 `last_access_time` 并在读取时写回（注意 I-10 的原子写），或保持只读语义并删除「刷新」逻辑。

---

## I-09 存储根目录依赖进程工作目录，无配置入口

**严重程度**：中 ｜ **状态**：未修复 ｜ **位置**：`src/ffi/types.rs:14`

### 现象

`STORAGE_DIR` 是硬编码的相对路径 `./storage`，所有读写都以宿主进程的 CWD 为基准，库没有提供设置根目录的接口。

### 影响

宿主切换工作目录（例如打开文件对话框、以服务方式启动）后，会读写到另一套 `storage/`，表现为「文件丢失」；同一份数据在不同启动方式下可能被写到不同位置。

### 修复方向

新增 `ostfil_init(const char *root)` 或从环境变量（如 `OSTFIL_STORAGE_DIR`）读取绝对路径，默认值保持 `./storage` 以兼容现有部署。

---

## I-10 无并发控制，非原子写入

**严重程度**：中 ｜ **状态**：未修复 ｜ **位置**：`src/utils/info.rs:20`、`src/ffi/delete.rs:37-39`

### 现象

- `save_info` 直接 `fs::write` 覆盖 `info.json`，写入中途崩溃会留下截断文件；
- 没有文件锁或进程内互斥，同一 `uuid` 的读写/删除可并发；
- `post_delete_file` 先 `exists()` 再 `remove_dir_all`，两步之间目录可能已被删除（TOCTOU），返回 `500203` 而不是 `400104`。

### 影响

并发调用下会出现「`info.json` 损坏 → `500201`」或删除误报失败；宿主需要自行串行化，但接口没有说明。

### 修复方向

写入采用「临时文件 + `rename`」；用 `std::sync::Mutex` 或其他机制保护同一 `uuid` 的写与删；删除侧直接调用 `remove_dir_all` 并处理 `NotFound`，去掉前置 `exists`。

---

## I-11 没有任何测试

**严重程度**：中 ｜ **状态**：未修复 ｜ **位置**：全仓库

### 现象

`src/` 中无 `#[cfg(test)]`，仓库无 `tests/` 目录，CI 只做 `cargo build --release`，不跑 `cargo test`。

### 影响

I-02、I-03 这类「编译通过、运行错误」的缺陷无法被发现；后续修 FFI 契约（I-06、I-12）时没有回归保护。

### 修复方向

先补三个最小用例：`pack`/`unpack` 往返、`post_storage` 成功路径、`uuid` 非法输入的拒绝；CI 增加 `cargo test`。

---

## I-12 FFI 边界存在 `unwrap()`，panic 会终止宿主进程

**严重程度**：中 ｜ **状态**：未修复 ｜ **位置**：`src/ffi/storage.rs:72-73`、`src/ffi/storage.rs:96`、`src/ffi/info.rs:63-68`

### 现象

路径转换直接 `to_str().unwrap()`：

```rust
original_path.to_str().unwrap()
```

路径按 UTF-8 无损转换失败（非 UTF-8 文件名、部分 Windows 场景）时会 panic。Rust 不允许 unwind 越过 `extern "C"` 边界，会直接 abort，宿主进程随之终止。

### 影响

一个畸形路径就能让宿主（桌面应用、服务）崩溃，而不是返回一个错误码。

### 修复方向

改用 `to_string_lossy()` 或 `Option` 处理并返回错误码；或在每个 FFI 入口包一层 `std::panic::catch_unwind`（注意 `UnwindSafe`），至少保证不会带崩宿主。

---

## I-13 `file_path` 不校验是否为普通文件

**严重程度**：低 ｜ **状态**：未修复 ｜ **位置**：`src/ffi/storage.rs:36-39`

### 现象

只检查 `file_path.exists()`。目录、设备文件、符号链接同样通过，随后 `fs::copy` 失败并返回 `500101`「failed to copy file」。

### 影响

错误码与错误信息误导调用方（看起来像磁盘问题）；也可能把 `storage/` 自身作为输入，产生自我复制的副本。

### 修复方向

先 `metadata()` 并检查 `is_file()`，不满足时返回明确的输入错误码；可选限制不得位于 `STORAGE_DIR` 之下。

---

## I-14 `size` 吞掉错误，与 0 字节文件不可区分

**严重程度**：低 ｜ **状态**：未修复 ｜ **位置**：`src/ffi/storage.rs:58-61`

### 现象

```rust
let size = match fs::metadata(file_path) {
    Ok(meta) => meta.len(),
    Err(_) => 0,
};
```

读取元数据失败被静默转成 `0`。

### 影响

空文件与「拿不到大小」在返回结果中完全相同，宿主无法判断；结合 I-13，输入异常时会写入错误的 `size`。

### 修复方向

失败时返回错误码而非 `0`；或在 `info.json` 增加 `size_known: bool`。

---

## I-15 冗余依赖 `libc`、`once_cell`

**严重程度**：低 ｜ **状态**：未修复 ｜ **位置**：`Cargo.toml:15-16`

### 现象

`src/` 中没有任何 `libc::` 或 `once_cell` 引用；`serde_derive` 与 `serde` 的 `derive` feature 也重复声明。

### 影响

增加编译时间与依赖面，没有功能收益。

### 修复方向

删除 `libc`、`once_cell`；保留 `serde = { version = "1.0", features = ["derive"] }` 并去掉单独的 `serde_derive`。

---

## I-16 输出缓冲区上限 65536 不可协商

**严重程度**：低 ｜ **状态**：未修复 ｜ **位置**：`src/utils/cjson.rs:14-16`

### 现象

```rust
if bytes.len() >= 65536 { return -2; }
```

超限时直接返回 `-2`，不写入任何内容，也不告知所需长度。

### 影响

`Ostfil` 本身很小，但错误 `msg` 含长路径/系统错误文本时输出会增长；调用方无法得知需要多大缓冲，只能盲目加大。

### 修复方向

新增 `ostfil_output_size(...)` 或带长度的调用约定；短期可在文档中写明上限的计算依据。

---

## I-17 `content_type` 覆盖范围有限

**严重程度**：低 ｜ **状态**：未修复 ｜ **位置**：`src/ffi/storage.rs:112`

### 现象

`content_type_for` 只覆盖 12 组扩展名，其余一律 `application/octet-stream`。

### 影响

`csv`、`xml`、`yaml`、`toml`、`mp4`、`mp3`、`woff2`、`ttf`、`tar`、`7z` 等常见类型没有正确 MIME，宿主若据此选择预览方式会退化为下载。

### 修复方向

扩充映射表，或引入 `mime_guess` 一类只依赖扩展名的库。

---

## I-18 发布流程存在不可达分支，未传 `DYTES_S3_ENDPOINT`

**严重程度**：低 ｜ **状态**：未修复 ｜ **位置**：`.github/workflows/dytes-release.yml:35-41`、`:55-59`

### 现象

- `prepare` 会探测 `CMakeLists.txt` / `Cargo.toml` / `deno`，但本仓库只有 `Cargo.toml`，cmake 的两个安装步骤与 cmake/deno 构建分支永远不执行；
- job 级 `env` 只映射了 bucket/access/secret，`DYTES_S3_ENDPOINT` 依赖 dytes 内置默认值。

### 影响

死分支增加阅读成本；如果 dytes 的默认 endpoint 不是 `https://s3.api.upyun.com`，发布会直接失败。

### 修复方向

删除不可达分支，或在注释中标注「为多语言模板保留」；确认 endpoint 后在 job `env` 中显式声明。
