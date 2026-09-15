use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Local;
use serde::Deserialize;
use crate::auth::{extract_auth_user, require_role};
use crate::db::Db;
use crate::models::{
    AjukanPinjamRequest, ApiResponse, ApprovalRequest, DetailPeminjaman, Peminjaman, Role,
};

#[derive(Debug, Deserialize)]
pub struct PeminjamanFilter {
    pub status: Option<String>,
    pub search: Option<String>,
}

pub async fn list_peminjaman(
    headers: HeaderMap,
    State(db): State<Db>,
    Query(filter): Query<PeminjamanFilter>,
) -> (StatusCode, Json<ApiResponse<Vec<Peminjaman>>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    let state = db.read().await;
    let mut loans = state.peminjaman.clone();

    // Jika peminjam biasa, hanya tampilkan milik dirinya sendiri
    if auth_user.role == Role::Peminjam {
        loans.retain(|p| p.user_id == auth_user.id);
    }

    // Filter status
    if let Some(ref st) = filter.status {
        if !st.trim().is_empty() && st != "semua" {
            loans.retain(|p| p.status.to_lowercase() == st.to_lowercase());
        }
    }

    // Filter pencarian
    if let Some(ref q) = filter.search {
        let q_lower = q.trim().to_lowercase();
        if !q_lower.is_empty() {
            loans.retain(|p| {
                p.kode_pinjam.to_lowercase().contains(&q_lower)
                    || p.peminjam_nama.to_lowercase().contains(&q_lower)
                    || p.nama_alat.to_lowercase().contains(&q_lower)
                    || p.keperluan.to_lowercase().contains(&q_lower)
            });
        }
    }

    // Urutkan dari yang terbaru
    loans.reverse();

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: format!("Berhasil memuat {} data peminjaman.", loans.len()),
            data: Some(loans),
        }),
    )
}

pub async fn ajukan_peminjaman(
    headers: HeaderMap,
    State(db): State<Db>,
    Json(payload): Json<AjukanPinjamRequest>,
) -> (StatusCode, Json<ApiResponse<Peminjaman>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    if let Err((status, err)) = require_role(&auth_user, &[Role::Peminjam]) {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    if payload.jumlah <= 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: "Jumlah alat yang dipinjam minimal 1 unit!".to_string(),
                data: None,
            }),
        );
    }

    let tgl_pinjam = payload.tanggal_pinjam.trim();
    let tgl_kembali = payload.tanggal_kembali_rencana.trim();
    let keperluan = payload.keperluan.trim();

    if tgl_pinjam.is_empty() || tgl_kembali.is_empty() || keperluan.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: "Tanggal pinjam, rencana tanggal kembali, dan keperluan wajib diisi!".to_string(),
                data: None,
            }),
        );
    }

    if tgl_kembali < tgl_pinjam {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: "Tanggal kembali rencana tidak boleh lebih awal dari tanggal pinjam!".to_string(),
                data: None,
            }),
        );
    }

    let mut state = db.write().await;

    // Cari alat dan periksa ketersediaan stok
    let alat = match state.alat.iter().find(|a| a.id == payload.alat_id) {
        Some(a) => a.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "Alat yang dipilih tidak ditemukan!".to_string(),
                    data: None,
                }),
            );
        }
    };

    if alat.stok_tersedia < payload.jumlah {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: format!(
                    "Stok tidak mencukupi! Stok tersedia saat ini: {} unit, jumlah yang diajukan: {} unit.",
                    alat.stok_tersedia, payload.jumlah
                ),
                data: None,
            }),
        );
    }

    let new_pinjam_id = state.next_peminjaman_id();
    let now = Local::now();
    let timestamp_str = now.format("%Y%m%d").to_string();
    let kode_pinjam = format!("PINJAM-{}-{:03}", timestamp_str, new_pinjam_id);
    let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let new_pinjam = Peminjaman {
        id: new_pinjam_id,
        kode_pinjam: kode_pinjam.clone(),
        user_id: auth_user.id,
        peminjam_nama: auth_user.nama_lengkap.clone(),
        peminjam_username: auth_user.username.clone(),
        alat_id: alat.id,
        nama_alat: alat.nama_alat.clone(),
        kode_alat: alat.kode_alat.clone(),
        jumlah: payload.jumlah,
        tanggal_pengajuan: now_str.clone(),
        tanggal_pinjam: tgl_pinjam.to_string(),
        tanggal_kembali_rencana: tgl_kembali.to_string(),
        tanggal_kembali_aktual: None,
        status: "menunggu".to_string(),
        keperluan: keperluan.to_string(),
        disetujui_oleh: None,
        nama_petugas: None,
        catatan_petugas: None,
        total_denda: 0.0,
        created_at: now_str,
    };

    // Tambah detail
    let new_detail_id = state.next_detail_id();
    let detail = DetailPeminjaman {
        id: new_detail_id,
        peminjaman_id: new_pinjam_id,
        alat_id: alat.id,
        jumlah: payload.jumlah,
        kondisi_saat_pinjam: "baik".to_string(),
        kondisi_saat_kembali: "baik".to_string(),
        denda_kerusakan: 0.0,
    };

    state.peminjaman.push(new_pinjam.clone());
    state.detail_peminjaman.push(detail);

    state.add_log(
        Some(auth_user.id),
        &auth_user.username,
        auth_user.role.as_str(),
        "AJUKAN_PINJAM",
        "PEMINJAMAN",
        &format!("Mengajukan pinjam: {} (Alat: {}, Qty: {})", kode_pinjam, alat.nama_alat, payload.jumlah),
    );
    state.save_to_disk();

    (
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            message: format!("Pengajuan peminjaman ({}) berhasil dikirim! Menunggu persetujuan petugas.", kode_pinjam),
            data: Some(new_pinjam),
        }),
    )
}

