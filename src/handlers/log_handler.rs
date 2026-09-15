use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use crate::auth::{extract_auth_user, require_role};
use crate::db::Db;
use crate::models::{ApiResponse, LogAktivitas, Role};

#[derive(Debug, Deserialize)]
pub struct LogFilter {
    pub search: Option<String>,
    pub limit: Option<usize>,
}

pub async fn list_logs(
    headers: HeaderMap,
    State(db): State<Db>,
    Query(filter): Query<LogFilter>,
) -> (StatusCode, Json<ApiResponse<Vec<LogAktivitas>>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    if let Err((status, err)) = require_role(&auth_user, &[Role::Admin]) {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    let state = db.read().await;
    let mut logs = state.log_aktivitas.clone();

    if let Some(ref q) = filter.search {
        let q_lower = q.trim().to_lowercase();
        if !q_lower.is_empty() {
            logs.retain(|l| {
                l.username.to_lowercase().contains(&q_lower)
                    || l.aksi.to_lowercase().contains(&q_lower)
                    || l.modul.to_lowercase().contains(&q_lower)
                    || l.detail.to_lowercase().contains(&q_lower)
            });
        }
    }

    // Urutkan dari aktivitas terbaru
    logs.reverse();

    let limit = filter.limit.unwrap_or(100);
    if logs.len() > limit {
        logs.truncate(limit);
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: format!("Berhasil memuat {} catatan log aktivitas.", logs.len()),
            data: Some(logs),
        }),
    )
}
