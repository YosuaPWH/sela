# Sela

<!-- impeccable:product-schema 1 -->

## Platform
adaptive

Native Windows desktop overlay. Windows is the supported release target; Linux is a development preview.

## Stack
Rust, eframe/egui with its AccessKit accessibility backend enabled, and Windows WinRT and Win32 APIs. The user explicitly requested Rust and Windows; the GUI library is an implementation choice. The package and executable are `sela-notch`, version 0.3.0. Performance capture uses an embedded, hash-pinned PresentMon provider, sysinfo, Windows performance counters, and optional NVIDIA NVML sensors.

## Product Purpose
A quiet top-center companion for music, focus, device status, and gaming performance. Sela keeps a small presence at rest and opens into the controls or information the user needs. The approved product direction is **Operate**: improve the working native overlay within its selected visual identity.

## Brand Commitments
The user selected **Sela** and **Pijar**, the small firefly mascot. The English interface expresses its durable voice as “A small companion that gives you room. Quiet while you focus, warm when it is time to rest.” The footer reads “Sela · a little space for you.” Sela and Pijar remain the approved names.

The Mac-notch reference establishes the top-center silhouette. The approved Sela world uses charcoal, sage, warm amber, ivory text, DM Sans Regular and Semibold, and the generated Pijar asset. Pijar is the leftmost character in `assets/pijar-sheet.png`; Rust crops the first 30% before texture upload. The other characters on that sheet are archived concepts, not Sela identities. A setting hides Pijar and restores the authored equalizer mark. Palette and component tokens live in `DESIGN.md`.

## Capabilities and Constraints
Four native pages provide media transport, a focus timer, live CPU/RAM/battery, and settings. Tray recovery, a global shortcut, monitor selection, top offset, hover opening, reduced motion, completion sound, local settings, and single-instance behavior remain part of the overlay. The owned interface is English, including tabs, settings, tooltips, accessibility names, tray actions, startup errors, and demo text. All repository README.md files remain English. Song titles, artist metadata, and real lyrics retain their source language; the app does not translate song content.

The approved Canvas music layout pairs a player with contextual lyrics. Expanded media uses 600 × 370 logical points; focus and settings use 464 × 370. System overview uses 464 × 440 and System → Performance uses 600 × 440. With a track present and lyrics enabled, the compact music pill is 580 × 54 and shows a left-aligned title beside one lyric centered within the right-hand lyric area, as requested by the user. Expanded lyrics remain left-aligned. With performance monitoring off, idle, active focus, or lyrics-disabled compact mode uses 324 × 48. Geometry and motion are documented in `DESIGN.md`.

The approved Performance Canvas adds an optional 816 × 48 gaming line that fits the selected monitor's logical width. Explicit **Text only** and **With background** radio choices show FPS, RAM, VRAM, GPU and CPU utilization/temperature, frame time, and 1% low in fixed-width cells on one baseline. Text only outlines the glyphs without a filled panel; With background retains charcoal. While closed, both pass mouse input through and suppress hover opening. Ctrl+Alt+N or the tray opens controls. Performance takes compact priority over media and focus; a focus session ending does not force the panel open over a game.

System → Performance provides enable/disable, foreground or visible-application selection, Retry, per-executable style saving, recent frame history, and 15/30/60-second benchmarks with an early Finish action. Sessions start empty, retain the latest 20 until exit, compare the latest result with the previous result, and export captured frame times and reports as CSV/JSON to the local benchmarks folder. Recording stays on one PID and dominant swap chain, excludes gaps without frames, and finishes on target changes or monitoring stop. Foreground mode retains the last application while Sela or the desktop is focused; it follows applications rather than classifying games.

Presented-frame cadence comes from the embedded PresentMon 2.6.0 provider. CPU/RAM are system-wide sysinfo readings; Windows GPU counters supply GPU/VRAM fallback data, and optional NVML supplies the first NVIDIA adapter's available readings. Hardware readings have system/adapter scope; FPS belongs to the selected process and dominant swap chain. CPU temperature is unavailable in this release, and unsupported sensors stay unavailable. Normal launches never fabricate values. The separate `--performance-demo` flag labels sample readings as DEMO and never starts capture or prefills benchmark sessions. `docs/PERFORMANCE.md` owns the detailed source, calculation, permissions, export, and compatibility contract.

When lyrics are enabled, a background worker sends track title, artist, and available duration over HTTPS to LRCLIB. Settings disclose this and provide a lyrics toggle. Local results persist for 30 days; not-found results persist for one day. Synced, unsynced, instrumental, loading, error, missing-metadata, and not-found states are distinct. A retry bypasses the cache. Stale results cannot replace a newer selection. No account or translation service is required.

Lyrics follow the Windows media position with monotonic elapsed time between updates. Pausing holds the line; a new position or a seek in the external player recalculates it. The native progress strip is read-only. A normal launch uses real Windows media data and honest empty/unavailable states. The explicit `--demo` flag supplies clearly labelled original sample tracks and lyrics, without audio or LRCLIB requests. Demo Previous/Next cycles synced, long-text, not-found, plain, and instrumental fixtures. Normal playback uses “NOW PLAYING” or “PAUSED” and LRCLIB source captions; demo playback separately identifies “DEMO · SAMPLE DATA” and “Original Sela demo lyrics · no audio.”

## Operating Context
Windows 10 version 1809 or newer / Windows 11, desktop session, multiple DPI scales and monitors. Media control requires a player that exposes a Windows media session. The native backing window is at most 840 × 440 logical points while the drawn silhouette and Win32 drawing/input region follow the animated bounds. Settings, lyric cache, and exported benchmarks belong to the current user. Version tags matching the Cargo package version publish a per-user Windows installer, a portable ZIP, a standalone executable, and SHA-256 checksums through GitHub Actions. Installer lifecycle checks run on a clean Windows runner before publication. Existing Luma settings are loaded when Sela settings do not yet exist; subsequent saves use the Sela location.

Official Windows builds embed the pinned provider and its MIT license. The extracted executable is SHA-256 checked, runs hidden in a kill-on-close job, and uses Sela's private per-user ETW session. Monitoring off releases capture and sensors. Capture may require administrator or Performance Log Users permission; Sela does not silently elevate. It does not inject into games or install a kernel sensor driver. Metrics and benchmark reports stay local.

A companion browser Canvas in `/workspace/sites/notch-concept-studio` presents all four panels, compact music/idle/focus states, a tray approximation, and six overview artboards. Its sample timeline, device values, settings, and retry flows are local preview behavior. It plays no audio and connects to neither Windows nor LRCLIB. The Canvas seek control is preview-only; native Rust remains authoritative for application behavior.

The native UI supplies semantic widget names to AccessKit, including the full text of clipped lyric labels. Windows Narrator behavior remains runtime-unverified.

Native Linux captures at 100% and 150% show both performance styles, controls, unavailable/off states, history, benchmark/export, and retained panels. Linux can preview CPU/RAM and explicit demo data, but it does not provide Windows media, tray, hotkey, input-region, or real game-capture evidence. The new Windows DXGI/PresentMon integration probe and v0.3.0 installer CI have not yet run. Exclusive fullscreen, anti-cheat compatibility, and real game runtime require Windows device testing.

## Reference Follow-up
The user requested a review of `lucacadalora/poni` and its README. The earlier Luma implementation adopted header-wheel navigation, demand-driven system sampling, and README/gallery structure. Sela retains those useful behaviors within the approved sage-and-amber identity. Source review and attribution remain in `docs/REFERENCE_PONI.md`.
