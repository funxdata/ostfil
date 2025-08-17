#ifndef FILE_OPS_H
#define FILE_OPS_H

#include <uv.h>

// 异步创建目录
void mkdir_async(const char *dir_path, uv_fs_cb cb, void *user_data);

// 异步打开文件
void open_async(const char *file_path, int flags, int mode, uv_fs_cb cb, void *user_data);

#endif // FILE_OPS_H