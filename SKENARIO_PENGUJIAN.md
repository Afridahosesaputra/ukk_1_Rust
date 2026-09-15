# SKENARIO PENGUJIAN APLIKASI PEMINJAMAN ALAT
**Uji Kompetensi Keahlian (UKK) Rekayasa Perangkat Lunak 2025/2026**
**Kode Soal:** KM25.4.1.1
**Metode Pengujian:** Black Box Testing & Integration Testing

Dokumen ini mendokumentasikan 5 skenario uji utama beserta kasus uji positif (*positive test*) dan kasus uji negatif (*negative test*) untuk memastikan seluruh fungsionalitas dan keamanan sistem berjalan sesuai spesifikasi.

---

## Ringkasan Matriks Pengujian

| ID Uji | Modul / Skenario Uji | Jumlah Kasus | Status Hasil |
| :---: | :--- | :---: | :---: |
| **TC-01** | Autentikasi Pengguna (Login User) | 3 Kasus | **PASS** |
| **TC-02** | Manajemen Inventaris (Tambah Alat) | 2 Kasus | **PASS** |
| **TC-03** | Transaksi Peminjaman Alat | 3 Kasus | **PASS** |
| **TC-04** | Pengembalian Alat & Perhitungan Denda | 3 Kasus | **PASS** |
| **TC-05** | Kontrol Privilege / Hak Akses (RBAC) | 2 Kasus | **PASS** |

---

## Rincian Skenario dan Hasil Pengujian

### Skenario 1: Login User (TC-01)
*Tujuan: Memverifikasi sistem hanya mengizinkan pengguna terdaftar dan aktif dengan kredensial yang valid.*

| Kasus ID | Deskripsi Kasus Uji | Data Masukan (Input) | Hasil yang Diharapkan | Hasil Aktual | Kesimpulan |
| :---: | :--- | :--- | :--- | :--- | :---: |
| **TC-01.1** | Login dengan password salah | Username: `admin`<br>Password: `salah123` | Sistem menolak login, status HTTP 401, muncul notifikasi: *"Username atau password salah!"* | Muncul pesan error kredensial tidak valid | **PASS** |
| **TC-01.2** | Login dengan form kosong | Username: ` ` (kosong)<br>Password: ` ` | Sistem menolak proses login dan meminta pengisian seluruh kolom. | Validasi form menolak submit | **PASS** |
| **TC-01.3** | Login sukses sebagai Admin | Username: `admin`<br>Password: `admin123` | Login berhasil, redirect ke dashboard Admin dengan menu penuh (CRUD User, Log Aktivitas, dll). | Masuk ke dashboard Admin | **PASS** |

---

### Skenario 2: Tambah Alat Baru (TC-02)
*Tujuan: Memverifikasi proses penambahan inventaris alat baru beserta validasi input.*

| Kasus ID | Deskripsi Kasus Uji | Data Masukan (Input) | Hasil yang Diharapkan | Hasil Aktual | Kesimpulan |
| :---: | :--- | :--- | :--- | :--- | :---: |
| **TC-02.1** | Tambah alat dengan data lengkap dan valid | Kode: `ALT-099`<br>Nama: `Oscilloscope Digital`<br>Kategori: `Elektronik & IoT`<br>Stok: `5`<br>Denda: `10000` | Data berhasil disimpan, stok tersedia otomatis bernilai 5, tercatat di audit log. | Alat tersimpan di basis data & muncul di katalog | **PASS** |
| **TC-02.2** | Tambah alat dengan kode yang sudah ada (duplikat) | Kode: `ALT-001` (sudah dipakai)<br>Nama: `Duplikat Alat` | Sistem menolak penyimpanan dan memberikan pesan: *"Kode alat sudah terdaftar!"* | Sistem mengembalikan pesan error konflik kode | **PASS** |

---

### Skenario 3: Alur Peminjaman Alat (TC-03)
*Tujuan: Menguji alur peminjaman dari pengajuan oleh siswa hingga persetujuan petugas.*

