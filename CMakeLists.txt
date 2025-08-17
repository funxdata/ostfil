cmake_minimum_required(VERSION 3.30)
project(ostfil_lib C CXX)

# --------------------------
# C / C++ 标准
# --------------------------
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# --------------------------
# 线程库
# --------------------------
find_package(Threads REQUIRED)

# --------------------------
# libuv (已编译好)
# --------------------------
add_library(uv_a STATIC IMPORTED)
set_target_properties(uv_a PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/crates/libuv/build/libuv.a"
    INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_CURRENT_SOURCE_DIR}/crates/libuv/include"
)

# Windows 系统依赖库
if(WIN32)
    set_target_properties(uv_a PROPERTIES
        INTERFACE_LINK_LIBRARIES "ws2_32;iphlpapi;psapi;dbghelp;userenv"
    )
endif()

# --------------------------
# zlib (已编译好)
# --------------------------
add_library(zlib_a STATIC IMPORTED)
set_target_properties(zlib_a PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/crates/zlib/build/libzs.a"
    INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_CURRENT_SOURCE_DIR}/crates/zlib"
)

if(WIN32)
    set_target_properties(zlib_a PROPERTIES
        INTERFACE_LINK_LIBRARIES "ws2_32"
    )
endif()

# --------------------------
# uuid4 静态库
# --------------------------
add_library(uuid4_a STATIC crates/uuid4/src/uuid4.c)
target_include_directories(uuid4_a PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/crates/uuid4/include)

# --------------------------
# ostfil_lib 静态库
# --------------------------
set(OSTFIL_SRC
    src/ostfil_async.c
    src/compression.c
    src/file_ops.c
    src/uuid_util.c
)

add_library(ostfil_lib STATIC ${OSTFIL_SRC})
target_include_directories(ostfil_lib PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(ostfil_lib PUBLIC uv_a zlib_a uuid4_a Threads::Threads)

# 如果有 Windows 特定库
if(WIN32)
    target_link_libraries(ostfil_lib PUBLIC ws2_32 psapi iphlpapi dbghelp userenv)
endif()

# --------------------------
# 示例程序
# --------------------------
add_executable(ostfil_demo src/main.c)
target_link_libraries(ostfil_demo PRIVATE ostfil_lib Threads::Threads)

if(WIN32)
    target_link_libraries(ostfil_demo PRIVATE ws2_32 psapi iphlpapi dbghelp userenv)
endif()

# --------------------------
# 测试
# --------------------------
enable_testing()
add_subdirectory(crates/googletest)

file(GLOB TEST_FILES "${CMAKE_CURRENT_SOURCE_DIR}/tests/*.cpp")
if(NOT TEST_FILES)
    message(FATAL_ERROR "No test files found in ${CMAKE_CURRENT_SOURCE_DIR}/tests")
endif()

add_executable(run_tests ${TEST_FILES})
target_include_directories(run_tests PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(run_tests PRIVATE
    ostfil_lib
    gtest
    gtest_main
    Threads::Threads
)

if(WIN32)
    target_link_libraries(run_tests PRIVATE ws2_32 psapi iphlpapi dbghelp userenv)
endif()

include(GoogleTest)
gtest_discover_tests(run_tests)
