mod auth;
mod db;
mod handlers;
mod models;

use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    routing::{get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::db::DatabaseState;
use crate::handlers::{
    alat_handler::{create_alat, delete_alat, list_alat, update_alat},
    auth_handler::{get_me, login, logout},
    kategori_handler::{create_kategori, delete_kategori, list_kategori, update_kategori},
    kembali_handler::{bayar_denda, list_pengembalian, proses_pengembalian},
    log_handler::list_logs,
    pinjam_handler::{ajukan_peminjaman, approve_peminjaman, list_peminjaman},
    report_handler::{generate_report, get_dashboard_stats},
    user_handler::{create_user, delete_user, list_users, update_user},
};

#[tokio::main]
async fn main() {
    println!("================================================================");
    println!("  UJI KOMPETENSI KEAHLIAN (UKK) REKAYASA PERANGKAT LUNAK 2025/2026");
    println!("  APLIKASI PEMINJAMAN ALAT - KODE: KM25.4.1.1");
    println!("  BAHASA PEMROGRAMAN: RUST (AXUM + TOKIO + ASYNC ENGINE)");
    println!("================================================================");

    // 1. Inisialisasi basis data
    let db_state = DatabaseState::new();
    let db = Arc::new(RwLock::new(db_state));

    // 2. Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 3. API Router
    let api_routes = Router::new()
        // Autentikasi
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(get_me))
        // Manajemen Pengguna (Admin)
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", put(update_user).delete(delete_user))
        // Kategori Alat (Admin & Petugas)
        .route("/kategori", get(list_kategori).post(create_kategori))
        .route("/kategori/:id", put(update_kategori).delete(delete_kategori))
        // Inventaris Alat (Semua level untuk lihat, Admin & Petugas untuk CRUD)
        .route("/alat", get(list_alat).post(create_alat))
        .route("/alat/:id", put(update_alat).delete(delete_alat))
        // Transaksi Peminjaman (Peminjam ajukan, Petugas/Admin approve)
        .route("/peminjaman", get(list_peminjaman).post(ajukan_peminjaman))
        .route("/peminjaman/:id/approve", post(approve_peminjaman))
        // Pengembalian & Denda
        .route("/pengembalian", get(list_pengembalian))
        .route("/pengembalian/proses", post(proses_pengembalian))
        .route("/pengembalian/bayar/:id", post(bayar_denda))
        // Log Aktivitas (Admin)
        .route("/logs", get(list_logs))
        // Dashboard & Laporan Siap Cetak
        .route("/dashboard/stats", get(get_dashboard_stats))
        .route("/reports", get(generate_report))
        .with_state(db.clone());

    // 4. Static Files & Root Router
    let static_service = ServeDir::new("static")
        .append_index_html_on_directories(true);

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(static_service)
        .layer(cors);

    // 5. Jalankan Server HTTP
    let port = 3000;
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("Gagal mengikat port 3000");

    println!("[SERVER] Server Aplikasi Peminjaman Alat Aktif!");
    println!("[URL]    Buka di browser: http://localhost:{}", port);
    println!("[AKUN]   Akun Demo Siap Uji:");
    println!("         - Admin    : username 'admin',    password 'admin123'");
    println!("         - Petugas  : username 'petugas',  password 'petugas123'");
    println!("         - Peminjam : username 'peminjam', password 'peminjam123'");
    println!("================================================================");

    axum::serve(listener, app).await.unwrap();
}
