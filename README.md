# ostfil
local file manager






## storage

`storage` 会把文件写入 `./storage/<uuid>/`（副作用），请求 `file_path` 必须是绝对路径。

- 返回 `size` / `sha256` / `content_type`（大文件另带 `sampled` / `sample_bytes`）；
- 文件不存在返回 `400103`；
- 拷贝失败 `500101`，压缩失败 `500102`，info 写入失败 `500103`，校验摘要计算失败 `500104`；
- `get_fil` 对旧记录会按需补算 `sha256` 与 `content_type`。
