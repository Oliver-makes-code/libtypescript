cmake_minimum_required(VERSION 3.15)

set(TYPESCRIPT_RUST_OUTDIR "${CMAKE_BINARY_DIR}/typescript")

set(_TYPESCRIPT_STATIC_LIBNAME "libtypescript.a")
set(_TYPESCRIPT_DYNAMIC_LIBNAME "libtypescript.so")

if(WIN32)
    set(_TYPESCRIPT_STATIC_LIBNAME "typescript.lib")
    set(_TYPESCRIPT_DYNAMIC_LIBNAME "typescript.dll")
endif()

set(TYPESCRIPT_INCLUDE "${CMAKE_CURRENT_SOURCE_DIR}/include")

set(TYPESCRIPT_STATIC_LIB "${TYPESCRIPT_RUST_OUTDIR}/release/${_TYPESCRIPT_STATIC_LIBNAME}")
set(TYPESCRIPT_DYNAMIC_LIB "${TYPESCRIPT_RUST_OUTDIR}/release/${_TYPESCRIPT_DYNAMIC_LIBNAME}")

file(GLOB_RECURSE TYPESCRIPT_RUST_SOURCES
    "${CMAKE_CURRENT_SOURCE_DIR}/src/lib.rs"
    "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml"
)

add_custom_command(
    USES_TERMINAL
    OUTPUT ${TYPESCRIPT_STATIC_LIB} ${TYPESCRIPT_DYNAMIC_LIB}
    DEPENDS ${TYPESCRIPT_RUST_SOURCES}
    COMMAND ${CMAKE_COMMAND} -E env CC= CXX= cargo build
        --release
        --target-dir ${TYPESCRIPT_RUST_OUTDIR}
        --manifest-path "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building libtypescript"
    VERBATIM
)

add_custom_target(typescript_build ALL
    DEPENDS ${TYPESCRIPT_STATIC_LIB} ${TYPESCRIPT_DYNAMIC_LIB}
)

add_library(typescript SHARED IMPORTED GLOBAL)

set_target_properties(typescript PROPERTIES
    IMPORTED_LOCATION "${TYPESCRIPT_DYNAMIC_LIB}"
    INTERFACE_INCLUDE_DIRECTORIES "${TYPESCRIPT_INCLUDE}"
)

if (CMAKE_SYSTEM_NAME STREQUAL "Windows")
    set_target_properties(typescript PROPERTIES
        IMPORTED_IMPLIB "${TYPESCRIPT_DYNAMIC_LIB}.lib"
    )
endif()

add_executable(typescript_test test/main.c)
target_link_libraries(typescript_test typescript)
