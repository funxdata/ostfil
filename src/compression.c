#include "../include/ostfil.h"
#include <stdlib.h>

// 压缩数据
int compress_buffer(const unsigned char *in, size_t in_len, unsigned char **out, size_t *out_len) {
    uLongf buf_len = compressBound(in_len);
    *out = malloc(buf_len);
    if (!*out) return -1;

    if (compress2(*out, &buf_len, in, in_len, Z_BEST_COMPRESSION) != Z_OK) {
        free(*out);
        return -2;
    }

    *out_len = buf_len;
    return 0;
}

// 解压数据
int decompress_buffer(const unsigned char *in, size_t in_len, unsigned char **out, size_t out_len) {
    *out = malloc(out_len);
    if (!*out) return -1;

    uLongf dest_len = out_len;
    if (uncompress(*out, &dest_len, in, in_len) != Z_OK) {
        free(*out);
        return -2;
    }
    return 0;
}