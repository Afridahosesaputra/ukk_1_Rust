use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use crate::auth::{create_simple_token, extract_auth_user};
use crate::db::Db;
use crate::models::{ApiResponse, LoginRequest, LoginResponse, User};

pub async fn login(
    State(db): State<Db>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<LoginResponse>) {
    let username = payload.username.trim();
    let password = payload.password.trim();

    if username.is_empty() || password.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(LoginResponse {
                success: false,
                message: "Username dan Password tidak boleh kosong!".to_string(),
                token: "".to_string(),
                user: None,
            }),
        );
    }

    let mut state = db.write().await;
    let found_user = state.users.iter().find(|u| u.username == username).cloned();

    match found_user {
        Some(user) => {
            if user.status != "aktif" {
                return (
                    StatusCode::FORBIDDEN,
                    Json(LoginResponse {
                        success: false,
                        message: "Akun Anda dinonaktifkan. Hubungi Administrator!".to_string(),
                        token: "".to_string(),
                        user: None,
                    }),
                );
            }

            if user.password == password {
                let token = create_simple_token(&user);
                let user_id = user.id;
                let role_str = user.role.as_str().to_string();
                let username_str = user.username.clone();

                state.add_log(
                    Some(user_id),
                    &username_str,
                    &role_str,
                    "LOGIN",
                    "AUTH",
                    &format!("Pengguna '{}' berhasil login sebagai '{}'", username_str, role_str),
                );

                (
                    StatusCode::OK,
                    Json(LoginResponse {
                        success: true,
                        message: format!("Selamat datang, {}! Anda berhasil login.", user.nama_lengkap),
                        token,
                        user: Some(user),
                    }),
                )
            } else {
                state.add_log(
                    None,
                    username,
                    "unknown",
                    "LOGIN_FAILED",
                    "AUTH",
                    &format!("Percobaan login gagal: Password salah untuk username '{}'", username),
                );

                (
                    StatusCode::UNAUTHORIZED,
                    Json(LoginResponse {
                        success: false,
                        message: "Username atau Password yang Anda masukkan salah!".to_string(),
                        token: "".to_string(),
                        user: None,
                    }),
                )
            }
        }
        None => {
            state.add_log(
                None,
                username,
                "unknown",
                "LOGIN_FAILED",
                "AUTH",
                &format!("Percobaan login gagal: Username '{}' tidak terdaftar", username),
            );

            (
                StatusCode::UNAUTHORIZED,
                Json(LoginResponse {
                    success: false,
                    message: "Username tidak ditemukan dalam sistem!".to_string(),
                    token: "".to_string(),
                    user: None,
                }),
            )
        }
    }
}

pub async fn logout(
    headers: HeaderMap,
    State(db): State<Db>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    if let Ok(user) = extract_auth_user(&headers, &db).await {
        let mut state = db.write().await;
        state.add_log(
            Some(user.id),
            &user.username,
            user.role.as_str(),
            "LOGOUT",
            "AUTH",
            &format!("Pengguna '{}' telah logout dari sistem.", user.username),
        );
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Anda telah berhasil logout.".to_string(),
            data: None,
        }),
    )
}

pub async fn get_me(
    headers: HeaderMap,
    State(db): State<Db>,
) -> (StatusCode, Json<ApiResponse<User>>) {
    let user_res = extract_auth_user(&headers, &db).await;
    match user_res {
        Ok(auth_user) => {
            let state = db.read().await;
            if let Some(user) = state.users.iter().find(|u| u.id == auth_user.id).cloned() {
                (
                    StatusCode::OK,
                    Json(ApiResponse {
                        success: true,
                        message: "Data profil pengguna berhasil dimuat.".to_string(),
                        data: Some(user),
                    }),
                )
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse {
                        success: false,
                        message: "Pengguna tidak ditemukan.".to_string(),
                        data: None,
                    }),
                )
            }
        }
        Err((status, err_json)) => (
            status,
            Json(ApiResponse {
                success: false,
                message: err_json.message.clone(),
                data: None,
            }),
        ),
    }
}
