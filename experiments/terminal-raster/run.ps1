param(
    [Parameter(Mandatory = $true)][string]$OutputDirectory,
    [string]$TerminalProfile = '{0caa0dad-35be-5f56-a8ff-afceeeaa6101}',
    [string]$FontProfile = '',
    [string]$DevWorktree = ''
)
$ErrorActionPreference = 'Stop'
$workspace = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$output = [IO.Path]::GetFullPath($OutputDirectory)
if (-not $FontProfile) { $FontProfile = Join-Path $workspace 'profiles/current.json' }
$FontProfile = (Resolve-Path -LiteralPath $FontProfile).Path
Get-Command wt.exe -ErrorAction Stop | Out-Null
Push-Location $workspace
try {
    cargo build --workspace --locked
    if ($LASTEXITCODE -ne 0) { throw 'Workspace build failed' }
    $auditArgs = @()
    if ($DevWorktree) { $auditArgs = @('--dev', $DevWorktree) }
    & './target/debug/xtask.exe' @auditArgs
    if ($LASTEXITCODE -ne 0) { throw 'Dependency/compiler parity check failed' }
    New-Item -ItemType Directory -Force -Path $output | Out-Null
    foreach ($name in @('terminal-raster', 'image-compare')) {
        Copy-Item -LiteralPath "target/debug/$name.exe" -Destination (Join-Path $output "$name.exe") -Force
    }
    Copy-Item -LiteralPath $FontProfile -Destination (Join-Path $output 'input-profile.json') -Force
    $raster = Join-Path $output 'terminal-raster.exe'
    $compare = Join-Path $output 'image-compare.exe'
    $title = 'TG-B7-LAB-' + [Guid]::NewGuid().ToString('N')
    $ready = Join-Path $output "$title-ready.json"
    # This is the interactive test window the user compares against, not a background helper.
    # Quote every path passed to Start-Process, whose ArgumentList is joined into one command line.
    Start-Process -FilePath 'wt.exe' -ArgumentList @('-w', 'new', 'new-tab', '--profile', "`"$TerminalProfile`"",
        '--title', $title, '--suppressApplicationTitle', "`"$raster`"", 'emit', $title, '120', "`"$ready`"")
    try {
        $deadline = [DateTime]::UtcNow.AddSeconds(20)
        while (-not (Test-Path -LiteralPath $ready)) {
            if ([DateTime]::UtcNow -gt $deadline) { throw 'Terminal not ready: inspect its error and ensure at least 80 x 20 cells' }
            Start-Sleep -Milliseconds 100
        }
        Start-Sleep -Milliseconds 1000
        & $raster capture $title (Join-Path $output 'window.png')
        if ($LASTEXITCODE -ne 0) { throw 'Target-window capture failed' }
    } finally {
        & $raster close $title
    }
    & $compare calibrate (Join-Path $output 'window.png') $FontProfile $output
    if ($LASTEXITCODE -ne 0) { throw 'Ruler calibration failed; no guessed crop is used' }
    & $raster render (Join-Path $output 'measured-profile.json') (Join-Path $output 'measured')
    if ($LASTEXITCODE -ne 0) { throw 'Raster experiment failed' }
    foreach ($variant in @('software-natural','software-fit','native-cell-gray','native-cell-cleartype','native-run-gray','native-cluster-gray')) {
        & $compare compare (Join-Path $output 'reference.png') (Join-Path $output "measured/$variant.png") (Join-Path $output "comparison/$variant")
        if ($LASTEXITCODE -ne 0) { throw "Comparison failed: $variant" }
    }
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'report.html') -Destination (Join-Path $output 'index.html') -Force
    Get-ChildItem -LiteralPath $output -Recurse -File | Where-Object { $_.Extension -in '.png','.json','.exe' -and $_.Name -ne 'checksums.json' } |
        Get-FileHash -Algorithm SHA256 | Select-Object Path, Hash |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $output 'checksums.json') -Encoding utf8
    Write-Host "Finished. Open $(Join-Path $output 'index.html')"
} finally {
    Pop-Location
}
