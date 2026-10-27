@echo off
setlocal

rem Cargo is not on the machine PATH, so add the default rustup location.
if exist "%USERPROFILE%\.cargo\bin" set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

rem Load the MSVC environment so the Rust linker can be found.
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if exist "%VSWHERE%" (
  for /f "usebackq delims=" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "VSDIR=%%i"
)
if defined VSDIR (
  call "%VSDIR%\VC\Auxiliary\Build\vcvars64.bat" >nul
)

rem `dev.cmd --cdp` merges tauri.debug.conf.json so DevTools can connect on
rem port 9222. Plain `dev.cmd` runs the normal debug build without it.
if "%~1"=="--cdp" (
  call npm.cmd run tauri -- dev --config src-tauri/tauri.debug.conf.json
) else (
  call npm.cmd run tauri dev
)
