add_library(aksara_runtime STATIC
    src/objek.c
)

target_include_directories(aksara_runtime PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

target_compile_options(aksara_runtime PRIVATE
    $<$<C_COMPILER_ID:GNU,Clang>:-Wall -Wextra -Wpedantic -Werror>
    $<$<C_COMPILER_ID:MSVC>:/W4 /WX>
)

add_executable(uji_runtime uji/uji_runtime.c)
target_link_libraries(uji_runtime PRIVATE aksara_runtime)

add_test(NAME uji_runtime COMMAND uji_runtime)
