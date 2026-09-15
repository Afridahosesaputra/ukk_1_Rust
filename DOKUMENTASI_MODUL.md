# DOKUMENTASI MODUL SISTEM PEMINJAMAN ALAT
**Uji Kompetensi Keahlian (UKK) Rekayasa Perangkat Lunak 2025/2026**
**Kode Soal:** KM25.4.1.1
**Bahasa Pemrograman:** Rust (Axum Web Framework, SQLite & SQL)

Dokumentasi ini menyajikan analisis **Input - Process - Output (IPO)** terperinci serta spesifikasi fungsi, prosedur, dan method yang diimplementasikan pada aplikasi.

---

## Modul 1: Modul Autentikasi & Otorisasi Pengguna (Auth & RBAC)

### 1.1 Proses Login Pengguna (`auth_handler::login`)
- **Tujuan:** Memvalidasi kredensial pengguna dan menerbitkan session/token untuk hak akses (Role-Based Access Control: Admin, Petugas, Peminjam).
- **Input:**
  - `username` (String): Nama akun pengguna (contoh: `admin`, `petugas`, `peminjam`).
  - `password` (String): Kata sandi pengguna.
- **Proses:**
  1. Validasi keberadaan input (tidak boleh string kosong).
  2. Eksekusi query basis data untuk mencocokkan `username` dan memastikan `status = 'aktif'`.
  3. Verifikasi hash password dengan `verify_password()`.
  4. Jika cocok, buat token otentikasi session.
  5. Catat waktu dan identitas login ke tabel `log_aktivitas`.
- **Output:**
  - Response JSON: Objek status sukses, profil pengguna (`id`, `username`, `nama_lengkap`, `role`), dan session cookie.
  - Error: Pesan kesalahan kredensial HTTP 401 jika username/password tidak cocok atau akun dinonaktifkan.

### 1.2 Method / Fungsi Terkait di Rust:
```rust
// auth.rs
pub fn hash_password(plain: &str) -> String;
pub fn verify_password(plain: &str, hashed: &str) -> bool;
pub fn generate_session_token(user_id: i64, role: &str) -> String;
pub fn validate_session(req: &Request) -> Result<AuthUser, StatusCode>;
```

---

## Modul 2: Modul Manajemen Pengguna (CRUD User - Admin)

### 2.1 Tambah Pengguna Baru (`user_handler::create_user`)
- **Input:** `username`, `password`, `nama_lengkap`, `no_telepon`, `alamat`, `role` (`admin` | `petugas` | `peminjam`).
- **Proses:**
  1. Validasi hak akses pemanggil (hanya role `admin` yang diizinkan).
  2. Periksa apakah username sudah terdaftar (keunikan username).
  3. Hash password sebelum disimpan.
  4. Eksekusi query `INSERT INTO users ...`.
  5. Catat audit log: `TAMBAH_USER`.
- **Output:** JSON user baru yang berhasil dibuat (tanpa mengembalikan password hash).

### 2.2 Lihat & Filter Pengguna (`user_handler::list_users`)
- **Input:** Query parameter `role` (opsional), `search` (opsional), `page`, `limit`.
- **Proses:** Query `SELECT * FROM users ORDER BY id DESC LIMIT ? OFFSET ?`.
- **Output:** Array JSON daftar pengguna dan total hitungan.

### 2.3 Update & Hapus Pengguna (`user_handler::update_user`, `user_handler::delete_user`)
- **Input:** `user_id` (Path parameter), data pembaruan, atau konfirmasi penghapusan.
- **Proses:** Cek integritas data (tidak boleh menghapus admin utama atau user yang memiliki transaksi peminjaman aktif).
- **Output:** Status pembaruan/penghapusan sukses.

---

## Modul 3: Modul Manajemen Kategori & Inventaris Alat (CRUD Alat & Kategori)

### 3.1 Tambah Kategori (`kategori_handler::create_kategori`)
- **Hak Akses:** Admin & Petugas.
- **Input:** `nama_kategori` (String), `deskripsi` (String).
- **Proses:** Validasi nama kategori unik, simpan ke tabel `kategori`.
- **Output:** Data kategori baru tersimpan.

### 3.2 Tambah / Edit Alat (`alat_handler::create_alat`, `alat_handler::update_alat`)
- **Hak Akses:** Admin & Petugas.
- **Input:** `kode_alat`, `nama_alat`, `kategori_id`, `stok_total`, `kondisi`, `lokasi`, `spesifikasi`, `tarif_denda_harian`.
- **Proses:**
  1. Validasi kelengkapan data & relasi `kategori_id`.
  2. Set `stok_tersedia = stok_total` pada penambahan baru.
  3. Jalankan query `INSERT` atau `UPDATE`.
  4. Catat aktivitas perubahan ke tabel log.
- **Output:** Objek data alat ter-update.

