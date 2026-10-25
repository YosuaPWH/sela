[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$InstallerPath,
    [Parameter(Mandatory = $true)][string]$ExpectedExecutable
)

$ErrorActionPreference = 'Stop'
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\{8981B9F1-B013-4470-828C-FBE4182EDCEA}_is1'
if (Test-Path $uninstallKey) { throw 'Run this smoke test on a clean Windows user profile; Sela is already installed.' }
$tempRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [System.IO.Path]::GetTempPath() }
$installDir = Join-Path $tempRoot ('Sela-install-test-' + [guid]::NewGuid().ToString('N'))
$installLog = Join-Path $tempRoot 'sela-install.log'
$uninstallLog = Join-Path $tempRoot 'sela-uninstall.log'
$process = Start-Process -FilePath (Resolve-Path $InstallerPath) -ArgumentList @(
    '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/SP-',
    ('/DIR="{0}"' -f $installDir), ('/LOG="{0}"' -f $installLog)
) -Wait -PassThru
if ($process.ExitCode -ne 0) { throw "Installer failed with exit code $($process.ExitCode). See $installLog" }

try {
    $installed = Join-Path $installDir 'sela-notch.exe'
    if (-not (Test-Path $installed)) { throw 'Installed executable is missing.' }
    if ((Get-FileHash $installed).Hash -ne (Get-FileHash $ExpectedExecutable).Hash) {
        throw 'Installed executable differs from the release build.'
    }
    foreach ($file in @('LICENSE', 'README.md', 'assets/OFL.txt', 'assets/licenses/PresentMon.txt', 'unins000.exe')) {
        if (-not (Test-Path (Join-Path $installDir $file))) { throw "Missing installed file: $file" }
    }
    $shortcut = Join-Path ([Environment]::GetFolderPath('Programs')) 'Sela.lnk'
    if (-not (Test-Path $shortcut)) { throw 'Start Menu shortcut is missing.' }
    if (-not (Test-Path $uninstallKey)) { throw 'Per-user uninstall registration is missing.' }
} finally {
    $uninstaller = Join-Path $installDir 'unins000.exe'
    if (Test-Path $uninstaller) {
        $process = Start-Process -FilePath $uninstaller -ArgumentList @(
            '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', ('/LOG="{0}"' -f $uninstallLog)
        ) -Wait -PassThru
        if ($process.ExitCode -ne 0) { throw "Uninstaller failed with exit code $($process.ExitCode). See $uninstallLog" }
    }
}
if (Test-Path (Join-Path $installDir 'sela-notch.exe')) { throw 'Uninstaller left the application executable behind.' }
if (Test-Path $uninstallKey) { throw 'Uninstaller left its per-user registration behind.' }
if (Test-Path $shortcut) { throw 'Uninstaller left its Start Menu shortcut behind.' }
Write-Host 'Installer smoke test passed: installed files, executable hash, shortcut, registration, and uninstall.'
