#include "gtest/gtest.h"
#include <uv.h>
#include <string>
#include "../include/ostfil.h"

static bool g_write_done = false;
static bool g_read_done = false;
static bool g_del_done = false;

void write_cb(int status, void *user_data) {
    EXPECT_EQ(status, 0);
    g_write_done = true;
}

void read_cb(int status, void *user_data) {
    EXPECT_EQ(status, 0);
    g_read_done = true;
}

void del_cb(int status, void *user_data) {
    EXPECT_EQ(status, 0);
    g_del_done = true;
}

TEST(OstfilAsync, WriteReadDelete) {
    const char *src_file = "tests/test_input.txt";
    const char *file_path = "tests/data_store";
    char file_name[64] = {0};
    int slice_sum = 0;

    // 异步写入
    ostfil_write_async(src_file, file_path, file_name, &slice_sum, write_cb, NULL);
    uv_run(uv_default_loop(), UV_RUN_DEFAULT);
    ASSERT_TRUE(g_write_done);
    ASSERT_GT(slice_sum, 0);
    ASSERT_STRNE(file_name, "");

    // 异步读取
    char dest_file[256] = "tests/test_output.txt";
    ostfil_read_async(file_path, file_name, dest_file, read_cb, NULL);
    uv_run(uv_default_loop(), UV_RUN_DEFAULT);
    ASSERT_TRUE(g_read_done);

    // 异步删除
    ostfil_del_async(file_path, file_name, del_cb, NULL);
    uv_run(uv_default_loop(), UV_RUN_DEFAULT);
    ASSERT_TRUE(g_del_done);
}
