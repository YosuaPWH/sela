<p align="center"><img src="docs/brand/sela-mark.svg" width="72" height="72" alt="Logo Sela"></p>

<h1 align="center">Sela</h1>

<p align="center"><strong>Musik dekat. Fokus tetap.</strong><br>Notch untuk Windows, dibuat dengan Rust. Ditemani Pijar, si kunang-kunang.</p>

<p align="center">
  <a href="#mulai"><strong>Mulai</strong></a> ·
  <a href="#musik-dan-lirik">Musik & lirik</a> ·
  <a href="docs/BUILDING.md">Build</a> ·
  <a href="docs/VALIDATION.md">Pengujian</a>
</p>

<p align="center"><code>Windows 10 / 11</code> · <code>Native Rust</code> · <code>Portable</code> · <a href="LICENSE">MIT</a></p>

<p align="center"><img src="docs/screenshots/sela-media-150.png" width="600" alt="Sela dengan kontrol pemutar sage dan baris lirik aktif amber"></p>

Sela tinggal di tengah atas layar. Buka panel untuk mengatur musik atau memulai sesi fokus. Saat ditutup, judul lagu dan satu baris lirik tetap menemanimu.

Warna charcoal, sage, dan amber menjaga panel tetap tenang. Lebar dan tinggi bergerak dengan spring yang menyambung saat arah dibalik. Pijar memberi sedikit hangat; kamu bisa menyembunyikannya di Atur.

<sub>Galeri diambil dari aplikasi Rust yang berjalan di Linux. Musik dan lirik pada gambar memakai mode demo dengan teks orisinal, tanpa audio. Integrasi Windows masih memerlukan pengujian pada perangkat Windows; lihat <a href="docs/VALIDATION.md">bukti dan batas pengujian</a>.</sub>

## Musik dan lirik

<p align="center"><img src="docs/screenshots/sela-compact-150.png" width="600" alt="Notch ringkas: judul di kiri dan tepat satu baris lirik di kanan"></p>

- **Kontrol Windows:** metadata, posisi, putar/jeda, sebelumnya, dan berikutnya melalui Windows System Media Transport Controls. Tombol mengikuti kemampuan pemutar.
- **Lirik tersinkron:** baris aktif disorot amber; panel terbuka menampilkan baris sebelum dan sesudahnya. Posisi mengikuti pemutar, termasuk ketika lagu dijeda atau posisi berubah.
- **Tetap terbaca:** judul dan lirik panjang dipotong dengan elipsis; arahkan kursor untuk membaca teks lengkap. Lirik tanpa timestamp ditampilkan sebagai teks biasa dalam panel.
- **Kondisi jelas:** pencarian, koneksi gagal, instrumental, metadata tidak lengkap, dan lirik yang belum tersedia memiliki pesan sendiri. Hasil pencarian lagu sebelumnya tidak menggantikan lagu baru.

