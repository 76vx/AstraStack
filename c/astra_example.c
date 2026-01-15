#include <stdio.h>
#include <string.h>

#include "astra.h"

int main(void) {
    AstraProfile profile = astra_profile_default();
    profile.to_upper = true;
    profile.deduplicate = true;

    AstraSession* session = astra_session_new(profile);
    if (!session) {
        fprintf(stderr, "No se pudo crear la sesion.\n");
        return 1;
    }

    const char* line1 = "  hola mundo  ";
    AstraBuffer out1 = astra_session_transform(session, line1, strlen(line1));
    if (out1.ptr) {
        printf("1) %.*s\n", (int)out1.len, out1.ptr);
    }
    astra_buffer_free(out1);

    // Duplicado; debe omitirse por deduplicacion.
    const char* line2 = "hola mundo";
    AstraBuffer out2 = astra_session_transform(session, line2, strlen(line2));
    if (out2.ptr && out2.len > 0) {
        printf("2) %.*s\n", (int)out2.len, out2.ptr);
    } else {
        printf("2) omitido (duplicado)\n");
    }
    astra_buffer_free(out2);

    // Nueva linea unica.
    const char* line3 = "rust y c";
    AstraBuffer out3 = astra_session_transform(session, line3, strlen(line3));
    if (out3.ptr) {
        printf("3) %.*s\n", (int)out3.len, out3.ptr);
    }
    astra_buffer_free(out3);

    astra_session_free(session);
    return 0;
}
