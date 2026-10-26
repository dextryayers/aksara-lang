cmake_minimum_required(VERSION 3.20)

project(aksara_runtime
    VERSION 0.1.0
    DESCRIPTION "Runtime C11 untuk bahasa Aksara"
    LANGUAGES C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Debug CACHE STRING "Build type" FORCE)
endif()

enable_testing()
add_subdirectory(runtime)
