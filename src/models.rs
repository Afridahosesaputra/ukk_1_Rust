use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Petugas,
    Peminjam,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Petugas => "petugas",
            Role::Peminjam => "peminjam",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => Role::Admin,
            "petugas" => Role::Petugas,
            _ => Role::Peminjam,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub nama_lengkap: String,
    pub no_telepon: String,
    pub alamat: String,
    pub role: Role,
    pub status: String, // "aktif" / "nonaktif"
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kategori {
    pub id: i64,
    pub nama_kategori: String,
    pub deskripsi: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alat {
    pub id: i64,
    pub kode_alat: String,
    pub nama_alat: String,
    pub kategori_id: i64,
    #[serde(default)]
    pub nama_kategori: String,
    pub stok_total: i32,
    pub stok_tersedia: i32,
    pub kondisi: String, // "baik", "rusak_ringan", "rusak_berat"
    pub lokasi: String,
    pub spesifikasi: String,
    pub tarif_denda_harian: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peminjaman {
    pub id: i64,
    pub kode_pinjam: String,
    pub user_id: i64,
    pub peminjam_nama: String,
    pub peminjam_username: String,
    pub alat_id: i64,
    pub nama_alat: String,
    pub kode_alat: String,
    pub jumlah: i32,
    pub tanggal_pengajuan: String,
    pub tanggal_pinjam: String,
    pub tanggal_kembali_rencana: String,
    pub tanggal_kembali_aktual: Option<String>,
    pub status: String, // "menunggu", "disetujui", "ditolak", "dipinjam", "selesai", "dibatalkan"
    pub keperluan: String,
    pub disetujui_oleh: Option<i64>,
    pub nama_petugas: Option<String>,
    pub catatan_petugas: Option<String>,
    pub total_denda: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailPeminjaman {
    pub id: i64,
    pub peminjaman_id: i64,
    pub alat_id: i64,
    pub jumlah: i32,
    pub kondisi_saat_pinjam: String,
    pub kondisi_saat_kembali: String,
    pub denda_kerusakan: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pengembalian {
    pub id: i64,
    pub kode_kembali: String,
    pub peminjaman_id: i64,
    pub kode_pinjam: String,
    pub user_id: i64,
    pub nama_peminjam: String,
    pub petugas_id: i64,
    pub nama_petugas: String,
    pub nama_alat: String,
    pub jumlah: i32,
    pub tanggal_kembali: String,
    pub hari_terlambat: i32,
    pub denda_keterlambatan: f64,
    pub denda_kerusakan: f64,
    pub total_denda: f64,
    pub status_pembayaran: String, // "lunas", "belum_lunas", "tidak_ada_denda"
    pub catatan: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAktivitas {
    pub id: i64,
    pub user_id: Option<i64>,
    pub username: String,
    pub role: String,
    pub aksi: String,
    pub modul: String,
    pub detail: String,
    pub ip_address: String,
    pub created_at: String,
}

// Request & Response DTOs
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub token: String,
    pub user: Option<User>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub nama_lengkap: String,
    pub no_telepon: String,
    pub alamat: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub nama_lengkap: Option<String>,
    pub password: Option<String>,
    pub no_telepon: Option<String>,
    pub alamat: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateKategoriRequest {
    pub nama_kategori: String,
    pub deskripsi: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAlatRequest {
    pub kode_alat: String,
    pub nama_alat: String,
    pub kategori_id: i64,
    pub stok_total: i32,
    pub kondisi: String,
    pub lokasi: String,
    pub spesifikasi: String,
    pub tarif_denda_harian: f64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAlatRequest {
    pub kode_alat: Option<String>,
    pub nama_alat: Option<String>,
    pub kategori_id: Option<i64>,
    pub stok_total: Option<i32>,
    pub stok_tersedia: Option<i32>,
    pub kondisi: Option<String>,
    pub lokasi: Option<String>,
    pub spesifikasi: Option<String>,
    pub tarif_denda_harian: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct AjukanPinjamRequest {
    pub alat_id: i64,
    pub jumlah: i32,
    pub tanggal_pinjam: String,
    pub tanggal_kembali_rencana: String,
    pub keperluan: String,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalRequest {
    pub aksi: String, // "setujui" atau "tolak"
    pub catatan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PengembalianRequest {
    pub peminjaman_id: i64,
    pub tanggal_kembali: String,
    pub kondisi_kembali: String, // "baik", "rusak_ringan", "rusak_berat"
    pub denda_kerusakan: Option<f64>,
    pub catatan: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_alat: usize,
    pub total_tersedia: i32,
    pub total_dipinjam: i32,
    pub total_menunggu: usize,
    pub total_kembali: usize,
    pub total_user: usize,
    pub total_denda_terkumpul: f64,
}

#[derive(Debug, Serialize)]
pub struct ReportSummary {
    pub periode_mulai: String,
    pub periode_selesai: String,
    pub total_transaksi: usize,
    pub total_alat_terpinjam: i32,
    pub total_denda: f64,
    pub items: Vec<Peminjaman>,
    pub returns: Vec<Pengembalian>,
}
