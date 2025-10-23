
#include <stddef.h>
#include <stdio.h>
#include "typescript_compiler.h"

char const Source[] =
    "import {thing, z} from \"test.d.ts\";\n"
    "let x: thing = z;\n"
;
size_t const SourceLen = sizeof(Source) - 1;

int main(int argc, char **argv) {
    char const *module_or_error;
    size_t module_or_error_len = 0;
    if (ts_compile(Source, SourceLen, "test.ts", 7, &module_or_error, &module_or_error_len) != TS_STATUS_OK) {
        printf("Error: %.*s\n", (int) module_or_error_len, module_or_error);
    } else {
        printf("%.*s", (int) module_or_error_len, module_or_error);
    }

    return 0;
}
