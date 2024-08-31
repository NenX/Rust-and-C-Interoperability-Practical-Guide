#include <stdio.h>
#include <stdint.h>

int32_t add(int32_t a, int32_t b)
{
    int32_t result = a + b;
    printf("[C] source: Argument a is:{ %i }, Argument b is:{ %i } \n", a, b);
    printf("[C] source: returning the result { %i } to Rust\n", result);
    return result;
}
