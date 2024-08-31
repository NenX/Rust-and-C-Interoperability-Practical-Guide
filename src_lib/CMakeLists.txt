# 我们使用 CMake 来简化对动态库和静态库的编译工作
# We use CMake to simplify the compilation of dynamic and static libraries
cmake_minimum_required(VERSION 3.10)
project(static_for_rust C)
project(dylib_for_rust C)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_HOME_DIRECTORY}/lib_build)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_HOME_DIRECTORY}/lib_build)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_HOME_DIRECTORY}/lib_build)
set(CMAKE_C_STANDARD 99)

add_library(static_for_rust static.c)
add_library(dylib_for_rust SHARED dylib.c)


# ==============