# Build dan pengembangan

[Kembali ke README](../README.md)

## Windows

Pasang [Rust stable](https://rustup.rs/) untuk target MSVC, **Visual Studio Build Tools** dengan workload **Desktop development with C++**, dan Windows SDK:

```powershell
cargo build --locked --release
.\target\release\sela-notch.exe
```

Gunakan `--expanded` untuk langsung membuka panel; `--demo --expanded` mencoba lima track contoh tanpa audio atau jaringan. Aplikasi memakai renderer OpenGL melalui eframe, tanpa WebView atau runtime JavaScript.

## Pemeriksaan

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
```

Tes memeriksa state timer, penyimpanan konfigurasi, kontinuitas spring, lintasan pada refresh rate berbeda, akumulasi input trackpad, hidup/matinya pembacaan CPU/RAM sesuai halaman, parser LRC, fallback provider HTTP, penolakan hasil lama, cache, dan interpolasi posisi media. Tes provider memakai server lokal; tidak bergantung pada LRCLIB.

## Cross-build dari Linux

Pasang MinGW-w64 (`gcc-mingw-w64-x86-64` pada Debian/Ubuntu), lalu:

```bash
rustup target add x86_64-pc-windows-gnu
cargo clippy --locked --target x86_64-pc-windows-gnu --all-targets -- -D warnings
cargo build --locked --release --target x86_64-pc-windows-gnu
```

Hasilnya `target/x86_64-pc-windows-gnu/release/sela-notch.exe`. Untuk compiler di lokasi nonstandar, isi `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER` dan `CC_x86_64_pc_windows_gnu` dengan path compiler, lalu tambahkan toolchain tersebut ke `PATH` agar `dlltool` ditemukan. `ring` (TLS) memerlukan compiler C target Windows.

Cross-build membuktikan executable dapat dibuat; ia tidak menggantikan uji integrasi Windows. Checklist ada di [VALIDATION.md](VALIDATION.md).

## Pratinjau Linux

Pratinjau memerlukan X11 dan OpenGL. Contoh paket Debian/Ubuntu: `libxkbcommon-dev libx11-dev libxi-dev libgl1-mesa-dev`.

```bash
cargo run --locked -- --demo --expanded
```

Media WinRT, tray, shortcut global, mutex, pembacaan baterai Windows, dan pembatasan area input native dinonaktifkan. Backing window pratinjau tetap berukuran penuh; jangan gunakan pratinjau sebagai bukti click-through Windows.

Capture Sela memakai Xvfb dengan compositor xcompmgr. `WINIT_X11_SCALE_FACTOR=1.5` menghasilkan pratinjau 150%. Mode demo menyediakan lirik tersinkron, string panjang, lirik kosong, plain text, dan instrumental. Track serta lirik demo orisinal dan tidak mengeluarkan audio. GIF lama `docs/animation.gif` adalah arsip Luma 0.1.1, bukan rekaman Sela 0.2.0.

## Struktur kode

| Berkas | Tanggung jawab |
| --- | --- |
| `src/app.rs` | Panel, input, state halaman, dan koordinasi worker |
| `src/lyrics.rs` | Parser LRC, LRCLIB HTTPS, pencocokan, cache, dan worker yang menolak hasil usang |
| `src/navigation.rs` | Batas akumulasi wheel dan jeda perpindahan halaman |
| `src/motion.rs` | Spring analitik dan perpindahan opacity |
| `src/focus.rs` | State machine timer berbasis durasi monotonik |
| `src/config.rs` | JSON, default, validasi, dan penyimpanan pengaturan |
| `src/telemetry.rs` | Worker CPU/RAM sesuai kebutuhan dan pemeriksaan baterai |
| `src/platform/windows.rs` | Media WinRT, tray, shortcut, monitor, window region, dan mutex |
| `src/icons.rs` | Ikon vektor dan logo UI |

UI mengaktifkan CPU/RAM hanya ketika panel terbuka di halaman Sistem. Worker membuat baseline CPU baru, menunggu interval sampling minimum, lalu mengirim pembacaan setiap 2 detik. Menutup halaman menghentikannya; pemeriksaan baterai berlanjut setiap 30 detik untuk tampilan ringkas. Pembacaan berjalan pada worker terpisah.

Saat beranimasi, aplikasi meminta frame terus dan memakai vsync. Saat musik berjalan, UI memperbarui posisi/lirik setiap 100 ms; saat diam, event integrasi diperiksa setiap 250 ms. Worker media memeriksa sesi setiap 750 ms. Worker lirik mencari hanya saat track berubah, lirik diaktifkan kembali, atau pengguna mencoba lagi. **Belum ada klaim zero idle CPU** atau benchmark FPS Windows.

## CI dan paket portable

[Workflow Build and test](../.github/workflows/ci.yml) memiliki dua job:

- **windows:** format, Clippy, tes, build MSVC, lalu ZIP portable berisi executable, README, lisensi, dokumentasi produk/desain, serta aset dan lisensi font.
- **linux-core:** tes logika dan dependensi Linux.

Workflow berjalan pada push, pull request, dan eksekusi manual. Artifact bernama **Sela-windows-x64**; tidak ada publikasi release otomatis.

## Memeriksa koneksi lirik

Contoh ini mengirim metadata ke LRCLIB melalui HTTPS dan mencetak jumlah baris, tanpa menampilkan isi lirik:

```bash
cargo run --locked --example lyrics_probe -- "Never Gonna Give You Up" "Rick Astley" 213
```

Koneksi dibatasi 4 detik, tiap respons maksimal 1 MiB dan 10 detik. Pencarian tepat dapat diikuti pencarian fallback; keseluruhan lookup dapat memerlukan dua permintaan. Cache lirik 30 hari, hasil kosong 1 hari. HTTP error tidak di-cache. Worker mengabaikan hasil usang dan tidak memulai permintaan berikutnya setelah pembatalan; permintaan blocking yang sudah berjalan akan menyelesaikan koneksinya. TLS memakai trust store sistem melalui rustls; jangan menonaktifkan verifikasi sertifikat.
