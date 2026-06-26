@echo off
cd /d "%~dp0"

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0prebuild.ps1"

cd /d "%~dp0source"
call npm run tauri build
if %errorlevel% neq 0 (
    echo Build failed!
    pause
    exit /b %errorlevel%
)
copy /y "src-tauri\target\release\tafiles.exe" "%~dp0tafiles.exe"
echo Build completed. Copied tafiles.exe to root directory.
