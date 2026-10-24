---
name: Luma Notch
description: A quiet black notch for Windows media, focus, and device status.
colors:
  notch-black: "#090B0A"
  raised-charcoal: "#202622"
  control-hover: "#303A33"
  soft-white: "#F3F6F4"
  muted-sage: "#A9B4AD"
  mint: "#A9EEC0"
  error-coral: "#FFB4AB"
typography:
  display:
    fontFamily: "Hack, monospace"
    fontSize: "57px"
  title:
    fontFamily: "Ubuntu Light, sans-serif"
    fontSize: "17px"
  body:
    fontFamily: "Ubuntu Light, sans-serif"
    fontSize: "13px"
  metric:
    fontFamily: "Hack, monospace"
    fontSize: "16px"
  caption:
    fontFamily: "Ubuntu Light, sans-serif"
    fontSize: "11px"
  time-caption:
    fontFamily: "Hack, monospace"
    fontSize: "11px"
  footer:
    fontFamily: "Ubuntu Light, sans-serif"
    fontSize: "10px"
rounded:
  control: "8px"
  floating-top: "16px"
  notch-bottom: "24px"
  progress: "2px"
spacing:
  tab-gap: "4px"
  control-gap: "10px"
  button-horizontal: "14px"
  button-vertical: "8px"
  content-horizontal: "24px"
  content-vertical: "18px"
components:
  notch-compact:
    backgroundColor: "{colors.notch-black}"
    textColor: "{colors.soft-white}"
    typography: "{typography.body}"
    width: "324px"
    height: "48px"
  notch-expanded:
    backgroundColor: "{colors.notch-black}"
    textColor: "{colors.soft-white}"
    width: "464px"
    height: "370px"
    padding: "18px 24px"
  button-focus:
    backgroundColor: "{colors.mint}"
    textColor: "{colors.notch-black}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    width: "160px"
    height: "35px"
    padding: "8px 14px"
  button-icon:
    textColor: "{colors.muted-sage}"
    rounded: "{rounded.control}"
    size: "32px"
    padding: "6px"
  button-icon-active:
    backgroundColor: "{colors.raised-charcoal}"
    textColor: "{colors.mint}"
    rounded: "{rounded.control}"
    size: "32px"
  tab:
    textColor: "{colors.muted-sage}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    width: "99px"
    height: "34px"
  tab-active:
    backgroundColor: "{colors.raised-charcoal}"
    textColor: "{colors.mint}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    width: "99px"
    height: "34px"
  duration-selected:
    backgroundColor: "{colors.raised-charcoal}"
    textColor: "{colors.mint}"
    rounded: "{rounded.control}"
    padding: "8px 14px"
  progress-track:
    backgroundColor: "{colors.raised-charcoal}"
    rounded: "{rounded.progress}"
    height: "3px"
---

# Design System: Luma Notch

## Overview

**Creative North Star: "The Quiet Black Notch"**

The user's Mac-notch reference establishes a black silhouette centered at the top of the display. Luma behaves like a small instrument: compact at rest, expanding around the task in use. Mint highlights interaction and state while the surrounding desktop remains visible.

This document extracts the implemented Rust/egui interface from `src/app.rs` and `src/icons.rs`. Windows is the native target; Linux is a development preview. The reviewed visual evidence is in `docs/screenshots/*.png`, including focus and settings at 100% and 150% scale; it does not establish Windows runtime validation.

**Key Characteristics:**

- A single black silhouette with gently rounded lower corners.
- Sparse mint accents, muted labels, and bright measured values.
- Authored vector icons and a large monospace focus timer.
- Four compact task tabs with Indonesian labels and truthful unavailable states.

## Colors

The palette is near-black and sage, with mint reserved for the action or state that needs attention. Frontmatter contains the normative values.

### Primary

- **Mint:** logo, active tab and pinned control, focus action, progress fill, and positive timer state.

### Neutral

- **Notch Black:** the opaque notch body and dark text on the mint focus button.
- **Raised Charcoal:** active and hovered custom controls, selected durations, and progress tracks.
- **Control Hover:** hovered backgrounds for egui controls that use the configured widget fill.
- **Soft White:** primary content and large timer or metric values.
- **Muted Sage:** inactive icons, captions, secondary labels, and unavailable data.
- **Error Coral:** actionable media errors and attention text.

**The Black Silhouette Rule.** Preserve the black body and top-center placement established by the user's reference.

## Typography

Proportional text uses eframe's embedded **Ubuntu-Light** font; measured values and time use embedded **Hack Regular**. These are the defaults bundled by `epaint_default_fonts` (0.31.1), not Windows system fonts. The font-family names in frontmatter describe those faces; a browser preview requires locally available fonts or falls back to its sans/monospace families.

