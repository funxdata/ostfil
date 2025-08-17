#include "../include/ostfil.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// 异步创建目录
void mkdir_async(const char *dir_path, uv_fs_cb cb, void *user_data) {
    uv_fs_t *req = malloc(sizeof(uv_fs_t));
    uv_fs_mkdir(uv_default_loop(), req, dir_path, 0755, cb);
    req->data = user_data;
}

// 异步打开文件
void open_async(const char *file_path, int flags, int mode, uv_fs_cb cb, void *user_data) {
    uv_fs_t *req = malloc(sizeof(uv_fs_t));
    uv_fs_open(uv_default_loop(), req, file_path, flags, mode, cb);
    req->data = user_data;
}

// 异步读写、关闭、删除等函数可类似封装