use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Local;

use crate::models::{
    Alat, DetailPeminjaman, Kategori, LogAktivitas, Peminjaman, Pengembalian, Role, User,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseState {
    pub users: Vec<User>,
    pub kategori: Vec<Kategori>,
    pub alat: Vec<Alat>,
    pub peminjaman: Vec<Peminjaman>,
    pub detail_peminjaman: Vec<DetailPeminjaman>,
    pub pengembalian: Vec<Pengembalian>,
    pub log_aktivitas: Vec<LogAktivitas>,
    next_user_id: i64,
    next_kategori_id: i64,
    next_alat_id: i64,
    next_peminjaman_id: i64,
    next_detail_id: i64,
    next_pengembalian_id: i64,
    next_log_id: i64,
}

pub type Db = Arc<RwLock<DatabaseState>>;

impl DatabaseState {
    pub fn new() -> Self {
        let db_path = Path::new("data/database.json");
        if db_path.exists() {
            if let Ok(content) = fs::read_to_string(db_path) {
                if let Ok(db) = serde_json::from_str::<DatabaseState>(&content) {
                    println!("[DATABASE] Memuat basis data dari data/database.json ({} alat, {} user, {} peminjaman)",
                             db.alat.len(), db.users.len(), db.peminjaman.len());
                    return db;
                }
            }
        }

        println!("[DATABASE] Inisialisasi basis data awal (Seed Data UKK)...");
        let mut db = DatabaseState {
            users: Vec::new(),
            kategori: Vec::new(),
            alat: Vec::new(),
            peminjaman: Vec::new(),
            detail_peminjaman: Vec::new(),
            pengembalian: Vec::new(),
            log_aktivitas: Vec::new(),
            next_user_id: 1,
            next_kategori_id: 1,
            next_alat_id: 1,
            next_peminjaman_id: 1,
            next_detail_id: 1,
            next_pengembalian_id: 1,
            next_log_id: 1,
        };

        db.seed_data();
        db.save_to_disk();
        db
    }

    pub fn save_to_disk(&self) {
        if let Err(e) = fs::create_dir_all("data") {
            eprintln!("[ERROR] Gagal membuat direktori data: {}", e);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if let Err(e) = fs::write("data/database.json", json) {
                eprintln!("[ERROR] Gagal menyimpan data/database.json: {}", e);
            }
        }
    }

    pub fn add_log(&mut self, user_id: Option<i64>, username: &str, role: &str, aksi: &str, modul: &str, detail: &str) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log = LogAktivitas {
            id: self.next_log_id,
            user_id,
            username: username.to_string(),
            role: role.to_string(),
            aksi: aksi.to_string(),
            modul: modul.to_string(),
            detail: detail.to_string(),
            ip_address: "127.0.0.1".to_string(),
            created_at: now,
        };
        self.next_log_id += 1;
        self.log_aktivitas.push(log);
        self.save_to_disk();
    }

    fn seed_data(&mut self) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // 1. Seed Users (Admin, Petugas, Peminjam)
        self.users = vec![
            User {
                id: 1,
                username: "admin".to_string(),
                password: "admin123".to_string(),
                nama_lengkap: "Administrator Utama".to_string(),
                no_telepon: "081234567890".to_string(),
                alamat: "Lab Komputer 1".to_string(),
                role: Role::Admin,
                status: "aktif".to_string(),
                created_at: now.clone(),
            },
            User {
                id: 2,
                username: "petugas".to_string(),
                password: "petugas123".to_string(),
                nama_lengkap: "Budi Santoso (Petugas Lab)".to_string(),
                no_telepon: "081234567891".to_string(),
                alamat: "Ruang Petugas Inventaris".to_string(),
                role: Role::Petugas,
                status: "aktif".to_string(),
                created_at: now.clone(),
            },
            User {
                id: 3,
                username: "peminjam".to_string(),
                password: "peminjam123".to_string(),
                nama_lengkap: "Ahmad Dani (Siswa RPL)".to_string(),
                no_telepon: "081234567892".to_string(),
                alamat: "Kelas XII RPL 1".to_string(),
                role: Role::Peminjam,
                status: "aktif".to_string(),
                created_at: now.clone(),
            },
            User {
                id: 4,
                username: "siswa2".to_string(),
                password: "siswa123".to_string(),
                nama_lengkap: "Siti Rahma (Siswi RPL)".to_string(),
                no_telepon: "081234567893".to_string(),
                alamat: "Kelas XII RPL 2".to_string(),
                role: Role::Peminjam,
                status: "aktif".to_string(),
                created_at: now.clone(),
            },
        ];
        self.next_user_id = 5;

        // 2. Seed Kategori Alat
        self.kategori = vec![
            Kategori {
                id: 1,
                nama_kategori: "Elektronik & IoT".to_string(),
                deskripsi: "Komponen mikrokontroler, sensor, kabel jumper, board ESP32/Arduino".to_string(),
                created_at: now.clone(),
            },
            Kategori {
                id: 2,
                nama_kategori: "Komputer & Jaringan".to_string(),
                deskripsi: "Laptop praktik, Switch hub, router Mikrotik, crimping tool, LAN tester".to_string(),
                created_at: now.clone(),
            },
            Kategori {
                id: 3,
                nama_kategori: "Audio Visual".to_string(),
                deskripsi: "Proyektor multimedia, kamera digital, mikrofon wireless, tripod".to_string(),
                created_at: now.clone(),
            },
            Kategori {
                id: 4,
                nama_kategori: "Pertukangan & Mekanik".to_string(),
                deskripsi: "Solder listrik, multimeter digital, set obeng presisi, bor tangan".to_string(),
                created_at: now.clone(),
            },
        ];
        self.next_kategori_id = 5;

        // 3. Seed Alat
        self.alat = vec![
            Alat {
                id: 1,
                kode_alat: "ALT-001".to_string(),
                nama_alat: "Arduino Uno R3 Starter Kit".to_string(),
                kategori_id: 1,
                nama_kategori: "Elektronik & IoT".to_string(),
                stok_total: 15,
                stok_tersedia: 14, // 1 sedang dipinjam di seed
                kondisi: "baik".to_string(),
                lokasi: "Rak A-1".to_string(),
                spesifikasi: "Board ATmega328P + Kabel USB + Breadboard + 30 Sensor".to_string(),
                tarif_denda_harian: 5000.0,
                created_at: now.clone(),
            },
            Alat {
                id: 2,
                kode_alat: "ALT-002".to_string(),
                nama_alat: "ESP32 NodeMCU Wi-Fi + BLE".to_string(),
                kategori_id: 1,
                nama_kategori: "Elektronik & IoT".to_string(),
                stok_total: 20,
                stok_tersedia: 20,
                kondisi: "baik".to_string(),
                lokasi: "Rak A-2".to_string(),
                spesifikasi: "Dual Core 240MHz, 4MB Flash, Wi-Fi 802.11 b/g/n".to_string(),
                tarif_denda_harian: 5000.0,
                created_at: now.clone(),
            },
            Alat {
                id: 3,
                kode_alat: "ALT-003".to_string(),
                nama_alat: "Crimping Tool RJ45 CAT6".to_string(),
                kategori_id: 2,
                nama_kategori: "Komputer & Jaringan".to_string(),
                stok_total: 10,
                stok_tersedia: 10,
                kondisi: "baik".to_string(),
                lokasi: "Kotak Alat Jaringan 1".to_string(),
                spesifikasi: "Tang crimping profesional 3 in 1 untuk kabel UTP CAT5e/CAT6".to_string(),
                tarif_denda_harian: 3000.0,
                created_at: now.clone(),
            },
            Alat {
                id: 4,
                kode_alat: "ALT-004".to_string(),
                nama_alat: "Digital LAN Cable Tester".to_string(),
                kategori_id: 2,
                nama_kategori: "Komputer & Jaringan".to_string(),
                stok_total: 8,
                stok_tersedia: 8,
                kondisi: "baik".to_string(),
                lokasi: "Kotak Alat Jaringan 1".to_string(),
                spesifikasi: "Tester RJ45 / RJ11 dengan indikator LED 8 pin".to_string(),
                tarif_denda_harian: 3000.0,
                created_at: now.clone(),
            },
            Alat {
                id: 5,
                kode_alat: "ALT-005".to_string(),
                nama_alat: "Router MikroTik RB750r2 (hEX lite)".to_string(),
                kategori_id: 2,
                nama_kategori: "Komputer & Jaringan".to_string(),
                stok_total: 12,
                stok_tersedia: 12,
                kondisi: "baik".to_string(),
                lokasi: "Lemari Server".to_string(),
                spesifikasi: "5 Port Fast Ethernet, RouterOS Level 4".to_string(),
                tarif_denda_harian: 10000.0,
                created_at: now.clone(),
            },
            Alat {
                id: 6,
                kode_alat: "ALT-006".to_string(),
                nama_alat: "Proyektor Epson EB-X500".to_string(),
                kategori_id: 3,
                nama_kategori: "Audio Visual".to_string(),
                stok_total: 4,
                stok_tersedia: 4,
                kondisi: "baik".to_string(),
                lokasi: "Ruang Multimedia".to_string(),
                spesifikasi: "3600 Lumens, Resolusi XGA 1024x768, Port HDMI/VGA".to_string(),
                tarif_denda_harian: 25000.0,
                created_at: now.clone(),
            },
            Alat {
                id: 7,
                kode_alat: "ALT-007".to_string(),
                nama_alat: "Solder Listrik Digital 60W".to_string(),
                kategori_id: 4,
                nama_kategori: "Pertukangan & Mekanik".to_string(),
                stok_total: 10,
                stok_tersedia: 10,
                kondisi: "baik".to_string(),
                lokasi: "Meja Praktik Solder".to_string(),
                spesifikasi: "Suhu adjustable 200-450C dengan dudukan dan timah".to_string(),
                tarif_denda_harian: 5000.0,
                created_at: now.clone(),
            },
            Alat {
                id: 8,
                kode_alat: "ALT-008".to_string(),
                nama_alat: "Multimeter Digital Auto-Ranging".to_string(),
                kategori_id: 4,
                nama_kategori: "Pertukangan & Mekanik".to_string(),
                stok_total: 6,
                stok_tersedia: 6,
                kondisi: "baik".to_string(),
                lokasi: "Meja Pengukuran".to_string(),
                spesifikasi: "Pengukur tegangan AC/DC, resistansi, kontinuitas buzzer".to_string(),
                tarif_denda_harian: 7500.0,
                created_at: now.clone(),
            },
        ];
        self.next_alat_id = 9;

        // 4. Seed Peminjaman Selesai (Untuk demo laporan)
        self.peminjaman = vec![
            Peminjaman {
                id: 1,
                kode_pinjam: "PINJAM-20260901-001".to_string(),
                user_id: 3,
                peminjam_nama: "Ahmad Dani (Siswa RPL)".to_string(),
                peminjam_username: "peminjam".to_string(),
                alat_id: 3,
                nama_alat: "Crimping Tool RJ45 CAT6".to_string(),
                kode_alat: "ALT-003".to_string(),
                jumlah: 2,
                tanggal_pengajuan: "2026-09-01 08:30:00".to_string(),
                tanggal_pinjam: "2026-09-01".to_string(),
                tanggal_kembali_rencana: "2026-09-04".to_string(),
                tanggal_kembali_aktual: Some("2026-09-04".to_string()),
                status: "selesai".to_string(),
                keperluan: "Praktik Uji Coba Jaringan LAN".to_string(),
                disetujui_oleh: Some(2),
                nama_petugas: Some("Budi Santoso (Petugas Lab)".to_string()),
                catatan_petugas: Some("Gunakan alat dengan hati-hati".to_string()),
                total_denda: 0.0,
                created_at: "2026-09-01 08:30:00".to_string(),
            },
            Peminjaman {
                id: 2,
                kode_pinjam: "PINJAM-20260910-002".to_string(),
                user_id: 3,
                peminjam_nama: "Ahmad Dani (Siswa RPL)".to_string(),
                peminjam_username: "peminjam".to_string(),
                alat_id: 1,
                nama_alat: "Arduino Uno R3 Starter Kit".to_string(),
                kode_alat: "ALT-001".to_string(),
                jumlah: 1,
                tanggal_pengajuan: "2026-09-10 09:15:00".to_string(),
                tanggal_pinjam: "2026-09-10".to_string(),
                tanggal_kembali_rencana: "2026-09-14".to_string(),
                tanggal_kembali_aktual: None,
                status: "dipinjam".to_string(),
                keperluan: "Praktik Proyek IoT Rumah Pintar".to_string(),
                disetujui_oleh: Some(2),
                nama_petugas: Some("Budi Santoso (Petugas Lab)".to_string()),
                catatan_petugas: Some("Kembalikan sebelum jam 15.00".to_string()),
                total_denda: 0.0,
                created_at: "2026-09-10 09:15:00".to_string(),
            },
        ];
        self.next_peminjaman_id = 3;

        // 5. Seed Detail Peminjaman
        self.detail_peminjaman = vec![
            DetailPeminjaman {
                id: 1,
                peminjaman_id: 1,
                alat_id: 3,
                jumlah: 2,
                kondisi_saat_pinjam: "baik".to_string(),
                kondisi_saat_kembali: "baik".to_string(),
                denda_kerusakan: 0.0,
            },
            DetailPeminjaman {
                id: 2,
                peminjaman_id: 2,
                alat_id: 1,
                jumlah: 1,
                kondisi_saat_pinjam: "baik".to_string(),
                kondisi_saat_kembali: "baik".to_string(),
                denda_kerusakan: 0.0,
            },
        ];
        self.next_detail_id = 3;

        // 6. Seed Pengembalian
        self.pengembalian = vec![
            Pengembalian {
                id: 1,
                kode_kembali: "KEMBALI-20260904-001".to_string(),
                peminjaman_id: 1,
                kode_pinjam: "PINJAM-20260901-001".to_string(),
                user_id: 3,
                nama_peminjam: "Ahmad Dani (Siswa RPL)".to_string(),
                petugas_id: 2,
                nama_petugas: "Budi Santoso (Petugas Lab)".to_string(),
                nama_alat: "Crimping Tool RJ45 CAT6".to_string(),
                jumlah: 2,
                tanggal_kembali: "2026-09-04".to_string(),
                hari_terlambat: 0,
                denda_keterlambatan: 0.0,
                denda_kerusakan: 0.0,
                total_denda: 0.0,
                status_pembayaran: "tidak_ada_denda".to_string(),
                catatan: "Alat kembali tepat waktu dalam keadaan lengkap dan baik.".to_string(),
                created_at: "2026-09-04 14:00:00".to_string(),
            },
        ];
        self.next_pengembalian_id = 2;

        // 7. Seed Log Aktivitas
        self.log_aktivitas = vec![
            LogAktivitas {
                id: 1,
                user_id: Some(1),
                username: "admin".to_string(),
                role: "admin".to_string(),
                aksi: "INIT".to_string(),
                modul: "SISTEM".to_string(),
                detail: "Inisialisasi basis data aplikasi peminjaman alat UKK".to_string(),
                ip_address: "127.0.0.1".to_string(),
                created_at: now.clone(),
            },
            LogAktivitas {
                id: 2,
                user_id: Some(2),
                username: "petugas".to_string(),
                role: "petugas".to_string(),
                aksi: "APPROVAL".to_string(),
                modul: "PEMINJAMAN".to_string(),
                detail: "Menyetujui peminjaman PINJAM-20260901-001".to_string(),
                ip_address: "127.0.0.1".to_string(),
                created_at: now.clone(),
            },
            LogAktivitas {
                id: 3,
                user_id: Some(2),
                username: "petugas".to_string(),
                role: "petugas".to_string(),
                aksi: "VERIFIKASI".to_string(),
                modul: "PENGEMBALIAN".to_string(),
                detail: "Memverifikasi pengembalian KEMBALI-20260904-001 (Total Denda: Rp 0)".to_string(),
                ip_address: "127.0.0.1".to_string(),
                created_at: now,
            },
        ];
        self.next_log_id = 4;
    }

    // Generator ID Baru
    pub fn next_user_id(&mut self) -> i64 {
        let id = self.next_user_id;
        self.next_user_id += 1;
        id
    }

    pub fn next_kategori_id(&mut self) -> i64 {
        let id = self.next_kategori_id;
        self.next_kategori_id += 1;
        id
    }

    pub fn next_alat_id(&mut self) -> i64 {
        let id = self.next_alat_id;
        self.next_alat_id += 1;
        id
    }

    pub fn next_peminjaman_id(&mut self) -> i64 {
        let id = self.next_peminjaman_id;
        self.next_peminjaman_id += 1;
        id
    }

    pub fn next_detail_id(&mut self) -> i64 {
        let id = self.next_detail_id;
        self.next_detail_id += 1;
        id
    }

    pub fn next_pengembalian_id(&mut self) -> i64 {
        let id = self.next_pengembalian_id;
        self.next_pengembalian_id += 1;
        id
    }
}
