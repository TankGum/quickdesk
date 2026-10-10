# Install or update QuickDesk (https://quickdesk.click) on Windows, from PowerShell:
#
#   irm https://quickdesk.click/install.ps1 | iex
#
# Takes the newest release from latest.json, checks the installer against its
# SHA-256 and runs it silently for the current user (no administrator rights).
# QUICKDESK_DRY_RUN=1: download and check, but install nothing.

& {
  $ErrorActionPreference = 'Stop'
  $ProgressPreference = 'SilentlyContinue' # Invoke-WebRequest is far faster without the bar
  $dl = 'https://dl.quickdesk.click'

  if (-not [Environment]::Is64BitOperatingSystem) { throw 'QuickDesk needs 64-bit Windows.' }

  $latest = Invoke-RestMethod "$dl/latest.json"
  $file = $latest.files.windows
  if (-not $file) { throw "QuickDesk $($latest.version) has no Windows installer yet." }

  Write-Host "==> QuickDesk $($latest.version) for Windows"
  $tmp = Join-Path ([IO.Path]::GetTempPath()) $file.name
  Write-Host "==> Downloading $($file.name)"
  Invoke-WebRequest "$dl/$($file.url)" -OutFile $tmp -UseBasicParsing

  $got = (Get-FileHash $tmp -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($got -ne $file.sha256) {
    Remove-Item $tmp -Force
    throw "Checksum does not match for $($file.name) (expected $($file.sha256), got $got)."
  }
  Write-Host '==> Checksum OK'

  if ($env:QUICKDESK_DRY_RUN) {
    Write-Host "==> Dry run: would install $tmp"
    return
  }
  # /S: NSIS silent install, for the current user as configured.
  Start-Process -FilePath $tmp -ArgumentList '/S' -Wait
  Remove-Item $tmp -Force
  Write-Host "==> QuickDesk $($latest.version) is installed. Start it from the Start menu; it lives in the system tray and updates itself from now on."
}
