---
name: Sela
description: A quiet Windows companion with a charcoal silhouette, sage controls, and amber lyrics.
colors:
  charcoal: "#0e1110"
  raised-charcoal: "#242b27"
  control-hover: "#303a33"
  ivory: "#f3f2ec"
  muted-sage: "#a6b2a5"
  sage: "#b8d8ba"
  amber: "#f2c879"
  error-coral: "#ffb4ab"
typography:
  display:
    fontFamily: "Hack, monospace"
    fontSize: "57px"
    fontWeight: 400
  title:
    fontFamily: "DM Sans, sans-serif"
    fontSize: "17px"
    fontWeight: 600
  track-title:
    fontFamily: "DM Sans, sans-serif"
    fontSize: "18px"
    fontWeight: 600
  lyric-current:
    fontFamily: "DM Sans, sans-serif"
    fontSize: "18px"
    fontWeight: 600
  body:
    fontFamily: "DM Sans, sans-serif"
    fontSize: "13px"
    fontWeight: 400
  metric:
    fontFamily: "Hack, monospace"
    fontSize: "16px"
    fontWeight: 400
  caption:
    fontFamily: "DM Sans, sans-serif"
    fontSize: "11px"
    fontWeight: 400
  footer:
    fontFamily: "DM Sans, sans-serif"
    fontSize: "10px"
    fontWeight: 400
rounded:
  control: "8px"
  media-tile: "12px"
  floating-top: "16px"
  notch-bottom: "24px"
  progress: "2px"
spacing:
  tab-gap: "4px"
  lyric-gap: "7px"
  control-gap: "10px"
  button-horizontal: "14px"
  button-vertical: "8px"
  content-horizontal: "24px"
  content-vertical: "18px"
  media-column-gap: "28px"
components:
  notch-compact:
    backgroundColor: "{colors.charcoal}"
    textColor: "{colors.ivory}"
    typography: "{typography.body}"
    width: "324px"
    height: "48px"
  music-compact:
    backgroundColor: "{colors.charcoal}"
    textColor: "{colors.ivory}"
    typography: "{typography.body}"
    width: "580px"
    height: "54px"
  notch-expanded:
    backgroundColor: "{colors.charcoal}"
    textColor: "{colors.ivory}"
    width: "464px"
    height: "370px"
    padding: "18px 24px"
  media-canvas:
    backgroundColor: "{colors.charcoal}"
    textColor: "{colors.ivory}"
    width: "600px"
    height: "370px"
    padding: "18px 24px"
  button-focus:
    backgroundColor: "{colors.sage}"
    textColor: "{colors.charcoal}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    width: "160px"
    height: "35px"
    padding: "8px 14px"
  button-play:
    backgroundColor: "{colors.sage}"
    textColor: "{colors.charcoal}"
    rounded: "{rounded.media-tile}"
    size: "44px"
    padding: "12px"
  button-icon:
    textColor: "{colors.muted-sage}"
    rounded: "{rounded.control}"
    size: "32px"
    padding: "6px"
  button-icon-active:
    backgroundColor: "{colors.raised-charcoal}"
    textColor: "{colors.sage}"
    rounded: "{rounded.control}"
    size: "32px"
  tab:
    textColor: "{colors.muted-sage}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    height: "34px"
  tab-active:
    backgroundColor: "{colors.raised-charcoal}"
    textColor: "{colors.sage}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    height: "34px"
  duration-selected:
    backgroundColor: "{colors.raised-charcoal}"
    textColor: "{colors.sage}"
    rounded: "{rounded.control}"
    padding: "8px 14px"
  progress-track:
    backgroundColor: "{colors.raised-charcoal}"
    rounded: "{rounded.progress}"
    height: "3px"
  media-cover:
    backgroundColor: "{colors.sage}"
    textColor: "{colors.charcoal}"
    rounded: "{rounded.media-tile}"
    size: "52px"
    padding: "15px"
  setting-checkbox:
    textColor: "{colors.ivory}"
    typography: "{typography.body}"
