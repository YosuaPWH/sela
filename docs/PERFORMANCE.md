# Performance monitoring

Sela 0.3.0 adds a native gaming line. MSI Afterburner and RTSS are not required.

## Enable it

1. Open Sela with **Ctrl+Alt+N** or the tray menu.
2. Choose **System → Performance** and enable **Show gaming performance line**.
3. Choose **Text only** or **With background**. Both keep FPS, RAM, VRAM, GPU/CPU readings, frame time, and 1% low on one line.
4. Select **Foreground application**, or choose a visible application's executable. Auto mode retains the last target while Sela or the desktop is focused.
5. Keep **Only while a game is playing** on (the default) to show the gaming line only during play. A game is the foreground application when it is presenting frames and covers its whole monitor (exclusive or borderless fullscreen). Browsers and video players are ignored. In this mode only fullscreen or marked applications are captured, so switching between ordinary apps starts no PresentMon process. Mark a windowed game, or override that list, with **Games → Mark a running app…** (or **Mark as game** next to the captured application); remove a mark with its **×** button. If frame capture is unavailable (for example, without the required permission), the fullscreen check alone decides. The line hides two seconds after the game leaves the foreground; Sela returns to the normal notch meanwhile.
6. Close the panel. The gaming line passes mouse input through to the game and does not expand on hover. Open it with **Ctrl+Alt+N** or the tray. A focus timer ending does not force the gaming panel open.

**Save style for this app** stores the selected style under its executable name; while that application is captured, the style choice edits its saved style, and **Use default style** removes it. Other applications keep the default style. Settings persist locally. The strip fits the selected monitor's logical width, with an 816 × 48 default size. The detail panel is 600 × 440; System overview is 464 × 440.

## Sources and scope

| Reading | Source and meaning |
| --- | --- |
| FPS / frame time / 1% low | Embedded PresentMon 2.6.0, reading Windows ETW presentation events for the target PID. |
| CPU / RAM | System-wide sysinfo readings, approximately once per second while monitoring is enabled. CPU waits for a baseline before showing a value. |
| NVIDIA GPU / VRAM / temperature / power | NVML, when the driver's System32 DLL and sensor are available; first NVIDIA adapter. |
| Other GPU / VRAM | Windows performance counters: busiest 3D engine, summed across processes on that engine; dedicated memory across adapters. |
| CPU temperature | Unavailable in this release. No kernel sensor driver is installed. |

A missing or unsupported value is **—** in the line and **Unavailable** in details. GPU temperature and power are not guaranteed on every device. VRAM is dedicated GPU memory; RAM is used system memory. GB values use 1024³ bytes. Hardware figures are system/adapter readings, while FPS belongs to the selected process and its dominant swap chain.

FPS means application-present cadence, not displayed/generated-frame totals. The app selects the swap chain with the most recent samples in a five-second window instead of adding multiple swap-chain rates. The rolling window is capped at 8192 samples per chain, with at most 16 chains. Frame data becomes unavailable after two seconds without new samples. The graph shows the latest 120 presents.

## Benchmarks

Choose 15, 30, or 60 seconds and **Start recording** once frames are available. **Finish** stops early. A recording stays on one PID and swap chain, finishes when its process/target changes or monitoring is disabled, and does not insert invented frames during gaps. Duration means captured frame time, excluding gaps. Sessions are capped at one million frames and the UI retains the latest 20 sessions until Sela exits.

Average FPS is `1000 × captured frames / sum(frame times in ms)`. 1% low is `1000 / mean(slowest ceil(frame count / 100) frame times)`. The latest session can be compared with the previous one; the applications may differ. **Export CSV + JSON** writes captured frame times and the report under Sela's local data folder, in `benchmarks`. **Open export folder** opens that directory.

## Capture permissions and compatibility

PresentMon may need administrator privileges or membership in Windows **Performance Log Users**, depending on system policy and target. If capture stops or no frames appear, check that the selected process is rendering, relaunch Sela with the required permission, and use **Retry**. Sela does not silently elevate itself. It uses a private, per-user trace-session name and does not stop another tool's default PresentMon session.

The overlay is intended for desktop and borderless/windowed games. Exclusive fullscreen and anti-cheat compatibility require testing with each game. Sela does not inject into games, install a kernel driver, cap FPS, change clocks/voltage, or control fans.

## Packaging and privacy

Official Windows builds embed the pinned PresentMon console executable. On first capture it is extracted into Sela's per-user cache; its SHA-256 is checked before use. No runtime download is needed. The child process is hidden, managed by a kill-on-close Windows job, and stopped when monitoring stops. Its private ETW session is cleaned up on normal shutdown and restarted using the same private name on a subsequent capture.

The provider's hash is `b2a706bc6ad475749e3b7e3409263aa1e6906d45bdcf993f6dbc0f660188f1af`. The upstream [PresentMon project](https://github.com/GameTechDev/PresentMon) is MIT-licensed; its [license](../assets/licenses/PresentMon.txt) is shipped and available in the Performance panel, including in the standalone executable. PresentMon input/display/GPU timing tracking is disabled; only presentation cadence is used. Metrics and reports stay local. Existing optional lyric lookups have their own network behavior described in the main README.

For a Windows source build, run `./scripts/fetch-presentmon.ps1` before `cargo build --locked --release`. The script downloads the pinned official executable and verifies its hash. Without it, a development build has no bundled FPS provider and explains the missing provider instead of showing demo data.

`--performance-demo` explicitly enables labelled sample frames and hardware for UI development. It never starts PresentMon or pretends to measure a game. Normal launches use real sources and unavailable states. Linux previews can show CPU/RAM and the UI; game frame capture and Windows GPU sources require Windows.
