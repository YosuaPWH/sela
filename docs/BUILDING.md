# Build dan pengembangan

[Kembali ke README](../README.md)

## Windows

Pasang [Rust stable](https://rustup.rs/) untuk target MSVC, **Visual Studio Build Tools** dengan workload **Desktop development with C++**, dan Windows SDK:

```powershell
cargo build --locked --release
.\target\release\luma-notch.exe
```

Gunakan `--expanded` untuk langsung membuka panel. Aplikasi memakai renderer OpenGL melalui eframe, tanpa WebView atau runtime JavaScript.

## Pemeriksaan

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
```

Tes memeriksa state timer, penyimpanan konfigurasi, kontinuitas spring, lintasan pada refresh rate berbeda, akumulasi input trackpad, serta hidup/matinya pembacaan CPU/RAM sesuai halaman.

## Cross-build dari Linux

Pasang MinGW-w64 (`gcc-mingw-w64-x86-64` pada Debian/Ubuntu), lalu:

```bash
rustup target add x86_64-pc-windows-gnu
cargo clippy --locked --target x86_64-pc-windows-gnu --all-targets -- -D warnings
cargo build --locked --release --target x86_64-pc-windows-gnu
```

Hasilnya `target/x86_64-pc-windows-gnu/release/luma-notch.exe`. Untuk compiler di lokasi nonstandar, isi `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER` dengan path compiler dan tambahkan toolchain tersebut ke `PATH` agar `dlltool` ditemukan.

Cross-build membuktikan executable dapat dibuat; ia tidak menggantikan uji integrasi Windows. Checklist ada di [VALIDATION.md](VALIDATION.md).

## Pratinjau Linux

Pratinjau memerlukan X11 dan OpenGL. Contoh paket Debian/Ubuntu: `libxkbcommon-dev libx11-dev libxi-dev libgl1-mesa-dev`.

```bash
cargo run --locked -- --expanded
```

Media WinRT, tray, shortcut global, mutex, pembacaan baterai Windows, dan pembatasan area input native dinonaktifkan. Backing window pratinjau tetap berukuran penuh; jangan gunakan pratinjau sebagai bukti click-through Windows.

Capture dokumentasi memakai Xvfb dengan compositor xcompmgr. `WINIT_X11_SCALE_FACTOR=1.5` menghasilkan pratinjau 150%. GIF memperlihatkan animasi 0.1.1 yang dipertahankan di 0.1.2; galeri 0.1.2 memperlihatkan UI dan data mesin pengembangan yang berjalan.

## Struktur kode

| Berkas | Tanggung jawab |
| --- | --- |
| `src/app.rs` | Panel, input, state halaman, dan koordinasi worker |
| `src/navigation.rs` | Batas akumulasi wheel dan jeda perpindahan halaman |
| `src/motion.rs` | Spring analitik dan perpindahan opacity |
| `src/focus.rs` | State machine timer berbasis durasi monotonik |
| `src/config.rs` | JSON, default, validasi, dan penyimpanan pengaturan |
| `src/telemetry.rs` | Worker CPU/RAM sesuai kebutuhan dan pemeriksaan baterai |
| `src/platform/windows.rs` | Media WinRT, tray, shortcut, monitor, window region, dan mutex |
| `src/icons.rs` | Ikon vektor dan logo UI |

UI mengaktifkan CPU/RAM hanya ketika panel terbuka di halaman Sistem. Worker membuat baseline CPU baru, menunggu interval sampling minimum, lalu mengirim pembacaan setiap 2 detik. Menutup halaman menghentikannya; pemeriksaan baterai berlanjut setiap 30 detik untuk tampilan ringkas. Pembacaan berjalan pada worker terpisah.

Saat beranimasi, aplikasi meminta frame terus dan memakai vsync. Saat diam, UI masih memeriksa event integrasi setiap 250 ms dan worker media memeriksa sesi setiap 750 ms. **Belum ada klaim zero idle CPU** atau benchmark FPS Windows.

## CI dan paket portable

[Workflow Build and test](../.github/workflows/ci.yml) memiliki dua job:

- **windows:** format, Clippy, tes, build MSVC, lalu ZIP portable berisi executable, README, lisensi, dan dokumentasi.
- **linux-core:** tes logika dan dependensi Linux.

Workflow berjalan pada push, pull request, dan eksekusi manual. Artifact bernama **LumaNotch-windows-x64**; tidak ada publikasi release otomatis.
