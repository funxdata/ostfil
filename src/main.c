#include <stdio.h>
#include <string.h>
#include <uv.h>
#include "../include/ostfil.h"

void write_cb(int status, void *user_data) {
    if (status == 0)
        printf("文件写入成功\n");
    else
        printf("文件写入失败: %d\n", status);
}

void read_cb(int status, void *user_data) {
    if (status == 0)
        printf("文件读取成功\n");
    else
        printf("文件读取失败: %d\n", status);
}

void del_cb(int status, void *user_data) {
    if (status == 0)
        printf("文件删除成功\n");
    else
        printf("文件删除失败: %d\n", status);
}

int main() {
    const char *src_file = "test_input.txt";        // 源文件
    const char *file_path = "data_store";          // 存储目录
    char file_name[64] = {0};
    int slice_sum = 0;

    // 异步写入
    ostfil_write_async(src_file, file_path, file_name, &slice_sum, write_cb, NULL);

    // 运行 uv loop，确保异步操作执行
    uv_run(uv_default_loop(), UV_RUN_DEFAULT);

    printf("生成文件 UUID: %s, 切片数: %d\n", file_name, slice_sum);

    // 异步读取
    char dest_file[256] = "test_output.txt";
    ostfil_read_async(file_path, file_name, dest_file, read_cb, NULL);
    uv_run(uv_default_loop(), UV_RUN_DEFAULT);

    // 异步删除
    ostfil_del_async(file_path, file_name, del_cb, NULL);
    uv_run(uv_default_loop(), UV_RUN_DEFAULT);

    return 0;
}
