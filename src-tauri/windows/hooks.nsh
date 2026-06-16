; Post-install: Set execution policy, add to PATH, inject shell hooks
!macro NSIS_HOOK_POSTINSTALL
  ; Set PowerShell execution policy to allow profile/hook scripts
  nsExec::ExecToLog 'powershell -ExecutionPolicy Bypass -Command "Set-ExecutionPolicy RemoteSigned -Scope CurrentUser -Force"'

  ; Add $INSTDIR to user PATH via PowerShell (avoids NSIS string length limits)
  nsExec::ExecToLog 'powershell -ExecutionPolicy Bypass -Command "\
    $instDir = \"$INSTDIR\"; \
    $currentPath = [Environment]::GetEnvironmentVariable(\"Path\", \"User\"); \
    if ($currentPath -notlike \"*$instDir*\") { \
      [Environment]::SetEnvironmentVariable(\"Path\", \"$currentPath;$instDir\", \"User\"); \
    }"'

  ; Run envtools init to inject shell hooks
  nsExec::ExecToLog '"$INSTDIR\envtools.exe" init'
!macroend

; Post-uninstall: Remove installation directory from user PATH
!macro NSIS_HOOK_POSTUNINSTALL
  nsExec::ExecToLog 'powershell -ExecutionPolicy Bypass -Command "\
    $instDir = \"$INSTDIR\"; \
    $currentPath = [Environment]::GetEnvironmentVariable(\"Path\", \"User\"); \
    $newPath = ($currentPath -split \";\" | Where-Object { $_ -ne $instDir }) -join \";\"; \
    [Environment]::SetEnvironmentVariable(\"Path\", $newPath, \"User\")"'
!macroend
