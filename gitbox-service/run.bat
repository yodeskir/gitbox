@echo off
setlocal EnableDelayedExpansion

tasklist /fi "imagename eq gitbox-service.exe" |find ":" > nul
if errorlevel 1 taskkill /f /im "gitbox-service.exe"
timeout 1 > NUL

SET  cfg_dir="%userprofile%\AppData\Roaming\gitbox-service\config"
echo %APPDATA%
if exist "%cfg_dir%\gitbox-service.toml" (
    echo "Service is configured"
		set /p reconf=Do you want to reconfigure? [Y/N]:
		if "!reconf!"=="Y" (
			set /p local=Enter local repo directory:
			set /p url=Enter github repo url:
			set /p branch=Enter branch name [master/main?]:
			set /p user=Enter github user name:
			set /p password=Enter github password:
			
			REM Creating config file
			
			echo(repourl = "!url!" > %cfg_dir%\gitbox-service.toml
			echo(branch = "!branch!" >> %cfg_dir%\gitbox-service.toml
			echo(localwatch = "!local!" >> %cfg_dir%\gitbox-service.toml
			echo(username = "!user!" >> %cfg_dir%\gitbox-service.toml
			echo(password = "!password!" >> %cfg_dir%\gitbox-service.toml			
			echo(cloned = false >> %cfg_dir%\gitbox-service.toml
		)
) else (
    echo "Config does not exists. Lets configure gitbox-service!"
		set /p local=Enter local repo directory:
		set /p url=Enter github repo url:
		set /p branch=Enter branch name [master/main?]:
		set /p user=Enter github user name:
		set /p password=Enter github password:
		
		REM Creating config file
		
		echo(repourl = "!url!" > %cfg_dir%\gitbox-service.toml
		echo(branch = "!branch!" >> %cfg_dir%\gitbox-service.toml
		echo(localwatch = "!local!" >> %cfg_dir%\gitbox-service.toml
		echo(username = "!user!" >> %cfg_dir%\gitbox-service.toml
		echo(password = "!password!" >> %cfg_dir%\gitbox-service.toml
		echo(cloned = false >> %cfg_dir%\gitbox-service.toml
)

Echo Set WshShell = CreateObject("WScript.Shell")   >>%temp%\ghost.vbs
Echo WshShell.Run chr(34) ^& "gitbox-service.exe" ^& Chr(34), 0 >>%temp%\ghost.vbs
Echo Set WshShell = Nothing                         >>%temp%\ghost.vbs
start %temp%\ghost.vbs
timeout /t 1 >nul
del %temp%\ghost.vbs