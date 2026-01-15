#pragma once

#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    bool trim;
    bool to_upper;
    bool drop_empty;
    bool deduplicate;
} AstraProfile;

typedef struct {
    char* ptr;
    size_t len;
    size_t capacity;
} AstraBuffer;

typedef struct AstraSession AstraSession;

// Perfil por defecto (recorta y elimina vacios).
AstraProfile astra_profile_default(void);

// Crear y liberar una sesion de transformacion.
AstraSession* astra_session_new(AstraProfile profile);
void astra_session_free(AstraSession* session);

// Transforma una cadena UTF-8. Devuelve un buffer que debe liberarse con astra_buffer_free.
AstraBuffer astra_session_transform(AstraSession* session, const char* data, size_t len);

// Liberar la memoria del buffer devuelto por Rust.
void astra_buffer_free(AstraBuffer buffer);

#ifdef __cplusplus
}
#endif
