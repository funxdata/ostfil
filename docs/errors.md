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

## 5001xx — 写入阶段（post_storage）

| code | msg | 触发条件 |
| --- | --- | --- |
| `500100` | `failed to create dir: <详情>` | 创建 `storage/<uuid>` 失败 |
| `500101` | `failed to copy file: <详情>` | 拷贝到 `original` 失败 |
| `500102` | `compression failed: <详情>` | gzip 压缩失败 |
| `500103` | `failed to write info.json: <详情>` | 写元数据失败 |
| `500104` | `failed to compute md5: <详情>` | 读取文件计算 MD5 失败 |

## 5002xx — 读取与删除阶段

| code | msg | 触发条件 |
| --- | --- | --- |
| `500200` | `failed to read info.json: <详情>` | `post_get_fil` 读取元数据失败 |
| `500201` | `invalid json: <详情>` | `info.json` 内容无法反序列化为 `Ostfil` |
| `500202` | `decompression failed: <详情>` | `post_get_fil` 解压失败（当前实现下该分支不可达，见 [存储布局](./storage.md#当前实现的已知问题)） |
| `500203` | `delete failed: <详情>` | `post_delete_file` 删除目录失败 |

## 失败时的残留

`post_storage` 在步骤 1 之后任意一步失败都会直接返回，**不回滚**已创建的目录与已拷贝的文件，可能留下不含 `info.json` 的 `storage/<uuid>/`。失败响应不会回传 `uuid`，宿主如需严格一致，应在失败后自行扫描清理 `storage/` 下的残留目录。
