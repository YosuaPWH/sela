# Aset Sela

- `pijar-sheet.png`: lembar PNG generatif dari eksplorasi maskot proyek Sela, dibuat dengan OpenAI image generation pada 24 Oktober 2026. Aset asal disalin dari preview Sela yang disetujui pengguna. Bagian kiri (30% lebar) adalah Pijar; dua konsep lain di lembar ini diarsipkan dan tidak dirender aplikasi. Pemotongan dilakukan saat decode di Rust agar tekstur GPU tetap di bawah batas OpenGL 2.0. PNG mempertahankan metadata asal; prompt lengkap berada di `pijar.prompt.txt`.
- `dm-sans-400.ttf`, `dm-sans-600.ttf`: DM Sans regular dan semibold, diperoleh dari distribusi Google Fonts pada eksplorasi preview. Lisensi SIL Open Font License ada di `OFL.txt`. Font di-embed ke executable.

Screenshot dalam `docs/screenshots/sela-*` adalah capture langsung aplikasi Rust di Linux/X11. Data musik/liriknya merupakan fixture demo orisinal, bukan hasil pengambilan lirik komersial. Tidak ada lirik dari LRCLIB yang dibundel dalam paket distribusi.