---

# Design System: Sela

## Overview

**Creative North Star: "Sedikit ruang untukmu"**

Sela is a calm companion at the top of the desktop. Its charcoal silhouette gives controls a quiet home; sage marks the available action and warm amber gives the current lyric a small glow. Pijar, the approved firefly mascot, brings warmth through one restrained identity asset.

The selected visual world is already established. The Operate direction extends the working native overlay with the approved Canvas music view while retaining focus, system status, and settings. This document records the implemented Rust/egui system; portable tokens use CSS pixels to represent egui logical points.

**Key Characteristics:**

- A top-center charcoal silhouette with a compact resting state.
- Sage actions, ivory content, and an amber current lyric.
- DM Sans Regular and Semibold, with retained monospace timer and metric values.
- Pijar as a small identity cue beside practical native controls.
- Stable content geometry, continuous spring motion, and truthful media states.

## Colors

The palette combines green-tinted charcoal with a soft sage action color and a warm amber reading accent. Frontmatter owns the normative values.

### Primary

- **Sage:** primary play/pause and focus actions, note cover, active tabs, pinned controls, progress fill, and positive focus state.

### Secondary

- **Amber:** the current lyric in compact and expanded music views, plus the unsynced-lyrics explanation.

### Neutral

- **Charcoal:** opaque shell and dark symbols on sage actions and the note cover.
- **Raised Charcoal:** selected and hovered custom controls, progress tracks, and the quiet column divider.
- **Control Hover:** hover fill for egui controls using the configured widget visuals.
- **Ivory:** primary content, track title, timer, and measured values.
- **Muted Sage:** secondary labels, inactive icons, surrounding lyric lines, and unavailable information.
- **Error Coral:** media and lyric errors and the footer attention state.

**The Two Accents Rule.** Sage identifies action and active state; amber identifies the current lyric. Preserve those roles when adding media controls or lyric states.

## Typography

The proportional UI embeds **DM Sans Regular (400)** and **DM Sans Semibold (600)** from `assets/dm-sans-400.ttf` and `assets/dm-sans-600.ttf`, licensed in `assets/OFL.txt`. The Sela heading, track title, and emphasized lyric labels explicitly select the semibold font family. The retained focus timer and system values use **Hack Regular**, bundled by eframe's default fonts. `RichText::strong()` alone changes emphasis color; it does not select the DM Sans Semibold face.

DM Sans keeps small labels open and friendly. The larger current lyric creates the music view's reading anchor; the numerical focus display remains its own functional anchor. Native egui controls retain their line layout. Do not infer a global line-height or tracking scale from browser previews.

### Hierarchy

- **Display:** the central monospace focus timer.
- **Title:** the semibold Sela header.
- **Track title / current lyric:** semibold emphasis in the two-column player.
- **Body:** navigation, compact title and lyric, supporting lyrics, and general controls.
- **Metric:** right-aligned monospace system readings.
- **Caption:** media timestamps, settings disclosure, and secondary details.
- **Footer:** status labels, source notes, playback state, and quiet shell footer.

**The Real Weight Rule.** Select the embedded semibold family for semibold content; keep the actual font choice explicit in native code.

## Layout

All native sizes are logical points and scale with DPI. Center the fixed backing viewport on the selected monitor and apply the configured top offset (0–80 points). The drawn silhouette changes size within that viewport; Win32 clips drawing and pointer input to its current bounds. Linux previews retain the full backing input canvas. If native region setup fails, the application reports the error and uses its resize fallback.

| Mode | Drawn bounds | Content |
| --- | --- | --- |
| Idle, active focus, or lyrics disabled | 324 × 48 | Pijar or mark, clock/title/focus status, available battery status |
| Track present, lyrics enabled, no active focus | 580 × 54 | Pijar or mark, title left, current lyric right |
| Expanded Media / backing viewport | 600 × 370 | Player and contextual lyrics |
| Expanded Fokus, Sistem, Atur | 464 × 370 | Existing task panel |

