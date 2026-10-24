# Luma Notch

<!-- impeccable:product-schema 1 -->

## Platform
adaptive

Native Windows desktop. Linux is a development preview, not a supported release target.

## Stack
Rust, eframe/egui, Windows WinRT and Win32 APIs. The user explicitly requested Rust and Windows; the GUI library is an implementation choice.

## Product Purpose
A Mac-inspired notch overlay for Windows, delivered as a Git repository with a commit.

## Brand Commitments
The user's reference is the Apple Mac notch. Preserve its quiet black silhouette and top-center placement.

## Capabilities and Constraints
Implementation assumptions: media transport through Windows system media sessions, a focus timer, live CPU/RAM/battery, persistent local settings, tray recovery and a global shortcut. No account or network service. Indonesian UI and documentation follow the user's language. These feature choices are inferred, not separately confirmed user requirements.

## Operating Context
Windows 10 version 1809 or newer / Windows 11, desktop session, multiple DPI scales and monitors. Settings belong to the current user. Media requires a player that exposes a Windows media session.