| Kasus ID | Deskripsi Kasus Uji | Data Masukan (Input) | Hasil yang Diharapkan | Hasil Aktual | Kesimpulan |
| :---: | :--- | :--- | :--- | :--- | :---: |
| **TC-03.1** | Mengajukan pinjam melebihi stok yang tersedia | Alat: `Router MikroTik`<br>Stok Ada: `12`<br>Jumlah Ajuan: `20` | Sistem menolak pengajuan dengan pesan: *"Jumlah pinjam melebihi stok tersedia!"* | Transaksi dibatalkan (Rollback) | **PASS** |
| **TC-03.2** | Mengajukan pinjam dengan stok cukup | Alat: `Arduino Uno`<br>Jumlah: `2`<br>Tgl Pinjam: Hari ini<br>Tgl Rencana Kembali: +3 hari | Pengajuan tersimpan dengan status `menunggu_persetujuan`. Stok belum terpotong. | Tersimpan di tabel peminjaman | **PASS** |
| **TC-03.3** | Petugas menyetujui peminjaman (Approval) | Aksi Petugas: Klik **Setujui** | Status berubah menjadi `dipinjam`, stok tersedia alat terpotong otomatis dari 15 menjadi 13. | Status menjadi dipinjam & stok berkurang 2 | **PASS** |

---

### Skenario 4: Pengembalian Alat & Perhitungan Denda (TC-04)
*Tujuan: Menguji kalkulasi otomatis denda keterlambatan per hari dan denda kondisi alat rusak.*

| Kasus ID | Deskripsi Kasus Uji | Data Masukan (Input) | Hasil yang Diharapkan | Hasil Aktual | Kesimpulan |
| :---: | :--- | :--- | :--- | :--- | :---: |
| **TC-04.1** | Pengembalian tepat waktu kondisi baik | Tgl Rencana: `2026-09-10`<br>Tgl Kembali: `2026-09-10`<br>Kondisi: `baik` | Hari telat: 0. Total Denda: **Rp 0**. Status bayar: *tidak ada denda*. Stok alat kembali bertambah. | Total denda Rp 0, stok alat dipulihkan | **PASS** |
| **TC-04.2** | Pengembalian terlambat 3 hari kondisi baik | Tgl Rencana: `2026-09-10`<br>Tgl Kembali: `2026-09-13`<br>Tarif Denda: `Rp 5.000/hari`<br>Kondisi: `baik` | Hari telat: 3 hari.<br>Denda Telat = 3 × 5.000 = **Rp 15.000**.<br>Denda Kerusakan: Rp 0.<br>Total Denda = **Rp 15.000**. | Kalkulasi denda telat otomatis bernilai Rp 15.000 | **PASS** |
| **TC-04.3** | Pengembalian terlambat 2 hari kondisi rusak ringan | Tgl Rencana: `2026-09-10`<br>Tgl Kembali: `2026-09-12`<br>Kondisi: `rusak_ringan` | Denda Telat: 2 × 5.000 = Rp 10.000.<br>Denda Rusak = Rp 25.000.<br>Total Denda = **Rp 35.000**. | Total denda terhitung Rp 35.000 dengan status belum lunas | **PASS** |

---

### Skenario 5: Pengecekan Privilege & Hak Akses (TC-05)
*Tujuan: Memverifikasi proteksi rute dan batasan hak akses antar level pengguna (RBAC).*

| Kasus ID | Deskripsi Kasus Uji | Data Masukan (Input) | Hasil yang Diharapkan | Hasil Aktual | Kesimpulan |
| :---: | :--- | :--- | :--- | :--- | :---: |
| **TC-05.1** | Peminjam mencoba mengakses endpoint/menu CRUD User Admin | User Login: `peminjam`<br>Akses URL: `/api/users` | Akses ditolak dengan status HTTP 403 Forbidden. Pengguna tidak dapat memodifikasi data akun. | Ditolak oleh middleware hak akses (403 Forbidden) | **PASS** |
| **TC-05.2** | Petugas mencoba mengakses modul Log Aktivitas Admin | User Login: `petugas`<br>Akses URL: `/api/logs` | Menu Log Aktivitas disembunyikan dan endpoint dilindungi khusus level `admin`. | Akses ditolak (403 Forbidden) | **PASS** |
