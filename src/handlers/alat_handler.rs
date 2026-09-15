use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Local;
use serde::Deserialize;
use crate::auth::{extract_auth_user, require_role};
use crate::db::Db;
use crate::models::{Alat, ApiResponse, CreateAlatRequest, Role, UpdateAlatRequest};

#[derive(Debug, Deserialize)]
pub struct AlatFilter {
    pub search: Option<String>,
    pub kategori_id: Option<i64>,
    pub kondisi: Option<String>,
    pub tersedia_saja: Option<bool>,
}

pub async fn list_alat(
    headers: HeaderMap,
    State(db): State<Db>,
    Query(filter): Query<AlatFilter>,
) -> (StatusCode, Json<ApiResponse<Vec<Alat>>>) {
    // Memeriksa autentikasi
    if let Err((status, err)) = extract_auth_user(&headers, &db).await {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    let state = db.read().await;
    let mut results: Vec<Alat> = state.alat.clone();

    // 1. Filter Kategori
    if let Some(kat_id) = filter.kategori_id {
        if kat_id > 0 {
            results.retain(|a| a.kategori_id == kat_id);
        }
    }

    // 2. Filter Kondisi
    if let Some(ref cond) = filter.kondisi {
        if !cond.trim().is_empty() && cond != "semua" {
            results.retain(|a| a.kondisi.to_lowercase() == cond.to_lowercase());
        }
    }

    // 3. Filter Hanya yang Tersedia
    if let Some(true) = filter.tersedia_saja {
        results.retain(|a| a.stok_tersedia > 0);
    }

    // 4. Filter Pencarian Teks (Nama / Kode / Spesifikasi)
    if let Some(ref q) = filter.search {
        let query_lower = q.trim().to_lowercase();
        if !query_lower.is_empty() {
            results.retain(|a| {
                a.nama_alat.to_lowercase().contains(&query_lower)
                    || a.kode_alat.to_lowercase().contains(&query_lower)
                    || a.spesifikasi.to_lowercase().contains(&query_lower)
                    || a.nama_kategori.to_lowercase().contains(&query_lower)
            });
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: format!("Berhasil memuat {} data alat.", results.len()),
            data: Some(results),
        }),
    )
}

pub async fn create_alat(
    headers: HeaderMap,
    State(db): State<Db>,
    Json(payload): Json<CreateAlatRequest>,
) -> (StatusCode, Json<ApiResponse<Alat>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    if let Err((status, err)) = require_role(&auth_user, &[Role::Admin, Role::Petugas]) {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    let kode = payload.kode_alat.trim();
    let nama = payload.nama_alat.trim();

    if kode.is_empty() || nama.is_empty() || payload.stok_total <= 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: "Kode alat, nama alat, dan stok (minimal 1) wajib diisi dengan benar!".to_string(),
                data: None,
            }),
        );
    }

    let mut state = db.write().await;

    // Periksa keunikan kode alat
    if state.alat.iter().any(|a| a.kode_alat.to_lowercase() == kode.to_lowercase()) {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse {
                success: false,
                message: format!("Kode alat '{}' sudah digunakan pada inventaris lain!", kode),
                data: None,
            }),
        );
    }

    // Ambil nama kategori
    let nama_kategori = match state.kategori.iter().find(|k| k.id == payload.kategori_id) {
        Some(k) => k.nama_kategori.clone(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    success: false,
                    message: "Kategori yang dipilih tidak valid.".to_string(),
                    data: None,
                }),
            );
        }
    };

    let new_id = state.next_alat_id();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let new_alat = Alat {
        id: new_id,
        kode_alat: kode.to_string(),
        nama_alat: nama.to_string(),
        kategori_id: payload.kategori_id,
        nama_kategori,
        stok_total: payload.stok_total,
        stok_tersedia: payload.stok_total, // Awalnya seluruh stok tersedia
        kondisi: if payload.kondisi.trim().is_empty() { "baik".to_string() } else { payload.kondisi.trim().to_string() },
        lokasi: if payload.lokasi.trim().is_empty() { "Gudang Utama".to_string() } else { payload.lokasi.trim().to_string() },
        spesifikasi: payload.spesifikasi.trim().to_string(),
        tarif_denda_harian: if payload.tarif_denda_harian <= 0.0 { 5000.0 } else { payload.tarif_denda_harian },
        created_at: now,
    };

    state.alat.push(new_alat.clone());
    state.add_log(
        Some(auth_user.id),
        &auth_user.username,
        auth_user.role.as_str(),
        "CREATE",
        "ALAT",
        &format!("Menambahkan alat baru: '{}' (Kode: {}), Stok: {}", nama, kode, payload.stok_total),
    );
    state.save_to_disk();

    (
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            message: "Alat baru berhasil ditambahkan ke inventaris.".to_string(),
            data: Some(new_alat),
        }),
    )
}

