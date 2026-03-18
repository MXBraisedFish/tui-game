$ErrorActionPreference = 'Stop'

$toolsDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$packageArgs = @{
  packageName   = 'tui-game'
  unzipLocation = $toolsDir
  url           = 'https://github.com/MXBraisedFish/TUI-GAME/releases/download/v__VERSION__/tui-game-__VERSION__-windows.zip'
  checksum      = '__SHA256__'
  checksumType  = 'sha256'
}

Install-ChocolateyZipPackage @packageArgs
Install-BinFile -Name 'tg' -Path (Join-Path $toolsDir 'tg.bat')
