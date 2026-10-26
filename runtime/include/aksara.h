#ifndef AKSARA_H
#define AKSARA_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define AKSARA_VERSION_MAYOR 0
#define AKSARA_VERSION_MINOR 1
#define AKSARA_VERSION_PATCH 0
#define AKSARA_VERSION_STRING "0.1.0"

/*
 * Kode status runtime. Nilai negatif adalah kesalahan, nol adalah sukses.
 */
typedef enum AksaraStatus {
    AKSARA_OK = 0,
    AKSARA_KESALAHAN_ARGUMEN = -1,
    AKSARA_KESALAHAN_MEMORI = -2,
    AKSARA_KESALAHAN_STATUS = -3
} AksaraStatus;

/*
 * Runtime belum memiliki object model pada Fase 0. Struct ini adalah opaque
 * handle agar ABI dapat berevolusi tanpa mengubah pemanggil.
 */
typedef struct AksaraRuntime AksaraRuntime;

/* Mengembalikan versi runtime sebagai string statis. */
const char *aksara_versi(void);

/* Mengembalikan versi bytecode yang didukung. */
uint32_t aksara_versi_bytecode(void);

/* Membuat runtime baru. Mengembalikan NULL jika alokasi gagal. */
AksaraRuntime *aksara_runtime_baru(void);

/* Menghapus runtime. Aman dipanggil dengan NULL. */
void aksara_runtime_hapus(AksaraRuntime *runtime);

/* Memeriksa apakah runtime berada pada status siap. */
int aksara_runtime_siap(const AksaraRuntime *runtime);

#ifdef __cplusplus
}
#endif

#endif /* AKSARA_H */
