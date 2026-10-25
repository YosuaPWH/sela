# Sela assets

- `pijar-sheet.png`: a generated PNG sheet from Sela's mascot exploration, created with OpenAI image generation on October 24, 2026. The source asset was copied from the Sela preview approved by the user. The leftmost 30% contains Pijar; the other two concepts are archived and are not rendered by the app. Rust crops the image during decoding to keep the GPU texture within the OpenGL 2.0 minimum texture-size limit. The PNG retains its source metadata; the full generation prompt is in `pijar.prompt.txt`.
- `dm-sans-400.ttf`, `dm-sans-600.ttf`: DM Sans Regular and Semibold, obtained from the Google Fonts distribution during the preview exploration. The SIL Open Font License is included in `OFL.txt`. Both fonts are embedded in the executable.

Images in `docs/screenshots/sela-*` are direct captures of the Rust app running on Linux/X11. Music and lyrics use original demo fixtures rather than retrieved commercial lyrics. No lyrics fetched from LRCLIB are bundled with the distribution.