Expanded views share content insets, a header, four equal-width tabs separated by the tab-gap token, a task area, and an anchored footer. Tab widths adapt to the panel's available width; they are not a single fixed token. The settings scroll area is bounded to 207 points. These are native modes, not web breakpoints.

The Canvas media body is 552 points wide after the shell insets. The player occupies 226 points, the lyrics occupy 298, and the columns are separated by 28. A thin tonal divider sits midway in the gap. Both regions allocate 208 points of body height. The player uses a sage note tile, truncated title and artist, playback label, progress, timestamps, and transport controls. Synced lyrics show previous, current, and next lines. Each lyric allocates a full-width row and paints one clipped line at the shared fixed left edge; its tooltip and semantic label retain the full text. Plain unsynced lyrics use a vertical scroll area.

Compact music keeps the title left-aligned in a clipped left region, separated from the single amber line by a short divider. Center the lyric horizontally within its own right-hand region after single-line truncation; preserve clipping and the line-change fade/offset. It retains the music geometry while paused. An active or paused focus session takes compact priority. The idle pill uses the local clock when there is no media title.

## Elevation & Depth

The shell is opaque charcoal over a transparent native window. No custom shell shadow or background blur is painted. Tonal surfaces and spacing give controls depth without separating them into decorative cards. Native egui tooltips, popups, checkboxes, sliders, and combo boxes retain the framework's depth behavior; they are not a separate authored shadow vocabulary.

**The Tonal Surface Rule.** Use raised charcoal and spacing to distinguish controls before adding any new elevation effect.

## Shapes

At zero top offset the shell meets the monitor edge with square upper corners and rounded lower corners. A positive top offset applies the floating-top radius. Shared controls have the control radius; the media note cover and primary play/pause button use the slightly rounder media-tile radius. Thin progress strips use the progress radius.

Authored vector icons use a consistent line stroke (1.65 points), with filled transport symbols and note heads. Icon controls place a 20-point icon in a 32-point target; tabs use 17-point icons. The 44-point primary play/pause control contains a 20-point dark icon. The equalizer fallback mark uses three 3-point strokes.

Pijar is the approved soft sculpted firefly: warm ivory body, sage wings, and amber light. Keep the original transparent source sheet intact. Rust uploads only the first 30% of its width, excluding the archived concepts and keeping the texture below the GL 2.0 minimum dimension limit. Render that crop in the compact and header identity regions. Hiding the mascot or failing to decode it uses the authored equalizer mark.

## Components

### Notch shell and motion

The shell opens as a small instrument coming into use. Independent width and height dimensions use exact critically damped springs (opening response 32/s; closing response 38/s). Retargeting preserves position and velocity. The normalized expansion spring drives content opacity; compact content fades out over progress 0.02–0.30 and expanded content fades in over 0.20–0.92 using smoothstep. Expanded content keeps its final layout width under the growing mask and reveals through a 6-point vertical offset. Bounds are aligned to physical pixels.

Reduced motion snaps dimensions, reveal offset, and opacity to their destinations and disables egui widget interpolation. Ordinary widget interpolation lasts 200 ms. Hover opening waits 240 ms; an unpinned hover-opened panel closes 650 ms after pointer exit. Clicking, keyboard activation, tray commands, or the global shortcut can open the shell. Esc collapses it.

**The Continuous Motion Rule.** Preserve spring position and velocity when a target changes; never restart expansion from a preset endpoint.

### Navigation

Media, Fokus, Sistem, and Atur share one compact tab row. Selected tabs use sage text and icons on raised charcoal; hover and keyboard focus also reveal the raised surface. Keyboard focus adds a sage inner outline (1 point). Header-wheel navigation cycles the four pages after a 32-point accumulated scroll threshold, with a 220 ms cooldown and 250 ms gesture expiry. Leaving the header discards partial input; content scrolling remains local.

