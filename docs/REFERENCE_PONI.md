# Kajian Poni

[Kembali ke README](../README.md)

Referensi pengguna: [lucacadalora/poni](https://github.com/lucacadalora/poni), diperiksa pada 24 Oktober 2026 di commit [`d24628468173a0d7ead8d6f7f56a81f00bab3371`](https://github.com/lucacadalora/poni/tree/d24628468173a0d7ead8d6f7f56a81f00bab3371).

Kajian membaca README, arsitektur, input header, worker sistem, dan spring. Poni tidak dijalankan pada perangkat Windows di lingkungan ini.

## Pola yang diterapkan

| Pola Poni | Penerapan Luma 0.1.2 |
| --- | --- |
| Scroll header berpindah halaman dengan akumulasi dan cooldown untuk trackpad | `HeaderWheel` memakai raw egui scroll, threshold 32 logical points, jeda 220 ms, serta reset saat arah berganti atau pointer meninggalkan header. Scroll isi halaman tetap ditangani isinya. |
| Worker statistik hanya aktif ketika halaman Sistem terbuka | Worker `sysinfo` menerima perintah aktif/jeda. CPU/RAM dibaca setiap 2 detik saat diperlukan; baseline dipanaskan kembali saat dibuka. Baterai ringkas tetap diperiksa setiap 30 detik. |
| README mendahulukan identitas, demo, fitur nyata, galeri, dan cara pakai | Logo sendiri, navigasi, rekaman sendiri, tabel halaman, galeri, dan panduan kendali. Panduan pengembang dipindah ke BUILDING.md. |

## Pola yang sudah selaras

Poni menggambar pulau di backing window berukuran tetap agar titik tengah tidak berpindah selama morph. Luma sudah memakai permukaan tetap dan region input Windows sejak 0.1.1. Renderer berbeda: Poni memakai Direct2D/DirectComposition, Luma memakai egui/eframe OpenGL.

Spring Luma tetap berupa solusi analitik critically damped. Spring yang sudah diuji tidak diganti sekadar untuk menyamakan kode referensi.

Poni menjadwalkan frame dan banyak layanan berdasarkan event. Luma baru menerapkan CPU/RAM sesuai kebutuhan; UI dan layanan media masih memakai polling. Klaim “zero idle CPU”, ukuran binary, FPS, dan cakupan fitur Poni tidak dipakai sebagai klaim Luma.

## Sumber yang diperiksa

- [README](https://github.com/lucacadalora/poni/blob/d24628468173a0d7ead8d6f7f56a81f00bab3371/README.md): hierarki dokumentasi dan interaksi.
- [ARCHITECTURE.md](https://github.com/lucacadalora/poni/blob/d24628468173a0d7ead8d6f7f56a81f00bab3371/ARCHITECTURE.md): permukaan tetap, frame clock, dan layanan.
- [app.rs](https://github.com/lucacadalora/poni/blob/d24628468173a0d7ead8d6f7f56a81f00bab3371/crates/poni/src/app.rs): wheel header dan aktivasi sysmon sesuai halaman.
- [services/sysmon.rs](https://github.com/lucacadalora/poni/blob/d24628468173a0d7ead8d6f7f56a81f00bab3371/crates/poni/src/services/sysmon.rs): worker yang menunggu perintah ketika halaman ditutup.
- [spring.rs](https://github.com/lucacadalora/poni/blob/d24628468173a0d7ead8d6f7f56a81f00bab3371/crates/poni-core/src/spring.rs): pembanding pendekatan gerak.

Kode Poni berlisensi MIT; nama Oyen Poni, logo, dan karakter Oyen dikecualikan oleh README-nya. Luma memakai implementasi Rust, identitas mint/hitam, logo, dan rekaman sendiri. Aset merek dan salinan teks Poni tidak disertakan dalam proyek ini.
