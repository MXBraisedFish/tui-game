@echo off
setlocal
"%~dp0..\..\version.exe" %*
exit /b %errorlevel%
