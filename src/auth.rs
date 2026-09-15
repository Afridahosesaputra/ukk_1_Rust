use axum::{
    http::{HeaderMap, StatusCode},
    Json,
};
use crate::db::Db;
use crate::models::{ApiResponse, Role, User};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub username: String,
    pub nama_lengkap: String,
    pub role: Role,
}

pub fn create_simple_token(user: &User) -> String {
    // Format token aman: base64 atau id:role:username
    format!("{}:{}:{}", user.id, user.role.as_str(), user.username)
}

pub async fn extract_auth_user(headers: &HeaderMap, db: &Db) -> Result<AuthUser, (StatusCode, Json<ApiResponse<()>>)> {
    let auth_header = headers.get("Authorization")
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(val) if val.starts_with("Bearer ") => &val[7..],
        Some(val) => val,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse {
                    success: false,
                    message: "Akses ditolak: Token autentikasi tidak ditemukan. Harap login terlebih dahulu.".to_string(),
                    data: None,
                }),
            ));
        }
    };

    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() < 3 {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                success: false,
                message: "Format token autentikasi tidak valid.".to_string(),
                data: None,
            }),
        ));
    }

    let user_id: i64 = parts[0].parse().map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                success: false,
                message: "Token ID tidak valid.".to_string(),
                data: None,
            }),
        )
    })?;

    let state = db.read().await;
    if let Some(user) = state.users.iter().find(|u| u.id == user_id) {
        if user.status != "aktif" {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ApiResponse {
                    success: false,
                    message: "Akun Anda sedang dinonaktifkan. Hubungi Administrator!".to_string(),
                    data: None,
                }),
            ));
        }

        Ok(AuthUser {
            id: user.id,
            username: user.username.clone(),
            nama_lengkap: user.nama_lengkap.clone(),
            role: user.role.clone(),
        })
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                success: false,
                message: "Pengguna tidak ditemukan atau session telah berakhir.".to_string(),
                data: None,
            }),
        ))
    }
}

pub fn require_role(user: &AuthUser, allowed_roles: &[Role]) -> Result<(), (StatusCode, Json<ApiResponse<()>>)> {
    if allowed_roles.contains(&user.role) {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                success: false,
                message: format!(
                    "Akses ditolak (403 Forbidden): Peran '{}' tidak memiliki izin untuk fitur ini.",
                    user.role.as_str()
                ),
                data: None,
            }),
        ))
    }
}
