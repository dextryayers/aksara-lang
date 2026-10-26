#include "aksara.h"

#include <stdio.h>
#include <string.h>

static int uji_versi(void) {
    if (strcmp(aksara_versi(), AKSARA_VERSION_STRING) != 0) {
        fprintf(stderr, "versi runtime tidak cocok\n");
        return 1;
    }
    if (aksara_versi_bytecode() != 1u) {
        fprintf(stderr, "versi bytecode tidak cocok\n");
        return 1;
    }
    return 0;
}

static int uji_siklus_hidup(void) {
    AksaraRuntime *runtime = aksara_runtime_baru();
    if (runtime == NULL) {
        fprintf(stderr, "runtime gagal dibuat\n");
        return 1;
    }
    if (!aksara_runtime_siap(runtime)) {
        fprintf(stderr, "runtime tidak siap\n");
        aksara_runtime_hapus(runtime);
        return 1;
    }
    aksara_runtime_hapus(runtime);
    runtime = NULL;
    if (aksara_runtime_siap(runtime)) {
        fprintf(stderr, "runtime NULL dianggap siap\n");
        return 1;
    }
    return 0;
}

int main(void) {
    if (uji_versi() != 0) {
        return 1;
    }
    if (uji_siklus_hidup() != 0) {
        return 1;
    }
    printf("uji runtime C: lulus\n");
    return 0;
}
