@echo off
title PDF Toolkit (Rust Native)
cd /d "%~dp0"
if exist "target\release\pdf-toolkit-rust.exe" (
    start "" "target\release\pdf-toolkit-rust.exe"
) else (
    echo Baue und starte Rust PDF Toolkit...
    cargo run --release
)
