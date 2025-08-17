#ifndef OSTFIL_H
#define OSTFIL_H

#include <stddef.h>
#include "../crates/libuv/include/uv.h"
#include "../crates/zlib/zlib.h"
#include "../crates/uuid4/src/uuid4.h"
#include "file_ops.h"
#include "compression.h"


#ifdef __cplusplus
extern "C" {
#endif

// 异步回调类型
typedef void (*ostfil_cb_t)(int status, void *user_data);

// 异步写入
void ostfil_write_async(const char *file_dist,
                        const char *file_path,
                        char *file_name,
                        int *slice_sum,
                        ostfil_cb_t cb,
                        void *user_data);

// 异步读取
void ostfil_read_async(const char *file_path,
                       const char *file_name,
                       const char *file_dist,
                       ostfil_cb_t cb,
                       void *user_data);

// 异步删除
void ostfil_del_async(const char *file_path,
                      const char *file_name,
                      ostfil_cb_t cb,
                      void *user_data);

#ifdef __cplusplus
}
#endif

#endif // OSTFIL_H