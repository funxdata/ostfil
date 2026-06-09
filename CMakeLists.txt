cmake_minimum_required(VERSION 3.20)

project(ostfil C)

set(CMAKE_C_STANDARD 11)

add_library(ostfil SHARED
    src/ostfil.c
)

target_include_directories(
    ostfil
    PUBLIC
    ${PROJECT_SOURCE_DIR}/include
)

target_compile_definitions(
    ostfil
    PRIVATE OSTFIL_BUILD
)

