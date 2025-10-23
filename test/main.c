
#include <stddef.h>
#include <stdio.h>
#include "typescript_compiler.h"

char const Source[] =
    "import {thing, z} from \"test.d.ts\";\n"
    "let x: thing = z;\n"
;
size_t const SourceLen = sizeof(Source) - 1;

int main(int argc, char **argv) {
    char *module_or_error;
    if (!ts_compile(Source, "test.ts", &module_or_error)) {
        printf("Error: %s\n", module_or_error);
    } else {
        printf("%s", module_or_error);
    }

    return 0;
}
