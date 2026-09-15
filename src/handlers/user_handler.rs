use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Local;
use crate::auth::{extract_auth_user, require_role};
use crate::db::Db;
use crate::models::{ApiResponse, CreateUserRequest, Role, UpdateUserRequest, User};

pub async fn list_users(
    headers: HeaderMap,
    State(db): State<Db>,
) -> (StatusCode, Json<ApiResponse<Vec<User>>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    if let Err((status, err)) = require_role(&auth_user, &[Role::Admin]) {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    let state = db.read().await;
    let users = state.users.clone();

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: format!("Berhasil memuat {} pengguna.", users.len()),
            data: Some(users),
        }),
    )
}

pub async fn create_user(
    headers: HeaderMap,
    State(db): State<Db>,
    Json(payload): Json<CreateUserRequest>,
) -> (StatusCode, Json<ApiResponse<User>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    if let Err((status, err)) = require_role(&auth_user, &[Role::Admin]) {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    let username = payload.username.trim();
    let password = payload.password.trim();
    let nama_lengkap = payload.nama_lengkap.trim();

    if username.is_empty() || password.is_empty() || nama_lengkap.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: "Username, Password, dan Nama Lengkap wajib diisi!".to_string(),
                data: None,
            }),
        );
    }

    let mut state = db.write().await;
    if state.users.iter().any(|u| u.username.to_lowercase() == username.to_lowercase()) {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse {
                success: false,
                message: format!("Username '{}' sudah digunakan. Silakan gunakan username lain.", username),
                data: None,
            }),
        );
    }

    let new_id = state.next_user_id();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let role = Role::from_str(&payload.role);

    let new_user = User {
        id: new_id,
        username: username.to_string(),
        password: password.to_string(),
        nama_lengkap: nama_lengkap.to_string(),
        no_telepon: payload.no_telepon.trim().to_string(),
        alamat: payload.alamat.trim().to_string(),
        role: role.clone(),
        status: "aktif".to_string(),
        created_at: now,
    };

    state.users.push(new_user.clone());
    state.add_log(
        Some(auth_user.id),
        &auth_user.username,
        auth_user.role.as_str(),
        "CREATE",
        "USER",
        &format!("Menambahkan pengguna baru: '{}' ({}) role: '{}'", username, nama_lengkap, role.as_str()),
    );
    state.save_to_disk();

    (
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            message: "Pengguna baru berhasil ditambahkan.".to_string(),
            data: Some(new_user),
        }),
    )
}

pub async fn update_user(
    Path(id): Path<i64>,
    headers: HeaderMap,
    State(db): State<Db>,
    Json(payload): Json<UpdateUserRequest>,
) -> (StatusCode, Json<ApiResponse<User>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    if let Err((status, err)) = require_role(&auth_user, &[Role::Admin]) {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    let mut state = db.write().await;
    let user_idx = match state.users.iter().position(|u| u.id == id) {
        Some(idx) => idx,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: format!("Pengguna dengan ID {} tidak ditemukan.", id),
                    data: None,
                }),
            );
        }
    };

    if let Some(nama) = payload.nama_lengkap {
        if !nama.trim().is_empty() {
            state.users[user_idx].nama_lengkap = nama.trim().to_string();
        }
    }
    if let Some(pass) = payload.password {
        if !pass.trim().is_empty() {
            state.users[user_idx].password = pass.trim().to_string();
        }
    }
    if let Some(telp) = payload.no_telepon {
        state.users[user_idx].no_telepon = telp.trim().to_string();
    }
    if let Some(almt) = payload.alamat {
        state.users[user_idx].alamat = almt.trim().to_string();
    }
    if let Some(r) = payload.role {
        // Cegah admin mengubah dirinya sendiri menjadi non-admin jika cuma ada 1 admin
        state.users[user_idx].role = Role::from_str(&r);
    }
    if let Some(st) = payload.status {
        state.users[user_idx].status = st.trim().to_string();
    }

    let updated_user = state.users[user_idx].clone();
    state.add_log(
        Some(auth_user.id),
        &auth_user.username,
        auth_user.role.as_str(),
        "UPDATE",
        "USER",
        &format!("Memperbarui data pengguna ID {}: '{}'", id, updated_user.username),
    );
    state.save_to_disk();

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Data pengguna berhasil diperbarui.".to_string(),
            data: Some(updated_user),
        }),
    )
}

pub async fn delete_user(
    Path(id): Path<i64>,
    headers: HeaderMap,
    State(db): State<Db>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    if let Err((status, err)) = require_role(&auth_user, &[Role::Admin]) {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    if auth_user.id == id {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: "Tidak dapat menghapus akun Anda sendiri yang sedang aktif digunakan!".to_string(),
                data: None,
            }),
        );
    }

    let mut state = db.write().await;

    // Periksa apakah user memiliki riwayat peminjaman aktif
    let has_active_loan = state.peminjaman.iter().any(|p| p.user_id == id && (p.status == "dipinjam" || p.status == "menunggu"));
    if has_active_loan {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: "Gagal menghapus: Pengguna ini masih memiliki peminjaman aktif atau pending!".to_string(),
                data: None,
            }),
        );
    }

    let user_idx = match state.users.iter().position(|u| u.id == id) {
        Some(idx) => idx,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "Pengguna tidak ditemukan.".to_string(),
                    data: None,
                }),
            );
        }
    };

    let deleted_username = state.users[user_idx].username.clone();
    state.users.remove(user_idx);
    state.add_log(
        Some(auth_user.id),
        &auth_user.username,
        auth_user.role.as_str(),
        "DELETE",
        "USER",
        &format!("Menghapus pengguna ID {}: '{}'", id, deleted_username),
    );
    state.save_to_disk();

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: format!("Pengguna '{}' berhasil dihapus.", deleted_username),
            data: None,
        }),
    )
}
