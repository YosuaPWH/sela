# Validasi

Dijalankan pada 24 Oktober 2026 di Linux x64 dengan Rust 1.99.1. Build Windows menggunakan target `x86_64-pc-windows-gnu` dan GCC MinGW 14-posix. CI Windows MSVC disiapkan, belum dijalankan dari lingkungan ini.

## Pemeriksaan otomatis

| Pemeriksaan | Hasil |
| --- | --- |
| `cargo fmt --all -- --check` | Lulus |
| `cargo clippy --locked --all-targets -- -D warnings` | Lulus |
| `cargo clippy --locked --target x86_64-pc-windows-gnu --all-targets -- -D warnings` | Lulus |
| `cargo test --locked --all-targets` | 8 tes lulus |
| `cargo build --locked --release --target x86_64-pc-windows-gnu` | Lulus; executable PE32+ x64, subsystem Windows GUI |

Tes mencakup jeda/lanjut timer, penyelesaian tepat satu kali setelah durasi terlampaui, reset sesi aktif, default kompatibel pengaturan lama, pembatasan nilai, penanganan JSON rusak tanpa menimpa file, serta penyimpanan dan penggantian pengaturan.

Executable diperiksa dengan `objdump -p`: hanya mengimpor DLL sistem Windows, tanpa kebutuhan DLL MinGW terpisah. Ini pemeriksaan statis, bukan bukti runtime Windows.

## Pemeriksaan animasi 0.1.1

- Spring memakai solusi analitik terhadap durasi frame; lintasan diuji sama pada simulasi 30/60/144 Hz. Ini bukan pengukuran FPS pada perangkat Windows.
- Retarget di tengah gerakan menjaga posisi/kecepatan dan tetap dalam batas 0–1; idle panjang tidak melewatkan pembukaan; mode kurangi animasi menghentikan gerakan dan menyetel opacity akhir.
- Window Windows tetap 464×370 logical points; `SetWindowRgn` mengikuti batas notch untuk menghindari resize/recenter HWND tiap frame dan membatasi area input. Pengaturan region hanya diperbarui saat batas fisik berubah; kegagalan memakai mode kompatibilitas.
- Isi panel memakai layout lebar tetap, fade, dan clipping; pembaruan frame diminta terus hanya selama spring aktif. Timer hover dijadwalkan ke deadline dan dibersihkan setelah transisi.
- Pratinjau Linux dengan xcompmgr menangkap 58 tinggi antara posisi ringkas dan terbuka. Pusat siluet bergeser paling banyak 1 physical pixel dalam capture uji.
- `docs/animation.gif` berasal dari rekaman langsung X11, termasuk pembalikan arah. Encoding 60 fps adalah konfigurasi video, bukan jaminan frame rate aplikasi.
- Mode kurangi animasi, buka/tutup dengan keyboard dan hover, empat tab, kontrol timer, dan keluar aplikasi dicoba kembali. `Esc` lalu `Space` saat transisi tidak menjeda timer yang aktif; `Tab`/`Space` dari mode ringkas tetap membuka panel.

## Pratinjau interaktif Linux

Aplikasi native eframe dijalankan dengan renderer OpenGL perangkat lunak di Xvfb 1440×900; capture 0.1.1 memakai compositor xcompmgr. Capture berasal dari framebuffer aplikasi yang berjalan, bukan mockup. `WINIT_X11_SCALE_FACTOR=1.5` dipakai untuk capture 150%.

- Empat tab, timer berjalan, state jeda, dan reset dicoba.
- Digit timer tetap sama selama jeda; keluar aplikasi lewat Atur berhasil.
- `Esc` menyusutkan siluet notch ke 324×48; backing window tetap 464×370 pada jalur normal.
- Klik mempertahankan panel 464×370 setelah kursor keluar.
- Hover membuka panel; keluar kursor menutup panel setelah penundaan.
- Perubahan pengaturan tersimpan pada folder konfigurasi sementara yang terisolasi.
- Tampilan 100% dan 150% diperiksa tanpa clipping pada state yang dicapture.
- Review terpisah versi 0.1.0 memberi disposition **ship** pada ruang lingkup pratinjau UI Linux dan kode yang diperiksa. Perubahan gerak 0.1.1 diperiksa melalui capture dan uji interaksi di atas; integrasi Windows tetap di luar verifikasi runtime.

Capture di `screenshots/`: `media.png`, `focus.png`, `focus-running.png`, `focus-paused.png`, `system.png`, `settings.png`, `compact.png`, `focus-150.png`, `settings-150.png`. Nilai sistem adalah data langsung mesin pengembangan; media kosong karena integrasi Windows tidak aktif pada Linux.

## Checklist runtime Windows yang belum dijalankan

Jalankan pada perangkat Windows 10 1809+ atau Windows 11 dengan driver OpenGL yang mendukung:

- [ ] Notch berada di tengah atas layar, sudut transparan, dan tidak muncul di taskbar.
- [ ] Animasi tetap mulus di perangkat Windows; area transparan di luar notch meneruskan klik ke aplikasi di bawahnya, termasuk saat transisi dan perubahan DPI.
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
