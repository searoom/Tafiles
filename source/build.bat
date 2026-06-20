@echo off
cd /d "%~dp0"
call npm run tauri build
if exist "src-tauri\target\release\tafiles.exe" (
    copy /y "src-tauri\target\release\tafiles.exe" "..\tafiles.exe"
    echo OK: tafiles.exe copied to root
) else (
    echo ERROR: tafiles.exe not found
    exit /b 1
)
