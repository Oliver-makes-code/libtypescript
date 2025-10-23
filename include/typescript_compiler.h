#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>

bool ts_compile(char const *input, char const *filename, char **module_or_error);
void ts_compile_free(char const *module_or_error);

#ifdef __cplusplus
}
#endif
