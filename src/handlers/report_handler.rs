use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use crate::auth::{extract_auth_user, require_role};
use crate::db::Db;
use crate::models::{ApiResponse, DashboardStats, ReportSummary, Role};

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub mulai: Option<String>,
    pub selesai: Option<String>,
    pub status: Option<String>,
}

pub async fn get_dashboard_stats(
    headers: HeaderMap,
    State(db): State<Db>,
) -> (StatusCode, Json<ApiResponse<DashboardStats>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    let state = db.read().await;

    let total_alat = state.alat.len();
    let total_tersedia: i32 = state.alat.iter().map(|a| a.stok_tersedia).sum();
    let total_dipinjam: i32 = state.peminjaman.iter()
        .filter(|p| {
            if auth_user.role == Role::Peminjam {
                p.user_id == auth_user.id && p.status == "dipinjam"
            } else {
                p.status == "dipinjam"
            }
        })
        .map(|p| p.jumlah)
        .sum();

    let total_menunggu = state.peminjaman.iter()
        .filter(|p| {
            if auth_user.role == Role::Peminjam {
                p.user_id == auth_user.id && p.status == "menunggu"
            } else {
                p.status == "menunggu"
            }
        })
        .count();

    let total_kembali = state.pengembalian.iter()
        .filter(|r| {
            if auth_user.role == Role::Peminjam {
                r.user_id == auth_user.id
            } else {
                true
            }
        })
        .count();

    let total_user = state.users.len();
    let total_denda_terkumpul: f64 = state.pengembalian.iter()
        .filter(|r| {
            if auth_user.role == Role::Peminjam {
                r.user_id == auth_user.id && r.status_pembayaran == "lunas"
            } else {
                r.status_pembayaran == "lunas"
            }
        })
        .map(|r| r.total_denda)
        .sum();

    let stats = DashboardStats {
        total_alat,
        total_tersedia,
        total_dipinjam,
        total_menunggu,
        total_kembali,
        total_user,
        total_denda_terkumpul,
    };

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Statistik dashboard berhasil dimuat.".to_string(),
            data: Some(stats),
        }),
    )
}

pub async fn generate_report(
    headers: HeaderMap,
    State(db): State<Db>,
    Query(params): Query<ReportQuery>,
) -> (StatusCode, Json<ApiResponse<ReportSummary>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    // Laporan hanya untuk Admin dan Petugas
    if let Err((status, err)) = require_role(&auth_user, &[Role::Admin, Role::Petugas]) {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    let state = db.read().await;

    let mulai = params.mulai.unwrap_or_else(|| "2026-01-01".to_string());
    let selesai = params.selesai.unwrap_or_else(|| "2026-12-31".to_string());

    let mut filtered_loans = state.peminjaman.clone();
    filtered_loans.retain(|p| {
        let tgl = &p.tanggal_pinjam;
        tgl >= &mulai && tgl <= &selesai
    });

    if let Some(ref st) = params.status {
        if !st.trim().is_empty() && st != "semua" {
            filtered_loans.retain(|p| p.status.to_lowercase() == st.to_lowercase());
        }
    }

    let mut filtered_returns = state.pengembalian.clone();
    filtered_returns.retain(|r| {
        let tgl = &r.tanggal_kembali;
        tgl >= &mulai && tgl <= &selesai
    });

    let total_transaksi = filtered_loans.len();
    let total_alat_terpinjam: i32 = filtered_loans.iter().map(|p| p.jumlah).sum();
    let total_denda: f64 = filtered_returns.iter().map(|r| r.total_denda).sum();

    let report = ReportSummary {
        periode_mulai: mulai,
        periode_selesai: selesai,
        total_transaksi,
        total_alat_terpinjam,
        total_denda,
        items: filtered_loans,
        returns: filtered_returns,
    };

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Laporan transaksi berhasil digenerate.".to_string(),
            data: Some(report),
        }),
    )
}