Lirik berasal dari [LRCLIB](https://lrclib.net). Ketersediaan, isi, dan ketepatan waktunya bergantung pada data sumber. Sela mencocokkan judul, artis, dan durasi; ia tidak menebak sinkronisasi dari lirik biasa. Timeline menunjukkan posisi; kontrol seek belum disediakan.

## Ruang untuk bekerja

| Halaman | Kegunaan |
| --- | --- |
| **Media** | Musik Windows dan lirik; panel 600×370, atau notch musik ringkas 580×54. |
| **Fokus** | Sesi 15, 25, atau 50 menit, jeda, lanjutkan, reset, dan bunyi saat selesai. Countdown menjadi prioritas dalam notch ringkas. |
| **Sistem** | CPU/RAM setiap 2 detik selama halaman terbuka; baterai dan status listrik jika tersedia. |
| **Atur** | Monitor, jarak dari atas, hover, kurangi animasi, bunyi, lirik, dan maskot. Pengaturan tersimpan di perangkat. |

| Fokus | Sistem |
| --- | --- |
| ![Timer fokus Sela](docs/screenshots/sela-focus.png) | ![CPU dan RAM dari mesin pengembangan](docs/screenshots/sela-system.png) |
| **Lirik belum tersedia** | **Pengaturan** |
| ![Pesan saat tidak ada lirik yang cocok; data demo](docs/screenshots/sela-missing.png) | ![Pengaturan Sela dan penjelasan data pencarian lirik](docs/screenshots/sela-settings.png) |

Sela selalu di atas, tanpa bingkai atau tombol taskbar. Tray menyediakan akses ke panel, fokus, pengaturan, dan keluar. Pin mempertahankan panel hover. Satu instance per sesi Windows mencegah overlay ganda.

## Mulai

**Windows 10 versi 1809 atau lebih baru / Windows 11, x64**, dengan driver OpenGL 2.0+.

1. Ekstrak paket portable **Sela-windows-x64-v0.2.0.zip** yang disertakan bersama proyek, atau artifact **Sela-windows-x64** dari workflow GitHub Actions.
2. Jalankan **`sela-notch.exe`**.
3. Sentuh notch sebentar, klik, atau tekan **`Ctrl+Alt+N`**. Putar musik di pemutar yang mendukung kontrol media Windows.

Tanpa installer atau hak administrator. Aplikasi belum ditandatangani dan tidak menambahkan startup Windows otomatis.

```powershell
.\sela-notch.exe --expanded
```

Untuk mencoba tampilan tanpa pemutar:

```powershell
.\sela-notch.exe --demo --expanded
```

Mode demo diberi label, tanpa audio atau permintaan LRCLIB. Tombol berikutnya/sebelumnya berpindah di antara lima contoh orisinal: lirik tersinkron, teks panjang, lirik kosong, teks tanpa waktu, dan instrumental. Jalankan tanpa `--demo` untuk memakai pemutar Windows.

| Tindakan | Cara |
| --- | --- |
| Buka / tutup | `Ctrl+Alt+N` |
| Tutup | `Esc` saat panel aktif |
| Buka lewat hover | Tahan kursor sekitar 240 ms |
| Pindah halaman | Klik tab atau gulir di header |
| Pertahankan panel hover | Klik pin |
| Keluar | Tray atau **Atur → Keluar dari Sela**; gulir pengaturan ke bawah |

Panel yang dibuka lewat klik/shortcut tetap terbuka sampai ditutup. Panel hover menutup sekitar 650 ms setelah kursor keluar, kecuali dipin. **Kurangi animasi** langsung menampilkan keadaan akhir. Lirik yang dinonaktifkan memakai notch musik pendek; timer fokus aktif mendapat prioritas di mode ringkas.

## Data dan koneksi

Sela tidak memakai akun atau telemetri. Metadata pemutar dan statistik dibaca dari perangkat. **Saat lirik aktif**, judul, artis, dan durasi dikirim melalui HTTPS ke LRCLIB. Matikan **Atur → Tampilkan lirik musik** untuk menghentikan pencarian baru; permintaan yang sudah berjalan dapat menyelesaikan koneksinya, tetapi hasilnya diabaikan.

- Lirik berhasil disimpan dalam cache lokal selama 30 hari; hasil tidak ditemukan selama 1 hari. Kegagalan koneksi tidak disimpan. Cache yang masih valid dapat dipakai tanpa internet.
- Pengaturan: `%APPDATA%\Sela\Sela\config\settings.json`. Cache: `%LOCALAPPDATA%\Sela\Sela\cache\lyrics`.
- Pengaturan Luma lama dibaca jika file Sela belum ada; penyimpanan berikutnya memakai lokasi Sela. File lama dipertahankan.
- JSON rusak memakai default dan menampilkan pesan di Atur. File asli tidak ditimpa sampai kamu menyimpan perubahan.
- CPU/RAM berhenti dibaca saat halaman Sistem ditutup; baterai ringkas diperiksa setiap 30 detik. Timer dan jumlah sesi berakhir ketika aplikasi keluar.

## Pengembangan

```powershell
cargo build --locked --release
.\target\release\sela-notch.exe
```

Lihat [panduan build](docs/BUILDING.md), [validasi](docs/VALIDATION.md), [desain](DESIGN.md), dan [ruang lingkup produk](PRODUCT.md). Build Windows x64, format, Clippy, dan 22 tes telah lulus dari Linux. Runtime Windows—media, tray, shortcut, click-through, dan DPI lintas monitor—masih perlu diuji langsung.

## Kredit

- [Poni / Oyen Poni](https://github.com/lucacadalora/poni): referensi navigasi header, pembacaan sistem sesuai kebutuhan, dan susunan README. [Catatan sumber](docs/REFERENCE_PONI.md).
- [egui/eframe](https://github.com/emilk/egui), [windows-rs](https://github.com/microsoft/windows-rs), [sysinfo](https://github.com/GuillaumeGomez/sysinfo), [reqwest](https://github.com/seanmonstar/reqwest), dan [LRCLIB](https://lrclib.net).
- DM Sans disertakan dengan [SIL Open Font License](assets/OFL.txt). Pijar berasal dari aset generatif proyek; [prompt dan asal](assets/README.md) disertakan. Lirik demo adalah teks orisinal Sela.

Kode berlisensi **[MIT](LICENSE)**. Proyek independen, tidak berafiliasi dengan Apple, Spotify, atau LRCLIB.
