# Releasing Sela

A pushed version tag such as `v0.2.0` builds and publishes a GitHub Release automatically. The numeric version must match the `sela-notch` package version in `Cargo.toml`. Prerelease versions such as `0.3.0-rc.1` use matching tags such as `v0.3.0-rc.1` and are marked as prereleases.

## Published files

| Asset | Purpose |
| --- | --- |
| `Sela-<version>-setup-x64.exe` | Per-user Windows installer with a Start Menu shortcut, optional desktop shortcut, and uninstaller. |
| `Sela-<version>-windows-x64.zip` | Portable app with documentation and licenses. |
| `sela-notch.exe` | Standalone Windows x64 GUI executable. |
| `SHA256SUMS.txt` | SHA-256 hashes of the three binary assets. |

The app and installer are currently unsigned. The installer requires Windows 10 version 1809 or later / Windows 11 and does not require administrator privileges. It installs to `%LOCALAPPDATA%\Programs\Sela` by default. Uninstalling removes installed program files while leaving settings and the lyric cache in their separate user-data directories.

## Create a release

1. Update the package version in `Cargo.toml` and regenerate `Cargo.lock` when preparing a new version. Add optional release notes at `docs/releases/v<version>.md`; otherwise GitHub generates notes from the commit history.
2. Commit and push the changes. Wait for the main-branch workflow to pass before tagging.
3. Create and push an annotated tag:

   ```bash
   git tag -a v0.2.0 -m "Sela v0.2.0"
   git push origin v0.2.0
   ```

   For subsequent releases, replace `0.2.0` with the new package version.
4. Open **Actions → Build, test, and release**. After the tag run succeeds, the files appear under **Releases**.

Only a `push` of a matching `v*` tag publishes a release. Branch pushes, pull requests, and manual workflow runs build and test the distribution without publishing. Updating an existing tag is not a release mechanism; publish a new version instead.

## What the workflow checks

- Formatting, Clippy, and Rust tests on Windows; core tests on Linux.
- Tag-to-package-version equality before a tagged build.
- A Windows MSVC build with the C runtime linked statically, so the app does not require a separate Visual C++ redistributable.
- The executable's x64 GUI headers and absence of dynamic Visual C++ runtime DLL imports.
- Inno Setup compilation, silent installation on a clean Windows runner, installed executable hash, required files, Start Menu shortcut, uninstall registration, and silent uninstallation.
- Checksums on Linux before publishing the release assets.

A release is created as a draft, populated with all four assets, then published. A failed upload can be retried against the draft. A rerun preserves already-published assets instead of replacing release binaries. Only the publish job receives `contents: write`; build jobs use read-only repository permissions.

The installer smoke test does not launch the app or verify desktop media integration, rendering, tray behavior, or Windows Narrator. Those still require direct testing on a Windows desktop.

## Build an installer locally

On Windows, install Rust, Visual Studio Build Tools with the C++ workload, the Windows SDK, and [Inno Setup 6.3 or later](https://jrsoftware.org/isinfo.php). In PowerShell 7:

```powershell
$env:RUSTFLAGS = '-C target-feature=+crt-static'
cargo build --locked --release --target x86_64-pc-windows-msvc
./scripts/package-windows.ps1 -Version 0.2.0
```

Files are written to `dist/release`. Packaging alone does not install the app. `scripts/test-installer.ps1` is a separate integration test for a clean Windows user profile; it refuses to run if Sela is already registered as installed.
