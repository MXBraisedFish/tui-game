# Inspect PE imports without an SDK, compiler or third-party PowerShell module.
[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$Executable)

$ErrorActionPreference = 'Stop'
$binaryPath = (Resolve-Path -LiteralPath $Executable).ProviderPath
$bytes = [IO.File]::ReadAllBytes($binaryPath)

function Read-U16([int]$Offset) { return [BitConverter]::ToUInt16($bytes, $Offset) }
function Read-U32([int]$Offset) { return [BitConverter]::ToUInt32($bytes, $Offset) }

if ($bytes.Length -lt 64 -or (Read-U16 0) -ne 0x5a4d) { throw 'Not a DOS/PE executable.' }
$peOffset = Read-U32 0x3c
if ((Read-U32 $peOffset) -ne 0x4550) { throw 'Missing PE signature.' }
$sectionCount = Read-U16 ($peOffset + 6)
$optionalSize = Read-U16 ($peOffset + 20)
$optionalOffset = $peOffset + 24
$magic = Read-U16 $optionalOffset
$directoryOffset = switch ($magic) {
  0x20b { $optionalOffset + 112 }
  0x10b { $optionalOffset + 96 }
  default { throw 'Unsupported PE optional header.' }
}
$headerSize = Read-U32 ($optionalOffset + 60)
$sections = @()
for ($index = 0; $index -lt $sectionCount; $index++) {
  $offset = $optionalOffset + $optionalSize + 40 * $index
  $sections += @{
    Address = Read-U32 ($offset + 12)
    Size = [Math]::Max((Read-U32 ($offset + 8)), (Read-U32 ($offset + 16)))
    RawSize = Read-U32 ($offset + 16)
    RawOffset = Read-U32 ($offset + 20)
  }
}

function Convert-Rva([uint32]$Address) {
  if ($Address -lt $headerSize) { return [int]$Address }
  foreach ($section in $sections) {
    if ($Address -ge $section.Address -and $Address -lt ($section.Address + $section.Size)) {
      $relative = $Address - $section.Address
      if ($relative -ge $section.RawSize) { throw 'RVA has no file data.' }
      return [int]($section.RawOffset + $relative)
    }
  }
  throw "Unmapped PE RVA: $Address"
}

function Read-ImportName([uint32]$Address) {
  $offset = Convert-Rva $Address
  $end = $offset
  while ($end -lt $bytes.Length -and $bytes[$end] -ne 0) { $end++ }
  if ($end -eq $bytes.Length) { throw 'Unterminated import name.' }
  return [Text.Encoding]::ASCII.GetString($bytes, $offset, $end - $offset).ToLowerInvariant()
}

$imports = @()
# Inspect both ordinary imports and RVA-based delay imports.
foreach ($table in @(@{Index = 1; Stride = 20; Name = 12}, @{Index = 13; Stride = 32; Name = 4})) {
  $address = Read-U32 ($directoryOffset + 8 * $table.Index)
  $size = Read-U32 ($directoryOffset + 8 * $table.Index + 4)
  if ($address -eq 0) { continue }
  $start = Convert-Rva $address
  for ($offset = $start; $offset + $table.Stride -le $start + $size; $offset += $table.Stride) {
    $nameAddress = Read-U32 ($offset + $table.Name)
    if ($nameAddress -eq 0) { break }
    if ($table.Index -eq 13 -and (Read-U32 $offset) -ne 1) { throw 'Unsupported VA-based delay imports.' }
    $imports += Read-ImportName $nameAddress
  }
}
if ($imports.Count -eq 0) { throw 'No imports found; review the PE file manually.' }
$imports = @($imports | Sort-Object -Unique)
$systemLibraries = @(
  'advapi32.dll', 'avrt.dll', 'bcrypt.dll', 'bcryptprimitives.dll', 'cfgmgr32.dll',
  'combase.dll', 'comctl32.dll', 'comdlg32.dll', 'crypt32.dll', 'd3d11.dll',
  'dwmapi.dll', 'dxgi.dll', 'gdi32.dll', 'imm32.dll', 'iphlpapi.dll', 'kernel32.dll',
  'msvcrt.dll', 'netapi32.dll', 'ntdll.dll', 'ole32.dll', 'oleaut32.dll', 'pdh.dll',
  'powrprof.dll', 'propsys.dll', 'psapi.dll', 'rpcrt4.dll', 'secur32.dll',
  'setupapi.dll', 'shell32.dll', 'shlwapi.dll', 'ucrtbase.dll', 'user32.dll',
  'userenv.dll', 'uxtheme.dll', 'version.dll', 'winhttp.dll', 'winmm.dll', 'ws2_32.dll'
)
$external = @($imports | Where-Object {
  $_ -notin $systemLibraries -and $_ -notmatch '^(api-ms-win-|ext-ms-win-).+\.dll$'
})
if ($external.Count -gt 0) {
  throw "Non-system DLL imports require review: $($external -join ', ')"
}
Write-Output "PASS: $binaryPath ($($imports.Count) Windows system imports; no external runtime DLL imports)"
