# Validasi Sela 0.2.0

Dijalankan pada 24 Oktober 2026 di Linux x64, Rust 1.99.1. Cross-build Windows memakai `x86_64-pc-windows-gnu` dan MinGW GCC 14-posix. CI Windows MSVC tersedia, belum dijalankan dari lingkungan ini.

## Pemeriksaan otomatis

| Pemeriksaan | Hasil |
| --- | --- |
| `cargo fmt --all -- --check` | Lulus |
| `cargo clippy --locked --all-targets -- -D warnings` | Lulus |
| `cargo clippy --locked --target x86_64-pc-windows-gnu --all-targets -- -D warnings` | Lulus |
| `cargo test --locked --all-targets` | 22 tes lulus |
| `cargo build --locked --release --target x86_64-pc-windows-gnu` | Lulus; PE32+ x64, subsystem Windows GUI |

Tes mencakup timer, konfigurasi lama/rusak, parser LRC dengan timestamp berulang dan offset, indeks lirik saat seek mundur, pemisahan plain/synced, pencocokan artis/durasi, pengabaian hasil usang, pembatalan sebelum jaringan, cache korup/kedaluwarsa/salah track, URL encoding, serta fallback HTTP 404→search. Provider dites melalui server HTTP lokal.

Posisi media diuji tetap ketika dijeda, maju dengan waktu monotonik, menerima seek mundur, membatasi durasi, dan menangani nilai tidak valid. Spring diuji pada simulasi 30/60/144 Hz serta retarget dan mode kurangi animasi. Ini pengujian matematika gerak, bukan benchmark FPS Windows.

## Koneksi nyata

`cargo run --locked --example lyrics_probe -- "Never Gonna Give You Up" "Rick Astley" 213` berhasil mengakses LRCLIB lewat HTTPS dan menerima satu baris tersinkron pada data sumber saat pengujian. Jumlah/isi respons berasal dari provider dan dapat berubah. Isi lirik komersial tidak dimasukkan ke galeri atau paket distribusi. Pengujian ini membuktikan jalur HTTPS dari Linux, bukan kualitas katalog LRCLIB atau konektivitas Windows.

## Pratinjau native

Aplikasi Rust eframe dijalankan di Xvfb 1440×900 dengan OpenGL software dan compositor xcompmgr. Capture Sela memakai skala 100% dan 150%; track demo diberi label dan memakai lirik orisinal tanpa audio/jaringan.

- Panel Media dan notch ringkas dicapture pada 100%/150%: 600×370 dan 580×54 logical points.
- Lirik ringkas saat jeda menghasilkan framebuffer yang sama selama dua detik.
- Track demo mencakup lirik tersinkron, judul/baris panjang dengan elipsis, lirik kosong, plain text, dan instrumental.
- Sakelar lirik memakai pill pendek 324×48; kurangi animasi menampilkan geometri akhir. Pengaturan tersimpan dan dibaca kembali saat restart.
- Fokus, Sistem, Atur, dan keadaan media kosong dicapture; CPU/RAM adalah data mesin pengembangan.
- Lebar dan tinggi memakai spring terpisah; permukaan native tetap 600×370 dan isi panel dilayout pada lebar akhir. Pijar di-decode dari 30% kiri lembar aset agar tidak melampaui batas minimum tekstur GL 2.0.

Galeri terkini berada di `screenshots/sela-*`. Screenshot merupakan bukti UI Linux. Media WinRT, tray, global hotkey, mutex, baterai Windows, dan area input native tidak aktif pada Linux.

AccessKit diaktifkan dalam build agar nama dan peran widget diekspor ke aksesibilitas native. Baris lirik memakai galley satu baris pada tepi kiri yang tetap, dengan teks lengkap di tooltip dan informasi semantik.

## Pemeriksaan akhir

Review terpisah menemukan alignment lirik dan fitur ekspor aksesibilitas yang belum aktif. Kedua perbaikan dinilai **resolved**, dengan disposition **ship** pada cakupan implementasi/pratinjau native yang diperiksa. Verdict terakhir hanya memeriksa dua koreksi itu, bukan sertifikasi integrasi Windows. Capture tersinkron, teks panjang, dan fallback diperbarui pada 100%/150%; `cargo tree` mengonfirmasi AccessKit dan executable mengimpor `uiautomationcore.dll`. Pemeriksaan statis import DLL menunjukkan hanya DLL sistem Windows, tanpa DLL MinGW terpisah.

## Penyesuaian lirik ringkas — 25 Oktober 2026

Lirik saat panel ditutup kini rata tengah di area kanan judul; judul dan lirik pada panel terbuka tetap rata kiri. Capture native Linux pada 100%/150% memeriksa teks pendek, teks panjang dengan elipsis, dan pesan lirik belum tersedia. Titik tengah piksel teks berada dalam 0,5 physical pixel dari titik tengah area lirik pada enam capture; tidak ada teks melewati pembatas. Clipping, fade 240 ms, dan pergeseran vertikal tetap dipertahankan. `cargo fmt`, `cargo check`, build Linux, dan Clippy lulus.

## Checklist runtime Windows yang belum dijalankan

- [ ] Posisi atas-tengah, transparansi, tray Pijar, dan tidak adanya tombol taskbar.
- [ ] Windows Narrator membaca tab, tombol transport, judul, dan lirik dengan benar.
- [ ] `Ctrl+Alt+N`, semua tindakan tray, keluar aplikasi, dan pencegahan instance ganda (termasuk Luma lama).
- [ ] Metadata/transport Spotify dan browser sesuai kemampuan sesi Windows; pergantian pemutar atau lagu tidak mempertahankan lirik lama.
- [ ] Lirik mengikuti pause, resume, seek maju/mundur, akhir lagu, dan sleep/resume pada sesi media nyata.
- [ ] HTTPS LRCLIB, cache offline, koneksi gagal, retry, dan penonaktifan lirik berfungsi pada Windows.
- [ ] Area di luar region meneruskan klik, termasuk saat perubahan lebar, pembalikan animasi, dan perubahan DPI.
- [ ] Performa animasi pada 60/120/144 Hz; belum ada benchmark FPS atau idle CPU.
- [ ] Timer selesai/bunyi, baterai, pengaturan lama, dan penyimpanan setelah restart.
- [ ] DPI 100/125/150/200%, monitor kedua/koordinat negatif, pencabutan monitor, fullscreen, dan remote desktop.

Catatan historis 0.1.x berada di [arsip validasi Luma](archive/VALIDATION-0.1.2.md). GIF lama dan gambar tanpa prefiks `sela-` adalah bukti versi tersebut, bukan tampilan Sela 0.2.0.
