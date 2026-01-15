param(
    [ValidateSet("debug", "release")]
    [string]$Configuration = "release"
)

$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
Push-Location $root
try {
    Write-Host "[AstraStack] Compilando Rust ($Configuration)..."
    cargo build --$Configuration

    $libPath = Join-Path $root "target\$Configuration\astra_stack.lib"
    if (-not (Test-Path $libPath)) {
        Write-Warning "No se encontro $libPath. Ajusta el nombre de la libreria si el toolchain genera otro artefacto."
    }

    Write-Host "[AstraStack] Compilando ejemplo C..."
    cl /I c c\astra_example.c $libPath /Fe:c\astra_example.exe
    Write-Host "Listo. Ejecuta: c\\astra_example.exe"
} finally {
    Pop-Location
}
