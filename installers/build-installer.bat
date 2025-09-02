@echo off
echo Building Chronos Installer...
echo.

REM Build Rust executable first
echo [1/3] Building Rust executable...
cd ..\rust-client
cargo build --release
if errorlevel 1 (
    echo Error: Failed to build Rust executable
    pause
    exit /b 1
)

REM Go back to installer directory
cd ..\installers

REM Check if Inno Setup is installed
echo [2/3] Checking Inno Setup...
set "INNO_PATH=C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
if not exist "%INNO_PATH%" (
    echo Error: Inno Setup 6 not found!
    echo Please download and install Inno Setup from: https://jrsoftware.org/isinfo.php
    echo Install it to the default location: C:\Program Files ^(x86^)\Inno Setup 6\
    pause
    exit /b 1
)

REM Build installer
echo [3/3] Building installer...
"%INNO_PATH%" chronos-installer.iss
if errorlevel 1 (
    echo Error: Failed to build installer
    pause
    exit /b 1
)

echo.
echo ✅ Installer built successfully!
echo Output: chronos-setup.exe
echo.
pause
