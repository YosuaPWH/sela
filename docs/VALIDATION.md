# Sela validation

## Native Performance — 0.3.0, 25 October 2026

| Check | Result |
| --- | --- |
| Formatting / Clippy / Linux build | Passed |
| `cargo test --locked` | 28 passed |
| Windows provider and DXGI integration-probe cross-target check / Clippy | Passed using the Windows GNU target; this does not execute the probe |
| Workflow syntax | actionlint passed |
| Native rendering | Both line styles and controls at 100%/150%, plus off/unavailable, recording, export, and retained panels |
| Text-only transparency | Direct framebuffer pixel matched the underlying green window; background mode produced charcoal at the same position |
| Benchmark export | A labelled demo recording exported 4305 frames to both CSV and JSON; counts and average-FPS calculation matched |
| Windows MSVC / real ETW probe / installer lifecycle | [Passed in GitHub Actions](https://github.com/YosuaPWH/sela/actions/runs/40205402006): build, Clippy, tests, real DXGI capture, static CRT, packaging, installation and removal |

The native line exposes both **Text only** and **With background**, keeps all seven metric groups on one baseline, and labels unavailable readings honestly. Explicit `--performance-demo` data was used for layout and export checks; it is not evidence of game capture. Frame-window tests reject other PIDs, keep swap chains separate, expire stale data, and verify FPS/1% low calculations. Recording tests keep one PID/chain and verify monitoring stops when disabled.

Linux captures use Xvfb, software OpenGL and xcompmgr. They establish layout and local interactions, not Windows click-through, hotkeys, real driver sensors, game compatibility, or exclusive fullscreen behavior. The Windows CI integration probe creates real DXGI presents with WARP and reads them through the embedded PresentMon process; it passed on the Windows runner independently of cross-compilation. A fallback finish review was completed directly after the separate reviewer reached its usage limit.

## English interface — 0.2.1, 25 October 2026

All app-owned interface strings are English: navigation, controls, settings, tooltips, tray actions, startup messages, and recovery states. Original demo tracks and lyrics are English. Real player metadata and LRCLIB lyrics retain their source language.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --all-targets --locked -- -D warnings` | Passed |
| `cargo test --locked` | 22 tests passed |
| `cargo build --locked` | Passed on Linux x64 |
| Native English interface | 27 captures across Media, Focus, System, Settings, and compact states at 100% and 150% scale |
| Compact lyric alignment | Six captures of short, truncated, and unavailable lyrics at 100%/150%; text centers within 0.5 physical pixel of the lyric area's center |
| Interactive Canvas | State and interaction checks at 1440, 900, and 390 px; no JavaScript errors or page overflow |

The Canvas includes all four panels, compact music/clock/focus modes, and a tray-menu approximation. Checks cover playback pause, previous/next, lyric retry across panel switches, persistent focus state, preferences, zoom, quit/relaunch, loaded fonts, and keyboard-operable HTML controls. The preview uses original sample tracks and illustrative system values. It plays no audio and does not connect to Windows or LRCLIB.

Native captures run in Xvfb with software OpenGL and xcompmgr. They establish Linux rendering and layout, not Windows desktop integration. Current gallery images use the `screenshots/sela-*` prefix. The title and expanded lyrics remain left-aligned; compact lyrics are centered within their own area to the right of the title.

An independent finish review resolved all three findings: first-draw font loading, contextual expanded error wording, and matching keyboard-operable HTML actions. The targeted recheck passed at all three browser widths.

## Existing functional coverage

The 22 tests cover focus timing, legacy and malformed configuration, LRC repeated timestamps and offsets, backward lyric seeking, plain/synced distinctions, artist/duration matching, stale-result rejection, pre-network cancellation, corrupt/expired/wrong-track caches, URL encoding, and HTTP 404-to-search fallback. Provider tests use a local HTTP server.

Media-clock tests cover pause, monotonic progress, backward seeks, duration bounds, and invalid inputs. Spring tests simulate 30/60/144 Hz, retargeting, reversal, and reduced motion. These are mathematical checks, not Windows FPS benchmarks. Telemetry tests verify on-demand warm-up and stopping after the System page closes.

In the earlier 0.2.0 check, a Linux HTTPS probe reached LRCLIB and received synchronized lyrics. Catalog content and availability can change; commercial lyrics were not included in the gallery or distribution. A Windows GNU release cross-build and Clippy passed, and static imports contained only Windows system DLLs. AccessKit is enabled; the executable imports `uiautomationcore.dll`.

The published 0.2.0 release also passed the GitHub Windows MSVC build, static-runtime dependency check, Inno Setup packaging, and automated install/uninstall lifecycle check. The tag workflow repeats these checks for each release and publishes an installer, portable ZIP, executable, and checksums after both Windows and Linux jobs pass.

## Windows desktop checks still requiring a device

- [ ] Top-center positioning, transparency, Pijar tray icon, and no taskbar button.
- [ ] Narrator reading tabs, transport controls, titles, and lyrics.
- [ ] `Ctrl+Alt+N`, every tray action, quitting, and single-instance protection including the legacy Luma instance.
- [ ] Real Spotify/browser metadata and transport; switching players or tracks must not preserve stale lyrics.
- [ ] Lyrics through pause, resume, forward/backward seek, end of track, and sleep/resume.
- [ ] LRCLIB HTTPS, offline cache, connection failure, retry, and disabling lyrics.
- [ ] Click-through outside the native region during width changes, reversals, and DPI changes.
- [ ] Animation performance at 60/120/144 Hz and idle CPU use.
- [ ] Completion sound, battery readings, legacy settings, and persistence after restart.
- [ ] DPI 100/125/150/200%, secondary/negative-coordinate monitors, monitor removal, fullscreen, and remote desktop.

Historical 0.1.x evidence is in the [Luma validation archive](archive/VALIDATION-0.1.2.md). The older GIF and screenshots without the `sela-` prefix show that earlier interface.
