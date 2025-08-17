#ifndef COMPRESSION_H
#define COMPRESSION_H

#include <stdlib.h>

#ifdef __cplusplus
extern "C" {
#endif

// 压缩数据
// in: 输入数据指针
// in_len: 输入数据长度
// out: 输出缓冲区指针（函数内部 malloc）
// out_len: 输出数据长度
// 返回 0 成功，-1 内存分配失败，-2 压缩失败
int compress_buffer(const unsigned char *in, size_t in_len, unsigned char **out, size_t *out_len);

// 解压数据
// in: 输入数据指针
// in_len: 输入数据长度
// out: 输出缓冲区指针（函数内部 malloc）
// out_len: 输出缓冲区大小（预分配长度）
// 返回 0 成功，-1 内存分配失败，-2 解压失败
int decompress_buffer(const unsigned char *in, size_t in_len, unsigned char **out, size_t out_len);

#ifdef __cplusplus
}
#endif

#endif // COMPRESSION_H