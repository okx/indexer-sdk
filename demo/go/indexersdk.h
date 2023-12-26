

#ifndef LIB_H
#define LIB_H

#include <stddef.h>  // 包含 size_t 的声明

// 定义 ByteArray 结构体
struct ByteArray {
    const unsigned char *data;
    size_t length;
};

struct ByteArray get_data(void);
void start_processor();

#endif
