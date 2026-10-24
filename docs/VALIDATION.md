# Validasi

Dijalankan pada 24 Oktober 2026 di Linux x64 dengan Rust 1.99.1. Build Windows menggunakan target `x86_64-pc-windows-gnu` dan GCC MinGW 14-posix. CI Windows MSVC disiapkan, belum dijalankan dari lingkungan ini.

## Pemeriksaan otomatis

| Pemeriksaan | Hasil |
| --- | --- |
| `cargo fmt --all -- --check` | Lulus |
| `cargo clippy --locked --all-targets -- -D warnings` | Lulus |
| `cargo clippy --locked --target x86_64-pc-windows-gnu --all-targets -- -D warnings` | Lulus |
| `cargo test --locked --all-targets` | 4 tes lulus |
| `cargo build --locked --release --target x86_64-pc-windows-gnu` | Lulus; executable PE32+ x64, subsystem Windows GUI |

Tes mencakup jeda/lanjut timer, penyelesaian tepat satu kali setelah durasi terlampaui, reset sesi aktif, default kompatibel pengaturan lama, pembatasan nilai, penanganan JSON rusak tanpa menimpa file, serta penyimpanan dan penggantian pengaturan.

Executable diperiksa dengan `objdump -p`: hanya mengimpor DLL sistem Windows, tanpa kebutuhan DLL MinGW terpisah. Ini pemeriksaan statis, bukan bukti runtime Windows.

## Pratinjau interaktif Linux

Aplikasi native eframe dijalankan dengan renderer OpenGL perangkat lunak di Xvfb 1440×900. Capture berasal dari framebuffer aplikasi yang berjalan, bukan mockup. `WINIT_X11_SCALE_FACTOR=1.5` dipakai untuk capture 150%.

- Empat tab, timer berjalan, state jeda, dan reset dicoba.
- Digit timer tetap sama selama jeda; keluar aplikasi lewat Atur berhasil.
- `Esc` menyusutkan window ke 324×48.
- Klik mempertahankan panel 464×370 setelah kursor keluar.
- Hover membuka panel; keluar kursor menutup panel setelah penundaan.
- Perubahan pengaturan tersimpan pada folder konfigurasi sementara yang terisolasi.
- Tampilan 100% dan 150% diperiksa tanpa clipping pada state yang dicapture.
- Review terpisah memberi disposition **ship** pada ruang lingkup pratinjau UI Linux dan kode yang diperiksa. Integrasi Windows berada di luar ruang lingkup review tersebut.

Capture di `screenshots/`: `media.png`, `focus.png`, `focus-running.png`, `focus-paused.png`, `system.png`, `settings.png`, `compact.png`, `focus-150.png`, `settings-150.png`. Nilai sistem adalah data langsung mesin pengembangan; media kosong karena integrasi Windows tidak aktif pada Linux.

## Checklist runtime Windows yang belum dijalankan

Jalankan pada perangkat Windows 10 1809+ atau Windows 11 dengan driver OpenGL yang mendukung:

- [ ] Notch berada di tengah atas layar, sudut transparan, dan tidak muncul di taskbar.
- [ ] Ikon tray membuka menu; semua tindakan termasuk keluar berfungsi.
- [ ] `Ctrl+Alt+N` bekerja dari aplikasi lain; konflik shortcut memberi pesan di Atur.
- [ ] Instance kedua tidak membuat overlay tambahan.
- [ ] Spotify/pemutar lain menampilkan metadata aktual; play/pause dan skip bekerja sesuai kemampuan pemutar.
- [ ] Media berganti sesi dengan benar saat pemutar ditutup; kegagalan akses menyediakan Coba lagi.
- [ ] Timer selesai, membuka tab Fokus dan berbunyi sesuai pengaturan.
- [ ] Baterai cocok dengan status perangkat; desktop tanpa baterai menampilkan Tidak tersedia.
- [ ] Monitor kedua, DPI 100/125/150/200%, koordinat monitor negatif, serta pencabutan monitor tetap menempatkan window dalam layar.
- [ ] Pengaturan tetap tersimpan setelah restart.
- [ ] Fullscreen, remote desktop, dan sleep/resume diperiksa sesuai lingkungan penggunaan.

Checklist ini mencatat pekerjaan verifikasi perangkat yang belum tersedia; tidak menyatakan integrasi Windows sudah diuji langsung.
