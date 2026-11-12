# 错误码

响应中的 `code` 为 `200` 表示业务成功，其余为错误。错误响应格式固定：

```json
{"code":400103,"data":{},"msg":"file does not exist"}
```

## 函数返回码

| 返回 | 含义 |
| --- | --- |
| `0` | JSON 已写入 `json_out`，业务结果看 `code` |
| `-1` | `json_out` 为空指针 |
| `-2` | 序列化结果达到 65536 字节，未写入 |

注意：业务错误同样返回 `0`（错误信封写入成功），不要用返回值判断业务成败。

## 4001xx — 输入与请求

三个函数共用这几项：

| code | msg | 触发条件 |
| --- | --- | --- |
| `400100` | `json_in is null` | 传入空指针 |
| `400101` | `invalid utf-8` | `json_in` 不是合法 UTF-8 或没有 NUL 结尾 |
| `400102` | `invalid json: <详情>` | JSON 解析失败 / 缺少必需字段 |
| `400103` | `file does not exist` | `post_storage` 的 `file_path` 不存在 |
| `400104` | `uuid directory does not exist` | `post_get_fil` / `post_delete_file` 的 `storage/<uuid>` 不存在 |
| `400105` | `invalid uuid` | `uuid` 不是 36 位十六进制 + 连字符形式（拦截 `../`、绝对路径等越界输入） |
| `400106` | `file_path is not a regular file` | `post_storage` 的 `file_path` 存在但不是普通文件（目录、设备等） |

## 5001xx — 写入阶段（post_storage）

| code | msg | 触发条件 |
| --- | --- | --- |
| `500100` | `failed to create dir: <详情>` | 创建 `storage/<uuid>` 失败 |
| `500101` | `failed to copy file: <详情>` | 拷贝到 `original` 失败 |
| `500102` | `compression failed: <详情>` | gzip 压缩失败 |
| `500103` | `failed to write info.json: <详情>` | 写元数据失败 |
| `500104` | `failed to compute md5: <详情>` | 读取文件计算 MD5 失败 |
| `500105` | `failed to read metadata: <详情>` | 读取 `file_path` 元数据失败 |
| `500106` | `storage path is not valid utf-8` | 拼接出的存储路径无法转为 UTF-8（三个函数均可能返回） |

## 5002xx — 读取与删除阶段

| code | msg | 触发条件 |
| --- | --- | --- |
| `500200` | `failed to read info.json: <详情>` | `post_get_fil` 读取元数据失败 |
| `500201` | `invalid json: <详情>` | `info.json` 内容无法反序列化为 `Ostfil` |
| `500202` | `decompression failed: <详情>` | `post_get_fil` 从 `original.z`（或旧布局的 `original.z.z`）还原 `original` 失败 |
| `500203` | `delete failed: <详情>` | `post_delete_file` 删除目录失败（权限不足、目标是文件而非目录等） |

## 失败时的清理

`post_storage` 在创建 `storage/<uuid>/` 之后任意一步失败，会先删除本次创建的目录再返回错误，不会留下孤儿目录。清理只覆盖本次新建的目录，失败响应仍然不回传 `uuid`（见 [已知问题](./issues.md) I-05）。
