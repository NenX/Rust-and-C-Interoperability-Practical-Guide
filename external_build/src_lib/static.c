// 这个文件为静态库源文件，我们将在 Windows 使用 MSBuild 来编译它，在 Linux 使用 gcc 来编译它
// This file is a static library source file and we will compile it using MSBuild on Windows and gcc on Linux
#include <stdio.h>
#include <stdint.h>

int32_t static_call(int32_t a, int32_t b)
{
    int32_t result = a + b;
    printf("[C] static_call: Argument a is:{ %i }, Argument b is:{ %i } \n", a, b);
    printf("[C] static_call: returning the result { %i } to Rust\n", result);
    return result;
}
