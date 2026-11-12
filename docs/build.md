# 构建与发布

## 环境

- Rust stable（`#[unsafe(no_mangle)]` 语法需要 1.82 及以上）
- 无系统级原生依赖；`flate2` 默认使用纯 Rust 后端

依赖（`Cargo.toml`）：`chrono`、`flate2`、`libc`、`once_cell`、`serde`、`serde_derive`、`serde_json`、`uuid`（`v4`）、`md5`。

## 构建

```bash
cargo build --release
```

`[lib]` 声明了 `crate-type = ["cdylib"]`、`name = "ostfil"`，因此只产出动态库：

| 平台 | 产物 |
| --- | --- |
| Windows | `target/release/ostfil.dll` |
| Linux | `target/release/libostfil.so` |
| macOS | `target/release/libostfil.dylib` |

本仓库没有单元测试与集成测试；验证以构建通过与宿主侧调用为主。

## 包元信息

`dytes.json`：

```json
{
  "name": "ostfil",
  "uuid": "0bddf9d9-9a72-43e7-ac64-6e9e39b773be",
  "version": "1.0.4",
  "type": "sys",
  "platform-exports": {
    "windows": { "./target/release/ostfil.dll": "libostfil.dll" },
    "linux": { "./target/release/libostfil.so": "libostfil.so" },
    "macos": { "./target/release/libostfil.dylib": "libostfil.dylib" }
  }
}
```

- `type: "sys"`：发布到系统能力库，安装侧落在 `library/`。
- `platform-exports` 的键同时决定发布矩阵：workflow 从这三个平台生成 matrix。
- Windows 的导出名是 `libostfil.dll`（不是 `ostfil.dll`），安装后宿主按 `libostfil.dll` 加载。

## 版本管理

发布前需同步三处版本号，`prepare` job 会校验 tag 与 `dytes.json` 一致：

1. `dytes.json` 的 `version`
2. `Cargo.toml` 的 `[package] version`
3. `Cargo.lock` 中 `name = "ostfil"` 条目的 `version`

```bash
# 示例：1.0.4 -> 1.0.5
# 改完三处后提交，再打 tag
git tag v1.0.5
git push origin main --tags
```

tag 必须形如 `vX.Y.Z`，且去掉前缀后与 `dytes.json` 版本完全相同，否则 `prepare` 直接失败。

## 发布流程

工作流：`.github/workflows/dytes-release.yml`，触发条件为推送 `v*` tag 或手动 `workflow_dispatch`。

### prepare（ubuntu-latest）

1. 读取 `dytes.json` 版本，与 tag 比对。
2. 从 `platform-exports` 的键生成平台矩阵。
3. 探测构建方式：存在 `CMakeLists.txt` 用 cmake，否则存在 `Cargo.toml` 用 cargo，都无则 deno。当前仓库为 `cargo`。

### publish（每平台一个 runner，串行）

`max-parallel: 1`，`fail-fast: false`：某个平台失败不影响其余平台继续。

1. `cargo build --release`。
2. 主路径：`uses: dytescn/dytes@v0.2.1`，`command: publish`，`version: 0.2.1`（官方 action 的默认值仍是 `0.1.27`，必须显式传入）。
3. 兜底路径：若上面的 Action 失败（`steps.dytes.outcome == 'failure'`），改用脚本下载 dytes 后自行 `publish`：
   - 先尝试 GitHub Release（`dytes-linux` / `dytes-macos` / `dytes-windows.exe`），最多 12 次、间隔 8 秒；
   - 全部失败后改走 CDN：`https://packages.funxdata.com/core/dytes/<version>/<platform>/<导出名>`，0.2.x 的导出名为 `linux/dytes.bin`、`macos/dytes.app`、`windows/dytes.exe`。
4. dytes 按 `platform-exports` 上传本平台产物，并更新 `<type>/<name>/<version>/<platform>/` 下的清单（按 dytes 规则即 `sys/ostfil/<version>/<platform>/`）。

### 凭据

job 级环境变量，由仓库 Secrets 提供：

| 环境变量 | Secret |
| --- | --- |
| `DYTES_S3_BUCKET` | `UPYUN_BUCKET` |
| `DYTES_S3_ACCESS_KEY` | `UPYUN_API_KEY` |
| `DYTES_S3_SECRET_KEY` | `UPYUN_API_SECRET` |

`DYTES_S3_ENDPOINT` 未显式传递，使用 dytes 内置默认值。若默认 endpoint 不是 UpYun 的 `https://s3.api.upyun.com`，需在 workflow 的 job `env` 中补上该变量。

## 已移除的工作流

`linux.yml`、`mac.yml`、`win.yml` 已删除。它们原先在 `main` 分支推送时直接构建并上传到 `system/<name>/<version>/{linux,mac,win}`，与 dytes 发布重复，现统一由 `dytes-release.yml` 负责。
