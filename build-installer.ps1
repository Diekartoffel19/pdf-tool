# PowerShell Skript zum Erstellen des Release-Builds und Installers

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

Write-Host "===> 1. Baue PDF Werkzeugkasten im Release-Modus (Rust)..." -ForegroundColor Cyan
cargo build --release

$releaseExe = "target\release\pdf-toolkit-rust.exe"
if (-not (Test-Path $releaseExe)) {
    Write-Error "Release Binary wurde nicht gefunden unter: $releaseExe"
}

Write-Host "===> Release Binary erfolgreich gebaut: $releaseExe" -ForegroundColor Green

# Suche ISCC.exe
$isccCandidates = @(
    "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe"
)

$isccPath = $isccCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $isccPath) {
    Write-Warning "Inno Setup Compiler (ISCC.exe) nicht gefunden. Standalone .exe liegt bereit in target/release/."
    exit 0
}

Write-Host "===> 2. Erstelle Windows Installer mit Inno Setup ($isccPath)..." -ForegroundColor Cyan
if (-not (Test-Path "dist")) {
    New-Item -ItemType Directory -Path "dist" | Out-Null
}

& $isccPath "installer.iss"

$installerExe = "dist\PDF_Werkzeugkasten_Setup_v1.0.0.exe"
if (Test-Path $installerExe) {
    $sizeMb = [math]::Round(((Get-Item $installerExe).Length / 1MB), 2)
    Write-Host "`n========================================================" -ForegroundColor Green
    Write-Host "  INSTALLER ERFOLGREICH ERSTELLT!" -ForegroundColor Green
    Write-Host "  Datei: $installerExe ($sizeMb MB)" -ForegroundColor Green
    Write-Host "========================================================`n" -ForegroundColor Green
} else {
    Write-Error "Installer konnte nicht in 'dist/' gefunden werden."
}