pub async fn approve_peminjaman(
    Path(id): Path<i64>,
    headers: HeaderMap,
    State(db): State<Db>,
    Json(payload): Json<ApprovalRequest>,
) -> (StatusCode, Json<ApiResponse<Peminjaman>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    if let Err((status, err)) = require_role(&auth_user, &[Role::Admin, Role::Petugas]) {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    let mut state = db.write().await;

    let pinjam_idx = match state.peminjaman.iter().position(|p| p.id == id) {
        Some(i) => i,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "Data peminjaman tidak ditemukan.".to_string(),
                    data: None,
                }),
            );
        }
    };

    if state.peminjaman[pinjam_idx].status != "menunggu" {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: format!(
                    "Peminjaman ini sudah diproses sebelumnya dengan status: '{}'",
                    state.peminjaman[pinjam_idx].status
                ),
                data: None,
            }),
        );
    }

    let alat_id = state.peminjaman[pinjam_idx].alat_id;
    let jumlah_pinjam = state.peminjaman[pinjam_idx].jumlah;
    let kode_pinjam = state.peminjaman[pinjam_idx].kode_pinjam.clone();

    let aksi = payload.aksi.trim().to_lowercase();
    if aksi == "setujui" {
        // Cek kembali stok fisik
        let alat_idx = match state.alat.iter().position(|a| a.id == alat_id) {
            Some(i) => i,
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse {
                        success: false,
                        message: "Alat terkait tidak ditemukan di database!".to_string(),
                        data: None,
                    }),
                );
            }
        };

        if state.alat[alat_idx].stok_tersedia < jumlah_pinjam {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    success: false,
                    message: format!(
                        "Gagal menyetujui: Stok alat tidak mencukupi (Tersedia: {}, Diminta: {}).",
                        state.alat[alat_idx].stok_tersedia, jumlah_pinjam
                    ),
                    data: None,
                }),
            );
        }

        // Potong stok alat secara atomik
        state.alat[alat_idx].stok_tersedia -= jumlah_pinjam;

        state.peminjaman[pinjam_idx].status = "dipinjam".to_string();
        state.peminjaman[pinjam_idx].disetujui_oleh = Some(auth_user.id);
        state.peminjaman[pinjam_idx].nama_petugas = Some(auth_user.nama_lengkap.clone());
        state.peminjaman[pinjam_idx].catatan_petugas = payload.catatan.clone();

        let updated = state.peminjaman[pinjam_idx].clone();

        state.add_log(
            Some(auth_user.id),
            &auth_user.username,
            auth_user.role.as_str(),
            "APPROVE",
            "PEMINJAMAN",
            &format!("Menyetujui peminjaman {} (Stok alat dipotong: {} unit)", kode_pinjam, jumlah_pinjam),
        );
        state.save_to_disk();

        (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: format!("Peminjaman {} berhasil disetujui. Alat berstatus dipinjam.", kode_pinjam),
                data: Some(updated),
            }),
        )
    } else {
        // Ditolak
        state.peminjaman[pinjam_idx].status = "ditolak".to_string();
        state.peminjaman[pinjam_idx].disetujui_oleh = Some(auth_user.id);
        state.peminjaman[pinjam_idx].nama_petugas = Some(auth_user.nama_lengkap.clone());
        state.peminjaman[pinjam_idx].catatan_petugas = payload.catatan.clone();

        let updated = state.peminjaman[pinjam_idx].clone();

        state.add_log(
            Some(auth_user.id),
            &auth_user.username,
            auth_user.role.as_str(),
            "REJECT",
            "PEMINJAMAN",
            &format!("Menolak peminjaman {}. Alasan: {:?}", kode_pinjam, payload.catatan),
        );
        state.save_to_disk();

        (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: format!("Peminjaman {} telah ditolak.", kode_pinjam),
                data: Some(updated),
            }),
        )
    }
}
