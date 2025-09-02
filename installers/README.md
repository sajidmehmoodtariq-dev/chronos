# Chronos Installer

This directory contains the installer setup for Chronos Activity Tracker.

## Prerequisites

1. **Inno Setup 6** - Download from [https://jrsoftware.org/isinfo.php](https://jrsoftware.org/isinfo.php)
   - Install to default location: `C:\Program Files (x86)\Inno Setup 6\`

## Building the Installer

1. Make sure the Rust project is built:

   ```bash
   cd ../rust-client
   cargo build --release
   ```

2. Run the build script:

   ```bash
   build-installer.bat
   ```

3. The installer `chronos-setup.exe` will be generated in this directory.

## What the Installer Does

- Installs Chronos to `Program Files\Chronos\`
- Creates start menu shortcuts
- Optionally adds to Windows startup
- Optionally creates desktop shortcut
- Includes uninstaller

## Distribution

1. Upload `chronos-setup.exe` to GitHub Releases
2. Link from the website download page
3. Users can download and install with one click

## Manual Installation (Alternative)

If you don't want to use the installer:

1. Copy `chronos.exe` to desired location
2. Run it manually or add to Windows startup
3. Get sync token from web dashboard
4. Enter token when prompted
