@echo off
setlocal EnableDelayedExpansion

tasklist /fi "imagename eq gitbox-service.exe" |find ":" > nul
if errorlevel 1 taskkill /f /im "gitbox-service.exe"
