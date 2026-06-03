@echo off
REM 注册 KeyVault Native Messaging Host for Chrome/Edge
REM 以管理员权限运行

set HOST_MANIFEST=%~dp0..\extension\com.keyvault.app.json
set EXE_PATH=%~dp0..\src-tauri\target\release\keyvault.exe

REM 更新 manifest 中的 path
powershell -Command "(Get-Content '%HOST_MANIFEST%') -replace '\"path\": \"\"', '\"path\": \"%EXE_PATH:\\=\\\\%\"' | Set-Content '%HOST_MANIFEST%'"

REM 注册到 Chrome
reg add "HKCU\SOFTWARE\Google\Chrome\NativeMessagingHosts\com.keyvault.app" /ve /t REG_SZ /d "%HOST_MANIFEST%" /f

REM 注册到 Edge
reg add "HKCU\SOFTWARE\Microsoft\Edge\NativeMessagingHosts\com.keyvault.app" /ve /t REG_SZ /d "%HOST_MANIFEST%" /f

echo Native Messaging Host 注册完成
pause
