<p align="center"><img src="docs/brand/sela-mark.svg" width="72" height="72" alt="Sela logo"></p>

<h1 align="center">Sela</h1>

<p align="center"><strong>Keep music close. Stay focused.</strong><br>A Windows notch built with Rust, accompanied by Pijar the firefly.</p>

<p align="center">
  <a href="#get-started"><strong>Get started</strong></a> ·
  <a href="#music-and-lyrics">Music & lyrics</a> ·
  <a href="docs/BUILDING.md">Build</a> ·
  <a href="docs/VALIDATION.md">Validation</a>
</p>

<p align="center"><code>Windows 10 / 11</code> · <code>Native Rust</code> · <code>Portable</code> · <a href="LICENSE">MIT</a></p>

<p align="center"><img src="docs/screenshots/sela-media-150.png" width="600" alt="Sela with sage playback controls and the active lyric highlighted in amber"></p>

Sela lives at the top center of your screen. Open the panel to control music or start a focus session. Close it, and the song title and one current lyric line stay with you.

Charcoal, sage, and amber keep the panel quiet. Width and height use continuous springs that preserve their motion when you reverse direction. Pijar adds a little warmth; you can hide the mascot in Settings.

<sub>The gallery shows the Rust app running on Linux. Music and lyrics in these images use original demo data without audio. Windows integration still requires testing on a Windows device; see the <a href="docs/VALIDATION.md">validation evidence and limits</a>.</sub>

