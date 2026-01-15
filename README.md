# AstraStack

Proyecto híbrido en Rust + C creado para Astra. Combina un núcleo de transformaciones de datos de alto rendimiento en Rust con enlaces FFI simples para C, más una CLI lista para producción.

## Características
- Núcleo Rust rápido con perfiles de transformación (recorte, mayúsculas, eliminación de vacíos, deduplicación).
- CLI `astra-stack-cli` para procesar archivos/streams.
- Enlaces C listos: estructura de sesión con estado y ejemplo compilable.
- Artefactos `staticlib` y `cdylib` para enlazar en Windows.
- Pruebas unitarias y flujo de CI base.

## Estructura
- `src/lib.rs`: lógica central y API pública/FFI.
- `src/main.rs`: CLI.
- `c/`: cabecera, ejemplo en C y script de build.
- `tests/`: pruebas de transformación.
- `.github/workflows/ci.yml`: workflow base.

## Requisitos previos
- Rust estable (`rustup`): https://rustup.rs
- Compilador C (Windows: `cl` de Visual Studio Build Tools o `clang` de LLVM).

## Uso rápido (Rust)
```bash
# Construir
cargo build --release

# Ejecutar CLI con transformaciones
cargo run --release -- \
  --trim --upper --drop-empty --dedup \
  --input samples/input.txt --output dist/output.txt
```

## Enlaces C
1) Construye la librería Rust (genera `astra_stack.lib` y `astra_stack.dll`):
```bash
cargo build --release
```

2) Compila el ejemplo en C (usa `cl`, ajusta rutas si cambias carpeta):
```bash
# Desde la raíz del repo
cl /I c c\astra_example.c target\release\astra_stack.lib /Fe:c\astra_example.exe
```

Con `clang` sería algo así:
```bash
clang -I c c/astra_example.c target/release/astra_stack.lib -o c/astra_example.exe
```

3) Ejecuta el ejemplo (coloca `astra_stack.dll` en el mismo directorio o en tu PATH):
```bash
c\astra_example.exe
```

## Notas de diseño
- El estado de deduplicación vive en la sesión FFI (`AstraSession`), seguro para un único hilo.
- Se expone un buffer gestionado por Rust; debes liberarlo con `astra_buffer_free` desde C.

## Próximos pasos sugeridos
- Añadir perfiles configurables desde JSON en la CLI (`--profile profile.json`).
- Extender el pipeline con normalización Unicode y métricas.
- Empaquetar una DLL firmada para distribución.
