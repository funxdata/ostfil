#include "../include/ostfil.h"
#include <string.h>

// 生成 UUID 字符串，长度 36
void generate_uuid_str(char *out_uuid) {
    uuid4_generate(out_uuid);
}