[Explore every panel in the interactive Canvas preview](https://notch-concept-studio.yosuah.chatgpt.site) (owner-only), including compact modes, playback states, Focus, System, Settings, and the tray menu.

## Music and lyrics

<p align="center"><img src="docs/screenshots/sela-compact-150.png" width="600" alt="Collapsed notch with a left-aligned song title and one centered lyric line in the right-hand area"></p>

- **Windows media controls:** metadata, playback position, play/pause, previous, and next through Windows System Media Transport Controls. Buttons follow the player's capabilities.
- **Synchronized lyrics:** the active line is highlighted in amber; the expanded panel shows the previous and next lines. Lyrics follow the player's position, including pauses and position changes.
- **Lyric alignment:** the collapsed notch centers the current lyric within the area to the right of the song title and divider. Expanded lyrics remain left-aligned.
- **Readable text:** long titles and lyrics truncate with an ellipsis; hover to read the full text. Lyrics without timestamps appear as plain text in the expanded panel.
- **Clear states:** loading, connection errors, instrumental tracks, incomplete metadata, and unavailable lyrics have their own messages. Results for an earlier song cannot replace the current song's lyrics.

Lyrics come from [LRCLIB](https://lrclib.net). Availability, content, and timing depend on the source data. Sela matches the title, artist, and duration; it does not infer timing from plain lyrics. The timeline displays playback position; seeking controls are not implemented yet.

## Gaming performance

<p align="center"><img src="docs/screenshots/sela-performance-text.png" width="816" alt="Sela gaming line with text-only FPS, RAM, VRAM, GPU, CPU, frame time and 1% low"></p>
<p align="center"><img src="docs/screenshots/sela-performance-background.png" width="816" alt="The same one-line metrics with a charcoal background"></p>

Open **System → Performance**, enable **Show gaming performance line**, and choose **Text only** or **With background**. The closed line is click-through and keeps its readings visible without opening the panel. **Ctrl+Alt+N** or the tray opens the controls. The images use the explicitly labelled native performance demo; normal launches use real data and unavailable states.

- **FPS and frame times:** embedded PresentMon 2.6.0 captures the chosen process. No MSI Afterburner or RTSS installation is needed. Capture may require administrator or Performance Log Users permissions.
- **Hardware:** live system CPU/RAM, Windows GPU/VRAM counters, and NVIDIA NVML sensors when supported. Unsupported readings, including CPU temperature, show `—`.
- **Benchmarks:** capture 15/30/60 seconds, finish early, compare sessions, and export their actual frames as CSV/JSON. Save the line style per executable.

[Read the performance guide](docs/PERFORMANCE.md) for source definitions, permissions, packaging, and game compatibility. Borderless/windowed games are recommended. Overclocking, fan control and FPS limiting are outside this release.

## Room to work

The interface uses English. Track metadata and real lyrics retain the language provided by your player or LRCLIB.

| Page | What it does |
| --- | --- |
| **Media** | Windows music controls and lyrics in a 600×370 panel, or a 580×54 collapsed music notch. |
| **Focus** | Sessions of 15, 25, or 50 minutes, with pause, resume, reset, and an optional completion sound. An active focus countdown takes priority when the gaming line is off. |
| **System** | CPU/RAM/battery overview and native Performance controls, frame history and benchmarks. |
| **Settings** | Monitor, top offset, hover, reduced motion, sound, lyrics, and mascot preferences. Settings are stored locally. |

| Focus | System |
| --- | --- |
| ![Sela focus timer](docs/screenshots/sela-focus.png) | ![CPU and RAM readings from the development machine](docs/screenshots/sela-system.png) |
| **Unavailable lyrics** | **Settings** |
| ![Unavailable lyrics message using demo data](docs/screenshots/sela-missing.png) | ![Sela settings and disclosure of data sent for lyric lookup](docs/screenshots/sela-settings.png) |

Sela stays on top, without a window frame or taskbar button. The tray provides access to the panel, focus timer, settings, and exit. Pinning keeps a hover-opened panel visible. A single instance per Windows session prevents duplicate overlays.

## Get started

**Windows 10 version 1809 or later / Windows 11, x64**, with an OpenGL 2.0+ driver.

1. Open the **[latest GitHub Release](https://github.com/YosuaPWH/sela/releases/latest)** and download **`Sela-<version>-setup-x64.exe`**.
2. Run the installer, then open **Sela** from the Start Menu.
3. Hover over the notch briefly, click it, or press **`Ctrl+Alt+N`**. Play music in a player that supports Windows media controls.

For portable use, extract **`Sela-<version>-windows-x64.zip`** and run **`sela-notch.exe`**, or download the standalone executable from the same release. **`SHA256SUMS.txt`** provides hashes for all three files.

The installer runs for the current user, provides a Start Menu shortcut and uninstaller, and does not require administrator privileges. The app and installer are unsigned. Sela does not add itself to Windows startup automatically.

```powershell
.\sela-notch.exe --expanded
```

To explore the interface without a media player:

```powershell
.\sela-notch.exe --demo --expanded
```

Demo mode is labelled and uses no audio or LRCLIB requests. Previous/Next cycles through five original examples: synchronized lyrics, long text, unavailable lyrics, plain lyrics, and an instrumental track. Launch without `--demo` to use Windows media sessions.

| Action | Control |
| --- | --- |
| Open / close | `Ctrl+Alt+N` |
| Close | `Esc` while the panel has focus |
| Open on hover | Keep the pointer over the notch for about 240 ms |
| Switch pages | Click a tab or scroll over the header |
| Keep a hover-opened panel visible | Click the pin |
| Exit | Tray menu, or **Settings → Quit Sela**; scroll down in Settings |

Panels opened by a click or shortcut stay open until closed. A hover-opened panel closes about 650 ms after the pointer leaves, unless pinned. **Reduce motion** displays the final state immediately. Disabling lyrics uses the shorter music notch; an active focus timer takes priority when the gaming line is off.

## Data and connectivity

Sela has no accounts or telemetry. Player metadata and system statistics are read from your device. **When lyrics are enabled**, the track title, artist, and duration are sent to LRCLIB over HTTPS. Turn off **Settings → Show music lyrics** to stop new lookups. An in-flight request may finish its connection, but its result will be ignored.

- Successful lyric results are cached locally for 30 days; not-found results for 1 day. Connection failures are not cached. A valid cache entry can be used offline.
- Settings: `%APPDATA%\Sela\Sela\config\settings.json`. Cache: `%LOCALAPPDATA%\Sela\Sela\cache\lyrics`.
- Legacy Luma settings are read if Sela settings do not exist yet. Subsequent saves use the Sela location, preserving the old file.
- Invalid settings JSON falls back to defaults and displays a message in Settings. The original file is not overwritten until you save a change.
- Overview CPU/RAM sampling stops when its page closes; performance sampling continues while the gaming line is enabled and stops when disabled; the collapsed battery indicator is checked every 30 seconds. The focus timer and session count end when the app exits.

## Development

```powershell
./scripts/fetch-presentmon.ps1
cargo build --locked --release
.\target\release\sela-notch.exe
```

See the [build guide](docs/BUILDING.md), [release guide](docs/RELEASING.md), [validation notes](docs/VALIDATION.md), [design system](DESIGN.md), and [product scope](PRODUCT.md). Formatting, Clippy, and 28 tests passed on Linux. The Windows provider and its DXGI probe also passed cross-target compile and Clippy checks; Windows MSVC CI also passed real DXGI capture, static-runtime checks, packaging and install/uninstall validation. Windows runtime behavior—including media, tray, global shortcuts, click-through, and mixed-DPI monitors—still requires direct testing on Windows.

Version tags such as `v0.3.0` automatically build the executable, portable ZIP, and installer, test installation/uninstallation, and publish the files to GitHub Releases. The tag must match the package version in `Cargo.toml`; see the [release guide](docs/RELEASING.md) for subsequent releases.

## Credits

- [PresentMon](https://github.com/GameTechDev/PresentMon): embedded frame capture under its [MIT license](assets/licenses/PresentMon.txt).
- [egui/eframe](https://github.com/emilk/egui), [windows-rs](https://github.com/microsoft/windows-rs), [sysinfo](https://github.com/GuillaumeGomez/sysinfo), [reqwest](https://github.com/seanmonstar/reqwest), and [LRCLIB](https://lrclib.net).
- DM Sans is included under the [SIL Open Font License](assets/OFL.txt). Pijar is a project-generated asset; its [prompt and provenance](assets/README.md) are included. Demo lyrics are original Sela text.

Code is licensed under **[MIT](LICENSE)**. This is an independent project, unaffiliated with Apple, Spotify, or LRCLIB.
