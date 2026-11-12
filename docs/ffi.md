# FFI 接口

库导出三个 `extern "C"` 函数，全部采用「JSON 字符串进、JSON 字符串出」的约定：

```c
int post_storage(const char *json_in, char *json_out);
int post_get_fil(const char *json_in, char *json_out);
int post_delete_file(const char *json_in, char *json_out);
```

## 通用约定

### 输入

- `json_in` 必须是 NUL 结尾的 UTF-8 JSON 字符串；传 `NULL` 返回 `400100`，非法 UTF-8 返回 `400101`，JSON 解析失败返回 `400102`。
- 输入只按 JSON 解析，不校验多余字段。

### 输出

- `json_out` 由调用方分配，**容量至少 65536 字节**。函数写入 NUL 结尾的 JSON。
- 输出长度（不含 NUL）达到 65536 时放弃写入并返回 `-2`。本库的响应只包含 `uuid`、路径、校验值等短字段，正常不会接近该上限，因此不提供可协商的缓冲区接口。
- `json_out` 为 `NULL` 时返回 `-1`。

### 响应信封

成功：

```json
{"code":200,"data":{},"msg":"success"}
```

失败：

```json
{"code":400103,"data":{},"msg":"file does not exist"}
```

字段顺序固定为 `code`、`data`、`msg`（`serde_json` 默认按键排序）。

### 返回值语义（重要）

函数返回值只表示**输出缓冲区是否写入成功**，不表示业务是否成功：

| 返回 | 含义 |
| --- | --- |
| `0` | JSON 已写入 `json_out`（业务成败看 `code`） |
| `-1` | `json_out` 为空指针 |
| `-2` | 输出超过 65536 字节，未写入 |

因此业务错误（如文件不存在）同样返回 `0`。调用方应先比较返回值，再解析 `code` 判断业务结果。

## Ostfil 数据结构

`data` 中的元数据对象，字段顺序与声明一致：

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

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `uuid` | string | 存储目录名，v4 UUID |
| `size` | u64 | 原始文件字节数 |
| `extra` | string | 扩展名，不含 `.`，取 `Path::extension()` 原样（大小写保留）；无扩展名为 `""` |
| `end_time` | u64 | Unix 时间戳（秒，UTC）。写入时是入库时间，每次 `post_get_fil` 刷新为最近读取时间 |
| `sha256` | string | 文件校验摘要，小写十六进制；计算规则见下 |
| `sampled` | bool | 是否按大文件采样规则计算 |
| `sample_bytes` | u64 | 采样时首尾各取的字节数（4194304），非采样为 `0` |
| `content_type` | string | 依据 `extra` 推断的 MIME 类型；旧记录缺失时在读取时补齐 |

**校验规则（与 dytes 的 `FileChecksum` 一致）**：

- `size <= 8 MiB`：整个文件的 SHA-256，`sampled = false`、`sample_bytes = 0`；
- `size > 8 MiB`：`size`（u64 小端）+ 首 4 MiB + 尾 4 MiB 的 SHA-256，`sampled = true`、`sample_bytes = 4194304`。

大文件因此不需要读取全部内容；写入与后续校验使用同一规则。

`content_type` 映射（比较时转小写）：

| 扩展名 | MIME |
| --- | --- |
| `txt` / `md` / `log` | `text/plain` |
| `json` | `application/json` |
| `html` / `htm` | `text/html` |
| `css` | `text/css` |
| `js` / `mjs` | `application/javascript` |
| `ts` | `application/typescript` |
| `csv` | `text/csv` |
| `xml` | `application/xml` |
| `yaml` / `yml` | `application/yaml` |
| `toml` | `application/toml` |
| `png` | `image/png` |
| `jpg` / `jpeg` | `image/jpeg` |
| `gif` | `image/gif` |
| `webp` | `image/webp` |
| `svg` | `image/svg+xml` |
| `bmp` | `image/bmp` |
| `ico` | `image/x-icon` |
| `mp3` | `audio/mpeg` |
| `wav` | `audio/wav` |
| `ogg` | `audio/ogg` |
| `mp4` | `video/mp4` |
| `webm` | `video/webm` |
| `woff` | `font/woff` |
| `woff2` | `font/woff2` |
| `ttf` | `font/ttf` |
| `otf` | `font/otf` |
| `pdf` | `application/pdf` |
| `zip` | `application/zip` |
| `gz` | `application/gzip` |
| `tar` | `application/x-tar` |
| `7z` | `application/x-7z-compressed` |
| 其他 | `application/octet-stream` |

