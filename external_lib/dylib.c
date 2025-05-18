// 这个文件为动态库源文件，我们将在 Windows 使用 MSBuild 来编译它，在 Linux 使用 gcc 来编译它
// This file is a dynamic library source file and we will compile it using MSBuild on Windows and gcc on Linux
#include <stdio.h>
#include <stdint.h>
#ifdef _WIN32
__declspec(dllexport)
#endif
int32_t dyloading_add(int32_t a, int32_t b, char *result)
{
    printf("[External dyloading] Hello %s\n", result);

    int32_t sum = a + b;

    sprintf(result,"[External dyloading] The result (%d + %d) is %d!", a, b, sum);
    return sum;
}