### Buttons and duration choices

The main play/pause control is a sage square with rounded corners and a dark symbol. Hover darkens sage through the native gamma multiplier (0.86); keyboard focus adds an ivory outer outline (1 point) two points beyond the button. Previous/next, pin, collapse, and reset use the shared icon control. Active icon controls use sage on raised charcoal; hover supplies the raised fill; keyboard focus adds a sage outline. Disabled icon controls dim muted sage through the native gamma multiplier (0.35).

The focus action uses sage with charcoal text. Duration choices use sage on raised charcoal when selected and cannot change during a running or paused session. Settings retain the native egui checkbox, slider, and combo-box interactions and include separate controls for Pijar, lyrics, and reduced motion.

### Accessibility semantics

The eframe AccessKit feature is enabled to export egui widget names through the native platform accessibility backend. Custom controls declare their names, roles, and enabled state through `widget_info`; painted lyric rows declare the full text as a Label. Keep these semantics when changing custom painting, alongside visible keyboard focus and tooltips. Windows Narrator behavior remains runtime-unverified; browser component previews do not validate that integration.

### Media and lyrics

The media player uses the authored note icon on the sage cover tile; it does not imply fetched album artwork. Current lyrics use amber and the explicit semibold face, while previous and next lines use muted sage. On a line change, amber text fades from 40% to full intensity over 240 ms; compact text also rises by 2 points. Reduced motion applies the new line immediately. Expanded lines remain left aligned throughout.

Playback time starts from the latest system position plus monotonic elapsed time while playing and is clamped to the known duration. Pause holds the position and line; seeking or a new system position recalculates the active lyric index. The asynchronous LRCLIB worker keeps networking away from rendering and rejects stale results. Synced lyrics, plain unsynced lyrics, instrumental, loading, missing metadata, not found, error, and disabled states each state what is available. Error and not-found states offer retry; source notes identify LRCLIB.

Normal launch uses real Windows sessions and explanatory empty states. Explicit demo mode labels original sample data and states that its lyrics have no audio. Previous/Next cycles five local fixtures: synced lyrics, long text, not found, plain lyrics, and instrumental. Demo makes no LRCLIB requests, including retry.

### Focus and system readings

The focus timer remains centered and monospace, with shared progress and duration choices. System readings pair muted labels with right-aligned monospace values. CPU and RAM sampling activates only while the System page is open, updates every two seconds, and exposes a short baseline-loading state. Progress fill stays within the track. Unavailable battery data is labelled explicitly.

### Portable component previews

`.impeccable/design.json` contains self-contained HTML/CSS illustrations of native controls, compact music, and lyric context. These aid design tooling and approximate egui rendering; they do not replace Rust widgets. Browser previews fall back to installed sans-serif/monospace fonts. The compact preview uses the real equalizer fallback geometry because the native Pijar raster remains a project asset. Generated tonal ramps are inspection aids, not additional application colors.

## Do's and Don'ts

### Do:

- **Do** preserve Sela, Pijar, and the top-center charcoal silhouette as the selected identity.
- **Do** use sage for primary actions and active state, and amber for the current lyric.
- **Do** use the embedded DM Sans faces explicitly and retain monospace numerical anchors.
- **Do** keep Canvas player and lyric columns stable, with lyrics left aligned.
- **Do** retain keyboard focus, tooltips, reduced motion, and clear unavailable states.
- **Do** distinguish real Windows data from explicitly labelled original demo content.

### Don't:

- **Don't** reintroduce archived mascot concepts as Sela identity assets.
- **Don't** replace the authored note cover with artwork that was never fetched.
- **Don't** restart interrupted springs or reflow text against every animated width.
- **Don't** imply that plain lyrics are synced or that a missing lyric result stops playback.
- **Don't** treat Linux previews as evidence that Windows integrations or input clipping have been exercised.