## post_storage

把文件纳入管理，返回元数据。

请求：

```json
{"file_path":"D:/data/a.txt"}
```

- `file_path`：源文件路径。可以是绝对路径，也可以是相对宿主进程工作目录的路径。

行为：

1. 校验 `json_in`、JSON 与 `file_path` 是否存在（不存在返回 `400103`）。
2. 要求 `file_path` 是普通文件：元数据读取失败 `500105`，目录/设备等返回 `400106`。
3. 生成 v4 UUID，创建 `./storage/<uuid>/`（失败 `500100`）。
4. 拷贝源文件到 `<uuid>/original`（失败 `500101`）。
5. gzip 压缩到 `<uuid>/original.z`（失败 `500102`），详见 [存储布局](./storage.md)。
6. 计算校验摘要（失败 `500104`）、推断 `content_type`。
7. 写 `<uuid>/info.json`（失败 `500103`）。
8. 返回 `Ostfil` 对象，`msg` 为 `success`。

第 3 步之后的失败会先清理本次创建的目录，不留孤儿目录。

成功响应：

```json
{"code":200,"data":{"uuid":"...","size":12,"extra":"txt","end_time":1762531200,"sha256":"...","sampled":false,"sample_bytes":0,"content_type":"text/plain"},"msg":"success"}
```

副作用：写入 `./storage/<uuid>/`；源文件保持原样。

## post_get_fil

按 `uuid` 读取元数据。

请求：

```json
{"uuid":"3f0c1a52-...-9d0e"}
```

行为：

1. 校验入参，`uuid` 格式非法返回 `400105`；目录不存在返回 `400104`。
2. 读取 `<uuid>/info.json`（失败 `500200`，内容不是合法 JSON 返回 `500201`）。
3. 把 `end_time` 刷新为当前 Unix 时间（仅返回内存中的值，**不回写** `info.json`）。
4. 若 `original` 不存在，取 `original.z`（缺失时回退旧布局 `original.z.z`）解压还原（失败 `500202`）。
5. 旧记录兼容：`content_type` 为空时按 `extra` 重新推断；`sha256` 为空且 `original` 存在时重新计算（含 `sampled` / `sample_bytes`）。
6. 返回 `Ostfil` 对象。

`uuid` 会先校验为 36 位 UUID 形式（十六进制 + 连字符），`../`、绝对路径等输入返回 `400105`，不会越出 `storage/`。

## post_delete_file

删除一个文件及其元数据。

请求：

```json
{"uuid":"3f0c1a52-...-9d0e"}
```

行为：先校验 `uuid` 格式（非法返回 `400105`），随后 `remove_dir_all` 删除整个 `storage/<uuid>/`；目录不存在返回 `400104`，其他失败返回 `500203`。

成功响应：

```json
{"code":200,"data":{},"msg":"success"}
```

删除是破坏性操作：整个目录被移除，无回收站。重复删除同一 `uuid` 会返回 `400104`。

## C 调用示例

```c
#include <stdio.h>
#include <string.h>

extern int post_storage(const char *json_in, char *json_out);
extern int post_get_fil(const char *json_in, char *json_out);
extern int post_delete_file(const char *json_in, char *json_out);

int main(void) {
    static char out[65536];

    int rc = post_storage("{\"file_path\":\"D:/data/a.txt\"}", out);
    if (rc != 0) {
        printf("buffer error: %d\n", rc);
        return 1;
    }
    printf("storage -> %s\n", out);

    rc = post_get_fil("{\"uuid\":\"3f0c1a52-...-9d0e\"}", out);
    printf("get     -> rc=%d %s\n", rc, out);

    rc = post_delete_file("{\"uuid\":\"3f0c1a52-...-9d0e\"}", out);
    printf("delete  -> rc=%d %s\n", rc, out);
    return 0;
}
```

编译（以 GCC 为例）：

```bash
gcc demo.c -L target/release -lostfil -o demo
```

Windows 下需确保 `ostfil.dll` 与可执行文件同目录或在 `PATH` 中。
