use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Local;
use crate::auth::{extract_auth_user, require_role};
use crate::db::Db;
use crate::models::{ApiResponse, CreateKategoriRequest, Kategori, Role};

pub async fn list_kategori(
    headers: HeaderMap,
    State(db): State<Db>,
) -> (StatusCode, Json<ApiResponse<Vec<Kategori>>>) {
    if let Err((status, err)) = extract_auth_user(&headers, &db).await {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    let state = db.read().await;
    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Data kategori berhasil dimuat.".to_string(),
            data: Some(state.kategori.clone()),
        }),
    )
}

pub async fn create_kategori(
    headers: HeaderMap,
    State(db): State<Db>,
    Json(payload): Json<CreateKategoriRequest>,
) -> (StatusCode, Json<ApiResponse<Kategori>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    if let Err((status, err)) = require_role(&auth_user, &[Role::Admin, Role::Petugas]) {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    let nama = payload.nama_kategori.trim();
    if nama.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: "Nama kategori tidak boleh kosong!".to_string(),
                data: None,
            }),
        );
    }

    let mut state = db.write().await;
    if state.kategori.iter().any(|k| k.nama_kategori.to_lowercase() == nama.to_lowercase()) {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse {
                success: false,
                message: format!("Kategori '{}' sudah ada dalam sistem!", nama),
                data: None,
            }),
        );
    }

    let new_id = state.next_kategori_id();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let new_kat = Kategori {
        id: new_id,
        nama_kategori: nama.to_string(),
        deskripsi: payload.deskripsi.trim().to_string(),
        created_at: now,
    };

    state.kategori.push(new_kat.clone());
    state.add_log(
        Some(auth_user.id),
        &auth_user.username,
        auth_user.role.as_str(),
        "CREATE",
        "KATEGORI",
        &format!("Menambahkan kategori baru: '{}'", nama),
    );
    state.save_to_disk();

    (
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            message: "Kategori baru berhasil ditambahkan.".to_string(),
            data: Some(new_kat),
        }),
    )
}

pub async fn update_kategori(
    Path(id): Path<i64>,
    headers: HeaderMap,
    State(db): State<Db>,
    Json(payload): Json<CreateKategoriRequest>,
) -> (StatusCode, Json<ApiResponse<Kategori>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    if let Err((status, err)) = require_role(&auth_user, &[Role::Admin, Role::Petugas]) {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    let mut state = db.write().await;
    let idx = match state.kategori.iter().position(|k| k.id == id) {
        Some(i) => i,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "Kategori tidak ditemukan.".to_string(),
                    data: None,
                }),
            );
        }
    };

    let nama = payload.nama_kategori.trim();
    if !nama.is_empty() {
        state.kategori[idx].nama_kategori = nama.to_string();
        // Update denormalized nama_kategori pada tabel alat
        for alat in state.alat.iter_mut() {
            if alat.kategori_id == id {
                alat.nama_kategori = nama.to_string();
            }
        }
    }
    state.kategori[idx].deskripsi = payload.deskripsi.trim().to_string();

    let updated = state.kategori[idx].clone();
    state.add_log(
        Some(auth_user.id),
        &auth_user.username,
        auth_user.role.as_str(),
        "UPDATE",
        "KATEGORI",
        &format!("Memperbarui kategori ID {}: '{}'", id, updated.nama_kategori),
    );
    state.save_to_disk();

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Kategori berhasil diperbarui.".to_string(),
            data: Some(updated),
        }),
    )
}

pub async fn delete_kategori(
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
    // Cek integritas referensial: apakah ada alat dengan kategori ini?
    let has_tools = state.alat.iter().any(|a| a.kategori_id == id);
    if has_tools {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: "Gagal menghapus kategori: Masih ada alat yang terdaftar dalam kategori ini! Harap pindahkan alat terlebih dahulu.".to_string(),
                data: None,
            }),
        );
    }

    let idx = match state.kategori.iter().position(|k| k.id == id) {
        Some(i) => i,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "Kategori tidak ditemukan.".to_string(),
                    data: None,
                }),
            );
        }
    };

    let deleted_name = state.kategori[idx].nama_kategori.clone();
    state.kategori.remove(idx);
    state.add_log(
        Some(auth_user.id),
        &auth_user.username,
        auth_user.role.as_str(),
        "DELETE",
        "KATEGORI",
        &format!("Menghapus kategori ID {}: '{}'", id, deleted_name),
    );
    state.save_to_disk();

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: format!("Kategori '{}' berhasil dihapus.", deleted_name),
            data: None,
        }),
    )
}
