@echo off
setlocal

rem Debug run with DevTools (CDP) exposed on http://127.0.0.1:9222.
call "%~dp0dev.cmd" --cdp
