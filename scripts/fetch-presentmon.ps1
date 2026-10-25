[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$folder = Join-Path (Split-Path $PSScriptRoot -Parent) 'tools/presentmon'
New-Item -ItemType Directory -Path $folder -Force | Out-Null
$destination = Join-Path $folder 'PresentMon.exe'
$expected = 'b2a706bc6ad475749e3b7e3409263aa1e6906d45bdcf993f6dbc0f660188f1af'
if ((Test-Path $destination) -and (Get-FileHash $destination -Algorithm SHA256).Hash.ToLowerInvariant() -eq $expected) { return }
$temporary = "$destination.download"
try {
    Invoke-WebRequest -Uri 'https://github.com/GameTechDev/PresentMon/releases/download/v2.6.0/PresentMon-2.6.0-x64.exe' -OutFile $temporary
    if ((Get-FileHash $temporary -Algorithm SHA256).Hash.ToLowerInvariant() -cne $expected) { throw 'PresentMon 2.6.0 checksum mismatch.' }
    Move-Item $temporary $destination -Force
} finally {
    if (Test-Path $temporary) { Remove-Item $temporary -Force }
}
Write-Host 'Verified PresentMon 2.6.0, ready to embed in Sela.'
