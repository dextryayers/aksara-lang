#include "aksara.h"

#include <stdlib.h>
#include <string.h>

struct AksaraRuntime {
    uint32_t magic;
    int siap;
};

#define AKSARA_RUNTIME_MAGIC 0x414B5341u /* "AKSA" */

const char *aksara_versi(void) {
    return AKSARA_VERSION_STRING;
}

uint32_t aksara_versi_bytecode(void) {
    return 1u;
}

AksaraRuntime *aksara_runtime_baru(void) {
    AksaraRuntime *runtime = (AksaraRuntime *)calloc(1, sizeof(AksaraRuntime));
    if (runtime == NULL) {
        return NULL;
    }
    runtime->magic = AKSARA_RUNTIME_MAGIC;
    runtime->siap = 1;
    return runtime;
}

void aksara_runtime_hapus(AksaraRuntime *runtime) {
    if (runtime == NULL) {
        return;
    }
    runtime->magic = 0;
    runtime->siap = 0;
    free(runtime);
}

int aksara_runtime_siap(const AksaraRuntime *runtime) {
    if (runtime == NULL) {
        return 0;
    }
    return runtime->magic == AKSARA_RUNTIME_MAGIC && runtime->siap != 0;
}
