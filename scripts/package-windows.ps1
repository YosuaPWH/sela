[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$')]
    [string]$Version
)

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
$binary = Join-Path $projectRoot 'target/x86_64-pc-windows-msvc/release/sela-notch.exe'
$stage = Join-Path $projectRoot 'dist/Sela'
$output = Join-Path $projectRoot 'dist/release'

if (-not (Test-Path $binary)) { throw "Build the Windows release executable first: $binary" }
foreach ($directory in @($stage, $output)) {
    if (Test-Path $directory) { Remove-Item $directory -Recurse -Force }
    New-Item -ItemType Directory -Path $directory -Force | Out-Null
}
Copy-Item $binary, (Join-Path $projectRoot 'README.md'), (Join-Path $projectRoot 'LICENSE'), (Join-Path $projectRoot 'PRODUCT.md'), (Join-Path $projectRoot 'DESIGN.md') $stage
Copy-Item (Join-Path $projectRoot 'docs'), (Join-Path $projectRoot 'assets') $stage -Recurse
Copy-Item $binary (Join-Path $output 'sela-notch.exe')
Compress-Archive -Path "$stage/*" -DestinationPath (Join-Path $output "Sela-$Version-windows-x64.zip")

$compiler = (Get-Command ISCC.exe -ErrorAction SilentlyContinue).Source
if (-not $compiler) {
    $roots = @(${env:ProgramFiles(x86)}, $env:ProgramFiles) | Where-Object { $_ }
    $compiler = $roots | ForEach-Object {
        Get-ChildItem (Join-Path $_ 'Inno Setup */ISCC.exe') -ErrorAction SilentlyContinue
    } | Sort-Object FullName -Descending | Select-Object -First 1 -ExpandProperty FullName
}
if (-not $compiler) { throw 'Inno Setup 6.3 or later is required to build the installer.' }
$fileVersion = ($Version -split '-')[0] + '.0'
& $compiler "/DAppVersion=$Version" "/DFileVersion=$fileVersion" "/DStageDir=$stage" "/DOutputDir=$output" (Join-Path $projectRoot 'packaging/windows/sela.iss')
if ($LASTEXITCODE -ne 0) { throw 'Inno Setup compilation failed.' }

# The executable must remain usable without installing the Visual C++ redistributable.
# RUSTFLAGS enables the static CRT; verify the generated PE's architecture and GUI subsystem.
$bytes = [System.IO.File]::ReadAllBytes($binary)
$pe = [BitConverter]::ToInt32($bytes, 0x3c)
if ([BitConverter]::ToUInt32($bytes, $pe) -ne 0x00004550 -or
    [BitConverter]::ToUInt16($bytes, $pe + 4) -ne 0x8664 -or
    [BitConverter]::ToUInt16($bytes, $pe + 24) -ne 0x20b -or
    [BitConverter]::ToUInt16($bytes, $pe + 24 + 68) -ne 2) {
    throw 'Expected a Windows x64 GUI executable.'
}
$expected = @('sela-notch.exe', "Sela-$Version-windows-x64.zip", "Sela-$Version-setup-x64.exe")
$checksums = foreach ($name in $expected) {
    $path = Join-Path $output $name
    if (-not (Test-Path $path)) { throw "Missing release asset: $name" }
    $hash = (Get-FileHash $path -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  $name"
}
[System.IO.File]::WriteAllText((Join-Path $output 'SHA256SUMS.txt'), ($checksums -join "`n") + "`n", [System.Text.UTF8Encoding]::new($false))
Write-Host "Release assets prepared in $output"
