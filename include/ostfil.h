#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char uuid[37];          // 生成 UUID，例如 "550e8400-e29b-41d4-a716-446655440000"
    char original_path[1024];
    uint64_t offset;
    uint64_t size;
} ostfil;


