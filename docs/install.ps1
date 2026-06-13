# grit-simple installer for Windows - downloads a pre-built gs.exe from GitHub Releases
# Usage: irm grit-scm.com/install.ps1 | iex
#
# Override the install location with $env:GRIT_INSTALL_DIR before running.

$ErrorActionPreference = 'Stop'

$Repo = 'gitbutlerapp/grit'
$InstallDir = if ($env:GRIT_INSTALL_DIR) { $env:GRIT_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'grit\bin' }

# Windows ARM64 runs x86_64 binaries via emulation, so x86_64 is the only target we ship.
$Target = 'x86_64-pc-windows-msvc'
$Url = "https://github.com/$Repo/releases/latest/download/gs-$Target.zip"

Write-Host "Downloading from: $Url"

$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("grit-" + [System.IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
  $zip = Join-Path $tmp 'gs.zip'
  Invoke-WebRequest -Uri $Url -OutFile $zip
  Expand-Archive -Path $zip -DestinationPath $tmp -Force

  New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
  Copy-Item -Path (Join-Path $tmp 'gs.exe') -Destination (Join-Path $InstallDir 'gs.exe') -Force
} finally {
  Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}

Write-Host "Installed gs to $InstallDir\gs.exe"

# Add the install dir to the user PATH if it isn't there already.
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($userPath -split ';') -notcontains $InstallDir) {
  $newPath = if ([string]::IsNullOrEmpty($userPath)) { $InstallDir } else { "$userPath;$InstallDir" }
  [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
  $env:Path = "$env:Path;$InstallDir"
  Write-Host "Added $InstallDir to your user PATH (restart open terminals to pick it up)."
}

& "$InstallDir\gs.exe" --version