The timer is the display role, metric values are secondary numerical anchors, and title/body/caption/footer roles carry decreasing emphasis. Media headings additionally use local sizes between 19 and 21 logical pixels. egui controls retain their native line layout. `RichText::strong()` changes the strong text color; it does not load a bold face or define a heavier font weight.

## Layout

All measurements are egui logical points, represented as CSS pixels in the portable tokens. Compact and expanded dimensions are fixed by mode, rather than web breakpoints. Center the viewport on the selected monitor and apply the configured top offset (0–80 logical points). At 150% scale the expanded viewport is 696 × 555 physical pixels.

The expanded body has horizontal and vertical content insets, a header, a four-tab row, one task area, and a footer anchored near the bottom. Tabs use a narrow gap; ordinary controls use the shared item spacing. Settings scroll within a bounded area (207 logical points high). Long media titles truncate. Error detail also truncates and retains a tooltip. Compact content clips within its allocated text region.

## Elevation & Depth

The main notch is a flat opaque surface over a transparent native window. The backing viewport stays at the expanded dimensions during normal operation so the OS does not resize and recenter it on every animation frame. Windows clips its drawing and input region to the animated bounds; the Linux development preview retains the full input canvas. No custom shadow or blur is painted behind it. Raised charcoal and spacing distinguish interactive regions. Native egui popups and tooltips retain their framework defaults; they are not a separately authored elevation system.

## Shapes

The default notch attaches flush to the monitor edge with square upper corners and rounded lower corners. A positive top offset rounds the upper corners using the floating-top token. Controls share a smaller radius, and progress tracks are thin rounded strips.

Icons are authored vector geometry with a consistent line stroke (1.65 logical points), supplemented by filled transport symbols and music notes. The equalizer logo uses three vertical strokes (3 logical points). Custom icon buttons inset a 20-point icon inside a 32-point target; tab icons are 17 points. No emoji is used as an interface icon.

## Components

### Notch shell

Compact mode shows the logo, current focus timer or media title, and battery information when available. Clicking opens the expanded panel. Expansion uses an analytically integrated, critically damped spring (opening response 32/s; closing response 38/s). Position and velocity survive interruption; closing settles faster than opening, without an elastic bounce. Compact content fades out over progress 0.02–0.30 and expanded content fades in over 0.20–0.92, using smoothstep. The expanded layout retains its final width, is masked by the growing silhouette, and settles through a 6-point vertical reveal. Reduced motion snaps geometry and content to their destinations and disables egui widget interpolation. Hover opening waits 240 ms, and a hover-opened panel closes 650 ms after the pointer leaves unless pinned.

### Navigation

Media, Fokus, Sistem, and Atur share a single row. Selected tabs use mint text and icons on raised charcoal. Hover adds the raised surface; keyboard focus also adds a mint inner outline (1 logical point).

### Buttons and duration chips

The focus action uses mint with black text. Header, reset, and transport icon buttons use muted icons; active buttons use mint on charcoal. Hover supplies charcoal and keyboard focus adds the same mint outline as tabs. Disabled transport icons are dimmed with `MUTED.gamma_multiply(0.35)`. Duration choices use charcoal and mint when selected and cannot change while the timer is running or paused. Settings retain egui checkbox, slider, and combo-box behavior.

### Timer, metrics, and media

The focus timer is centered and monospace. System metrics pair a muted label with a right-aligned monospace value and shared progress strip. Progress fill is clamped to the track width. Media empty/error states state what is available and provide an action or explanation; unavailable battery data is labelled explicitly. The release does not substitute sample content.

### Portable component previews

`.impeccable/design.json` contains self-contained HTML/CSS illustrations of these native patterns for design tooling. They approximate egui rendering and do not replace the Rust widgets. Its tonal ramps are generated inspection aids, not extra application colors.

## Do's and Don'ts

### Do:

- **Do** preserve the top-centered black silhouette and the compact/expanded geometry.
- **Do** use mint to identify active controls, progress, and the principal focus action.
- **Do** keep numerical timers and metrics monospace and ordinary labels proportional.
- **Do** retain visible keyboard focus, explanatory tooltips, reduced motion, and truthful unavailable states.
- **Do** check both 100% and 150% scale against the captured Linux preview evidence.

### Don't:

- **Don't** replace the user-pinned black notch with a different visual direction.
- **Don't** claim the embedded fonts are Windows system fonts or that strong text loads a bold face.
- **Don't** introduce emoji icons, fabricated media data, or invented battery values.
- **Don't** treat Linux screenshots as evidence that Windows integrations have been exercised.
