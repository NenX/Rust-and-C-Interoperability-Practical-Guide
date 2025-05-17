#include <stdio.h>
#include <stdint.h>

int32_t add(int32_t a, int32_t b, char *result)
{
    printf("[C source] Hello %s\n", result);

    int32_t sum = a + b;

    sprintf(result,"[C source] The result (%d + %d) is %d!", a, b, sum);
    return sum;
}
