# LAPORAN EVALUASI PENGEMBANGAN SISTEM
**Uji Kompetensi Keahlian (UKK) Rekayasa Perangkat Lunak 2025/2026**
**Kode Soal:** KM25.4.1.1
**Judul Tugas:** Pengembangan Aplikasi Peminjaman Alat
**Teknologi:** Rust, Axum, SQLite / SQL Database, Vanilla HTML/CSS/JS

---

## 1. Pendahuluan
Laporan evaluasi ini disusun sebagai bentuk pertanggungjawaban teknis pasca-pengembangan aplikasi Peminjaman Alat. Evaluasi mencakup pencapaian fungsionalitas, identifikasi kendala atau batasan teknis saat ini, serta rekomendasi rencana tindak lanjut pengembangan di masa mendatang.

---

## 2. Fitur yang Sudah Berjalan dengan Baik

1. **Manajemen Otentikasi dan Multi-Level Hak Akses (RBAC):**
   - Sistem berhasil memisahkan hak akses ke dalam 3 peran pengguna secara tegas: **Admin**, **Petugas**, dan **Peminjam**.
   - Keamanan login terlindungi dengan validasi kredensial dan audit log.

2. **Pengelolaan Master Data (CRUD User, Kategori, dan Alat):**
   - Penambahan, pengeditan, dan penghapusan alat inventaris dilengkapi dengan kontrol stok otomatis (`stok_total` dan `stok_tersedia`).
   - Kategori alat terintegrasi dengan relasi *foreign key* yang menjamin konsistensi data.

3. **Alur Transaksi Peminjaman (Approval Workflow):**
   - Peminjam dapat melihat katalog alat dan mengajukan peminjaman secara mandiri.
   - Fitur approval oleh Petugas/Admin memotong stok alat secara *atomic* (aman dari race condition).

4. **Kalkulasi Otomatis Pengembalian & Denda:**
   - Perhitungan denda keterlambatan secara dinamis berdasarkan formula `hari_terlambat × tarif_denda_per_hari`.
   - Perhitungan denda kondisi fisik (alat rusak ringan / rusak berat).
   - Pemulihan stok alat kembali ke gudang segera setelah pengembalian dikonfirmasi.

5. **Modul Audit Trail (Log Aktivitas):**
   - Setiap mutasi data penting (login, tambah alat, approval, pengembalian) tercatat lengkap dengan identitas pelaku, jenis aksi, dan waktu transaksi.

6. **Modul Pelaporan & Siap Cetak (Print Ready):**
   - Fasilitas pencetakan laporan transaksi peminjaman dan pengembalian dengan format resmi standar UKK, filter tanggal, dan statistik agregat.

---

## 3. Bug / Catatan Teknis yang Belum Diperbaiki

1. **Notifikasi Real-Time Pihak Ketiga:**
   - Saat ini status persetujuan peminjaman baru dapat dilihat ketika pengguna me-refresh atau membuka halaman riwayat peminjaman (belum terhubung dengan *WebSocket* push notification atau SMS/WhatsApp Gateway).
2. **Unggah Berkas Fisik Bukti Rusak:**
   - Foto kondisi fisik alat yang rusak saat dikembalikan saat ini masih didokumentasikan dalam bentuk catatan teks (*textual remarks*) dan belum memiliki modul upload multi-gambar (*file upload storage*).
3. **Mekanisme Pembayaran Denda Digital:**
   - Pencatatan pembayaran denda baru mencakup status verifikasi kasir manual (*lunas* / *belum_lunas*), belum terintegrasi langsung dengan payment gateway (*QRIS* / *Virtual Account*).

---

## 4. Rencana Pengembangan Berikutnya (Next Milestones)

1. **Integrasi Barcode / QR Code Scanner:**
   - Menambahkan fitur scan QR code atau barcode label pada setiap fisik alat untuk mempercepat proses identifikasi saat penyerahan alat dan pengembalian.
2. **Fitur Notifikasi WhatsApp / Email:**
   - Pengiriman otomatis pesan pengingat (*reminder*) ke nomor WhatsApp peminjam H-1 sebelum tanggal jatuh tempo pengembalian untuk meminimalisir keterlambatan.
3. **Modul Maintenance & Kalibrasi Alat:**
   - Menambahkan jadwal servis berkala dan riwayat perbaikan alat agar kondisi alat laboratorium selalu terpantau dalam kondisi prima.
4. **Export Laporan ke Format Spreadsheet (XLSX / CSV) dan PDF Generator Terintegrasi:**
   - Menyediakan unduhan berkas langsung dalam format Excel untuk mempermudah integrasi dengan bagian keuangan sekolah.
