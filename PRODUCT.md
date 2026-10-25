# Sela

<!-- impeccable:product-schema 1 -->

## Platform
adaptive

Native Windows desktop overlay. Windows is the supported release target; Linux is a development preview.

## Stack
Rust, eframe/egui with its AccessKit accessibility backend enabled, and Windows WinRT and Win32 APIs. The user explicitly requested Rust and Windows; the GUI library is an implementation choice. The package and executable are `sela-notch`, version 0.2.1.

## Product Purpose
A quiet top-center companion for music, focus, and device status. Sela keeps a small presence at rest and opens into the controls or information the user needs. The approved product direction is **Operate**: improve the working native overlay within its selected visual identity.

## Brand Commitments
The user selected **Sela** and **Pijar**, the small firefly mascot. The English interface expresses its durable voice as “A small companion that gives you room. Quiet while you focus, warm when it is time to rest.” The footer reads “Sela · a little space for you.” Sela and Pijar remain the approved names.

The Mac-notch reference establishes the top-center silhouette. The approved Sela world uses charcoal, sage, warm amber, ivory text, DM Sans Regular and Semibold, and the generated Pijar asset. Pijar is the leftmost character in `assets/pijar-sheet.png`; Rust crops the first 30% before texture upload. The other characters on that sheet are archived concepts, not Sela identities. A setting hides Pijar and restores the authored equalizer mark. Palette and component tokens live in `DESIGN.md`.

## Capabilities and Constraints
Four native pages provide media transport, a focus timer, live CPU/RAM/battery, and settings. Tray recovery, a global shortcut, monitor selection, top offset, hover opening, reduced motion, completion sound, local settings, and single-instance behavior remain part of the overlay. The owned interface is English, including tabs, settings, tooltips, accessibility names, tray actions, startup errors, and demo text. All repository README.md files remain English. Song titles, artist metadata, and real lyrics retain their source language; the app does not translate song content.

The approved Canvas music layout pairs a player with contextual lyrics. Expanded media uses 600 × 370 logical points; focus, system, and settings use 464 × 370. With a track present and lyrics enabled, the compact music pill is 580 × 54 and shows a left-aligned title beside one lyric centered within the right-hand lyric area, as requested by the user. Expanded lyrics remain left-aligned. Idle, active focus, or lyrics-disabled compact mode uses 324 × 48. Geometry and motion are documented in `DESIGN.md`.

When lyrics are enabled, a background worker sends track title, artist, and available duration over HTTPS to LRCLIB. Settings disclose this and provide a lyrics toggle. Local results persist for 30 days; not-found results persist for one day. Synced, unsynced, instrumental, loading, error, missing-metadata, and not-found states are distinct. A retry bypasses the cache. Stale results cannot replace a newer selection. No account or translation service is required.

Lyrics follow the Windows media position with monotonic elapsed time between updates. Pausing holds the line; a new position or a seek in the external player recalculates it. The native progress strip is read-only. A normal launch uses real Windows media data and honest empty/unavailable states. The explicit `--demo` flag supplies clearly labelled original sample tracks and lyrics, without audio or LRCLIB requests. Demo Previous/Next cycles synced, long-text, not-found, plain, and instrumental fixtures. Normal playback uses “NOW PLAYING” or “PAUSED” and LRCLIB source captions; demo playback separately identifies “DEMO · SAMPLE DATA” and “Original Sela demo lyrics · no audio.”

## Operating Context
Windows 10 version 1809 or newer / Windows 11, desktop session, multiple DPI scales and monitors. Media control requires a player that exposes a Windows media session. The native 600 × 370 backing window remains fixed while the drawn silhouette and Win32 drawing/input region follow the animated bounds. Settings and lyric cache belong to the current user. Version tags matching the Cargo package version publish a per-user Windows installer, a portable ZIP, a standalone executable, and SHA-256 checksums through GitHub Actions. Installer lifecycle checks run on a clean Windows runner before publication. Existing Luma settings are loaded when Sela settings do not yet exist; subsequent saves use the Sela location.

A companion browser Canvas in `/workspace/sites/notch-concept-studio` presents all four panels, compact music/idle/focus states, a tray approximation, and six overview artboards. Its sample timeline, device values, settings, and retry flows are local preview behavior. It plays no audio and connects to neither Windows nor LRCLIB. The Canvas seek control is preview-only; native Rust remains authoritative for application behavior.

The native UI supplies semantic widget names to AccessKit, including the full text of clipped lyric labels. Windows Narrator behavior remains runtime-unverified.

Linux can preview the interface and explicit demo, but it does not provide Windows media, tray, hotkey, or native input-region evidence. Windows integration must be validated on Windows.

## Reference Follow-up
The user requested a review of `lucacadalora/poni` and its README. The earlier Luma implementation adopted header-wheel navigation, demand-driven system sampling, and README/gallery structure. Sela retains those useful behaviors within the approved sage-and-amber identity. Source review and attribution remain in `docs/REFERENCE_PONI.md`.