### 3.3 Katalog Alat Publik (`alat_handler::list_alat`)
- **Hak Akses:** Semua pengguna (Admin, Petugas, Peminjam).
- **Input:** Filter `kategori_id`, `status_stok` (`semua` | `tersedia_saja`), kata kunci pencarian.
- **Proses:** Query efisien dengan `LEFT JOIN kategori` dan filter aktif.
- **Output:** Daftar inventaris alat beserta informasi status ketersediaan.

---

## Modul 4: Modul Transaksi Peminjaman Alat

### 4.1 Pengajuan Peminjaman (`pinjam_handler::ajukan_peminjaman`)
- **Hak Akses:** Peminjam.
- **Input:** `alat_id`, `jumlah`, `tanggal_pinjam`, `tanggal_kembali_rencana`, `keperluan`.
- **Proses:**
  1. Validasi tanggal kembali tidak boleh lebih awal dari tanggal pinjam.
  2. Periksa stok alat di database: `stok_tersedia >= jumlah`.
  3. Generate `kode_pinjam` unik otomatis (misal: `PINJAM-20260915-081`).
  4. Buka transaksi database (`BEGIN TRANSACTION`).
  5. Simpan record header ke tabel `peminjaman` (`status = 'menunggu'`).
  6. Simpan detail ke tabel `detail_peminjaman`.
  7. Commit transaksi database.
- **Output:** Kode peminjaman dan notifikasi status "Menunggu Persetujuan Petugas".

### 4.2 Verifikasi & Persetujuan Peminjaman (`pinjam_handler::approve_peminjaman`)
- **Hak Akses:** Petugas & Admin.
- **Input:** `peminjaman_id`, `aksi` (`setujui` | `tolak`), `catatan_petugas`.
- **Proses:**
  1. Buka transaksi basis data (`BEGIN TRANSACTION`).
  2. Jika `aksi == 'setujui'`:
     - Periksa ulang ketersediaan stok fisik.
     - Ubah status peminjaman menjadi `dipinjam`.
     - Kurangi stok alat: `stok_tersedia = stok_tersedia - jumlah`.
  3. Jika `aksi == 'tolak'`:
     - Ubah status peminjaman menjadi `ditolak`.
  4. Catat `disetujui_oleh` = user ID petugas yang memverifikasi.
  5. Catat audit trail ke `log_aktivitas`.
  6. Commit transaksi (`COMMIT`).
- **Output:** Status baru peminjaman dan pembaruan stok real-time.

---

## Modul 5: Modul Pengembalian Alat & Perhitungan Denda

### 5.1 Proses Pengembalian & Kalkulasi Denda (`kembali_handler::proses_pengembalian`)
- **Hak Akses:** Petugas & Admin.
- **Input:**
  - `peminjaman_id` (Integer)
  - `tanggal_kembali_aktual` (Date)
  - `kondisi_alat_kembali` (`baik` | `rusak_ringan` | `rusak_berat`)
  - `denda_kerusakan_manual` (Opsional, jika ada kerusakan khusus)
  - `catatan` (String)
- **Proses Bisnis:**
  1. Ambil data peminjaman dan tarif denda per hari dari database.
  2. Hitung keterlambatan: `hari_terlambat = max(0, tgl_kembali - tgl_rencana)`.
  3. Hitung denda keterlambatan: `denda_terlambat = hari_terlambat * tarif_harian`.
  4. Hitung denda kerusakan:
     - `baik`: Rp 0
     - `rusak_ringan`: Rp 25.000
     - `rusak_berat`: Rp 100.000 (atau ganti rugi)
  5. `total_denda = denda_terlambat + denda_kerusakan`.
  6. Mulai transaksi ACID:
     - Update status peminjaman menjadi `selesai`.
     - Pulihkan stok alat: `stok_tersedia = stok_tersedia + jumlah_kembali`.
     - Insert record baru ke tabel `pengembalian`.
     - Catat log aktivitas.
  7. Commit transaksi.
- **Output:** Rincian bukti pengembalian (struk), total denda yang harus dibayar, dan status pembayaran.

---

## Modul 6: Modul Audit Trail & Log Aktivitas (Admin)

### 6.1 Catat & Pantau Log (`log_handler::get_logs`)
- **Hak Akses:** Admin.
- **Input:** Filter tanggal, modul, atau username.
- **Proses:** `SELECT * FROM log_aktivitas ORDER BY created_at DESC LIMIT 100`.
- **Output:** Tabel jejak rekam audit aktivitas semua pengguna secara kronologis.

---

## Modul 7: Modul Pelaporan & Cetak Laporan (Laporan UKK)

### 7.1 Generate Laporan Transaksi (`report_handler::generate_report`)
- **Hak Akses:** Admin & Petugas.
- **Input:** `periode_mulai`, `periode_selesai`, filter `status`.
- **Proses:**
  - Query agregasi: total peminjaman, jumlah alat terpinjam, total denda terkumpul.
  - Query rekapitulasi data peminjaman & pengembalian terperinci.
- **Output:**
  - Tampilan tabel laporan resmi lengkap dengan Kop Surat Sekolah, Nomor Laporan, Ringkasan Statistik, dan Format Siap Cetak (`window.print()`).