pub async fn update_alat(
    Path(id): Path<i64>,
    headers: HeaderMap,
    State(db): State<Db>,
    Json(payload): Json<UpdateAlatRequest>,
) -> (StatusCode, Json<ApiResponse<Alat>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    if let Err((status, err)) = require_role(&auth_user, &[Role::Admin, Role::Petugas]) {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    let mut state = db.write().await;
    let idx = match state.alat.iter().position(|a| a.id == id) {
        Some(i) => i,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "Alat tidak ditemukan.".to_string(),
                    data: None,
                }),
            );
        }
    };

    if let Some(kode) = payload.kode_alat {
        let trim_kode = kode.trim();
        if !trim_kode.is_empty() {
            if state.alat.iter().any(|a| a.id != id && a.kode_alat.to_lowercase() == trim_kode.to_lowercase()) {
                return (
                    StatusCode::CONFLICT,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Kode alat '{}' sudah digunakan!", trim_kode),
                        data: None,
                    }),
                );
            }
            state.alat[idx].kode_alat = trim_kode.to_string();
        }
    }

    if let Some(nama) = payload.nama_alat {
        if !nama.trim().is_empty() {
            state.alat[idx].nama_alat = nama.trim().to_string();
        }
    }

    if let Some(kat_id) = payload.kategori_id {
        let kat_nama = state.kategori.iter().find(|k| k.id == kat_id).map(|k| k.nama_kategori.clone());
        if let Some(nama) = kat_nama {
            state.alat[idx].kategori_id = kat_id;
            state.alat[idx].nama_kategori = nama;
        }
    }

    if let Some(stok_tot) = payload.stok_total {
        let selisih = stok_tot - state.alat[idx].stok_total;
        state.alat[idx].stok_total = stok_tot;
        state.alat[idx].stok_tersedia = (state.alat[idx].stok_tersedia + selisih).max(0);
    }

    if let Some(stok_ters) = payload.stok_tersedia {
        state.alat[idx].stok_tersedia = stok_ters.min(state.alat[idx].stok_total).max(0);
    }

    if let Some(cond) = payload.kondisi {
        state.alat[idx].kondisi = cond;
    }

    if let Some(lok) = payload.lokasi {
        state.alat[idx].lokasi = lok;
    }

    if let Some(spek) = payload.spesifikasi {
        state.alat[idx].spesifikasi = spek;
    }

    if let Some(denda) = payload.tarif_denda_harian {
        state.alat[idx].tarif_denda_harian = denda.max(0.0);
    }

    let updated = state.alat[idx].clone();
    state.add_log(
        Some(auth_user.id),
        &auth_user.username,
        auth_user.role.as_str(),
        "UPDATE",
        "ALAT",
        &format!("Memperbarui alat ID {}: '{}' (Kode: {})", id, updated.nama_alat, updated.kode_alat),
    );
    state.save_to_disk();

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Data alat berhasil diperbarui.".to_string(),
            data: Some(updated),
        }),
    )
}

pub async fn delete_alat(
    Path(id): Path<i64>,
    headers: HeaderMap,
    State(db): State<Db>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    if let Err((status, err)) = require_role(&auth_user, &[Role::Admin, Role::Petugas]) {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    let mut state = db.write().await;

    // Cek apakah alat sedang dipinjam
    let is_loaned = state.peminjaman.iter().any(|p| p.alat_id == id && (p.status == "dipinjam" || p.status == "menunggu"));
    if is_loaned {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: "Gagal menghapus alat: Alat ini sedang dalam transaksi peminjaman aktif atau pending!".to_string(),
                data: None,
            }),
        );
    }

    let idx = match state.alat.iter().position(|a| a.id == id) {
        Some(i) => i,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "Alat tidak ditemukan.".to_string(),
                    data: None,
                }),
            );
        }
    };

    let deleted_name = state.alat[idx].nama_alat.clone();
    let deleted_kode = state.alat[idx].kode_alat.clone();
    state.alat.remove(idx);

    state.add_log(
        Some(auth_user.id),
        &auth_user.username,
        auth_user.role.as_str(),
        "DELETE",
        "ALAT",
        &format!("Menghapus alat ID {}: '{}' (Kode: {})", id, deleted_name, deleted_kode),
    );
    state.save_to_disk();

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: format!("Alat '{}' berhasil dihapus dari inventaris.", deleted_name),
            data: None,
        }),
    )
}
